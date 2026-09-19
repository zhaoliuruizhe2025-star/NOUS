use std::{error::Error, fmt};

use sqlx::{error::ErrorKind, pool::PoolConnection, Connection, Sqlite, SqlitePool};
use tauri::{Manager, Runtime};
use tauri_plugin_sql::{DbInstances, DbPool};

use crate::DATABASE_URL;

/// The plugin-managed SQLite pool that later persistence phases will use.
#[derive(Clone)]
pub(crate) struct SharedSqlitePool {
    pool: SqlitePool,
}

impl SharedSqlitePool {
    fn from_manager<R: Runtime>(manager: &impl Manager<R>) -> Result<Self, SharedPoolError> {
        let instances = manager
            .try_state::<DbInstances>()
            .ok_or(SharedPoolError::PluginStateUnavailable)?;

        Self::from_instances(&instances)
    }

    fn from_instances(instances: &DbInstances) -> Result<Self, SharedPoolError> {
        let instances = instances
            .0
            .try_read()
            .map_err(|_| SharedPoolError::DatabaseMapUnavailable)?;
        let database = instances
            .get(DATABASE_URL)
            .ok_or(SharedPoolError::DatabaseNotPreloaded(DATABASE_URL))?;

        let DbPool::Sqlite(pool) = database;

        Ok(Self { pool: pool.clone() })
    }

    pub(crate) fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    #[cfg(test)]
    pub(crate) fn from_test_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Acquires and verifies the exact connection that later SQL must use.
    #[allow(dead_code)]
    pub(crate) async fn acquire_verified_connection(
        &self,
    ) -> Result<VerifiedSqliteConnection, PersistenceError> {
        VerifiedSqliteConnection::acquire(&self.pool).await
    }
}

/// A pooled connection whose SQLite foreign-key setting was verified on itself.
/// Future repository operations must retain this connection or a transaction derived from it.
#[allow(dead_code)]
pub(crate) struct VerifiedSqliteConnection {
    connection: PoolConnection<Sqlite>,
}

#[allow(dead_code)]
impl VerifiedSqliteConnection {
    async fn acquire(pool: &SqlitePool) -> Result<Self, PersistenceError> {
        let mut connection = pool.acquire().await.map_err(PersistenceError::from)?;
        verify_foreign_keys(&mut connection).await?;

        Ok(Self { connection })
    }

    pub(crate) fn connection(&mut self) -> &mut PoolConnection<Sqlite> {
        &mut self.connection
    }
}

async fn verify_foreign_keys(
    connection: &mut PoolConnection<Sqlite>,
) -> Result<(), PersistenceError> {
    let enabled: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&mut **connection)
        .await
        .map_err(PersistenceError::from)?;

    if enabled != 1 {
        return Err(PersistenceError::NotReady(
            "SQLite foreign-key enforcement is disabled for the operation connection".into(),
        ));
    }

    Ok(())
}

/// Installs a narrow Rust owner for the exact pool opened and migrated by the SQL plugin.
///
/// Tauri initializes plugins during `Builder::build`, before this application setup hook
/// runs. The SQL plugin manages `DbInstances` only after every configured preload has
/// connected and its registered migrations have completed, so successful resolution here
/// is the repository-readiness barrier.
pub(crate) fn install_shared_sqlite_pool<R: Runtime>(
    manager: &impl Manager<R>,
) -> Result<(), SharedPoolError> {
    let database = SharedSqlitePool::from_manager(manager)?;

    // This annotation compile-proves compatibility with the direct SQLx 0.8.6 type.
    let _: &SqlitePool = database.pool();

    if !manager.manage(database) {
        return Err(SharedPoolError::OwnerAlreadyManaged);
    }

    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SharedPoolError {
    PluginStateUnavailable,
    DatabaseMapUnavailable,
    DatabaseNotPreloaded(&'static str),
    OwnerAlreadyManaged,
}

impl fmt::Display for SharedPoolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PluginStateUnavailable => {
                write!(formatter, "the SQL plugin database state is unavailable")
            }
            Self::DatabaseMapUnavailable => {
                write!(formatter, "the SQL plugin database map is unavailable")
            }
            Self::DatabaseNotPreloaded(database_url) => {
                write!(
                    formatter,
                    "the configured database was not preloaded: {database_url}"
                )
            }
            Self::OwnerAlreadyManaged => {
                write!(formatter, "the shared SQLite pool owner is already managed")
            }
        }
    }
}

impl Error for SharedPoolError {}

/// Persistence failures with narrow domain-facing categories and local diagnostics.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PersistenceError {
    #[allow(dead_code)]
    NotFound {
        entity: &'static str,
        id: String,
    },
    #[allow(dead_code)]
    ConstraintViolation {
        operation: &'static str,
        detail: String,
    },
    DomainReconstruction {
        entity: &'static str,
        field: &'static str,
        detail: String,
    },
    Storage(String),
    #[allow(dead_code)]
    Migration(String),
    #[allow(dead_code)]
    NotReady(String),
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { entity, id } => write!(formatter, "{entity} was not found: {id}"),
            Self::ConstraintViolation { operation, detail } => {
                write!(
                    formatter,
                    "constraint violation during {operation}: {detail}"
                )
            }
            Self::DomainReconstruction {
                entity,
                field,
                detail,
            } => {
                write!(formatter, "cannot reconstruct {entity}.{field}: {detail}")
            }
            Self::Storage(detail) => write!(formatter, "storage failure: {detail}"),
            Self::Migration(detail) => write!(formatter, "migration failure: {detail}"),
            Self::NotReady(detail) => write!(formatter, "persistence is not ready: {detail}"),
        }
    }
}

impl Error for PersistenceError {}

impl From<sqlx::Error> for PersistenceError {
    fn from(error: sqlx::Error) -> Self {
        Self::Storage(error.to_string())
    }
}

fn write_error(operation: &'static str, error: sqlx::Error) -> PersistenceError {
    if let sqlx::Error::Database(database_error) = &error {
        if matches!(
            database_error.kind(),
            ErrorKind::UniqueViolation
                | ErrorKind::ForeignKeyViolation
                | ErrorKind::NotNullViolation
                | ErrorKind::CheckViolation
        ) {
            return PersistenceError::ConstraintViolation {
                operation,
                detail: database_error.message().to_owned(),
            };
        }
    }

    PersistenceError::Storage(error.to_string())
}

fn reconstruct<T>(
    entity: &'static str,
    field: &'static str,
    result: Result<T, impl fmt::Display>,
) -> Result<T, PersistenceError> {
    result.map_err(|error| PersistenceError::DomainReconstruction {
        entity,
        field,
        detail: error.to_string(),
    })
}

fn percentage(
    entity: &'static str,
    field: &'static str,
    value: i64,
) -> Result<u8, PersistenceError> {
    reconstruct(entity, field, u8::try_from(value))
}

fn revision_number(entity: &'static str, value: i64) -> Result<u32, PersistenceError> {
    reconstruct(entity, "revision_number", u32::try_from(value))
}

type ThoughtDatabaseRow = (String, String, Option<String>, String, Option<i64>, i64);
type BeliefRevisionDatabaseRow = (
    String,
    String,
    i64,
    String,
    Option<i64>,
    Option<String>,
    String,
    i64,
);
type ValueRevisionDatabaseRow = (
    String,
    String,
    i64,
    String,
    Option<i64>,
    Option<String>,
    String,
    i64,
);

#[derive(Clone, Debug)]
pub(crate) struct InitialBeliefRevisionInput {
    id: crate::domain::BeliefRevisionId,
    proposition: String,
    endorsement: Option<crate::domain::BeliefEndorsement>,
    change_note: Option<String>,
}

#[allow(dead_code)]
impl InitialBeliefRevisionInput {
    pub(crate) fn new(
        id: crate::domain::BeliefRevisionId,
        proposition: impl Into<String>,
        endorsement: Option<crate::domain::BeliefEndorsement>,
        change_note: Option<String>,
    ) -> Self {
        Self {
            id,
            proposition: proposition.into(),
            endorsement,
            change_note,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AppendBeliefRevisionInput {
    id: crate::domain::BeliefRevisionId,
    proposition: String,
    endorsement: Option<crate::domain::BeliefEndorsement>,
    change_note: Option<String>,
    origin: crate::domain::RevisionOrigin,
}

#[allow(dead_code)]
impl AppendBeliefRevisionInput {
    pub(crate) fn new(
        id: crate::domain::BeliefRevisionId,
        proposition: impl Into<String>,
        endorsement: Option<crate::domain::BeliefEndorsement>,
        change_note: Option<String>,
        origin: crate::domain::RevisionOrigin,
    ) -> Self {
        Self {
            id,
            proposition: proposition.into(),
            endorsement,
            change_note,
            origin,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct InitialValueRevisionInput {
    id: crate::domain::ValueRevisionId,
    label: String,
    importance: Option<crate::domain::ValueImportance>,
    change_note: Option<String>,
}

#[allow(dead_code)]
impl InitialValueRevisionInput {
    pub(crate) fn new(
        id: crate::domain::ValueRevisionId,
        label: impl Into<String>,
        importance: Option<crate::domain::ValueImportance>,
        change_note: Option<String>,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            importance,
            change_note,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AppendValueRevisionInput {
    id: crate::domain::ValueRevisionId,
    label: String,
    importance: Option<crate::domain::ValueImportance>,
    change_note: Option<String>,
    origin: crate::domain::RevisionOrigin,
}

#[allow(dead_code)]
impl AppendValueRevisionInput {
    pub(crate) fn new(
        id: crate::domain::ValueRevisionId,
        label: impl Into<String>,
        importance: Option<crate::domain::ValueImportance>,
        change_note: Option<String>,
        origin: crate::domain::RevisionOrigin,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            importance,
            change_note,
            origin,
        }
    }
}

fn invalid_write(operation: &'static str, error: impl fmt::Display) -> PersistenceError {
    PersistenceError::ConstraintViolation {
        operation,
        detail: error.to_string(),
    }
}

/// Explicit persistence operations for the approved Self Model schema.
/// Phase 4 intentionally has no command/application caller; Phase 6 will supply that boundary.
#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct SqliteSelfModelRepository {
    database: SharedSqlitePool,
}

#[allow(dead_code)]
impl SqliteSelfModelRepository {
    pub(crate) fn new(database: SharedSqlitePool) -> Self {
        Self { database }
    }

    pub(crate) async fn create_self_subject(
        &self,
        subject: &crate::domain::SelfSubject,
        created_at_ms: i64,
    ) -> Result<(), PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        sqlx::query("INSERT INTO self_subjects (id, display_name, created_at_ms) VALUES (?, ?, ?)")
            .bind(subject.id().as_str())
            .bind(subject.display_name())
            .bind(created_at_ms)
            .execute(&mut **connection.connection())
            .await
            .map_err(|error| write_error("create_self_subject", error))?;
        Ok(())
    }

    pub(crate) async fn load_self_subject(
        &self,
        id: &crate::domain::SelfSubjectId,
    ) -> Result<crate::domain::SelfSubject, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let row: Option<(String, String, i64)> = sqlx::query_as(
            "SELECT id, display_name, created_at_ms FROM self_subjects WHERE id = ?",
        )
        .bind(id.as_str())
        .fetch_optional(&mut **connection.connection())
        .await
        .map_err(PersistenceError::from)?;
        let (id, display_name, created_at_ms) = row.ok_or_else(|| PersistenceError::NotFound {
            entity: "SelfSubject",
            id: id.as_str().to_owned(),
        })?;
        SelfSubjectRow {
            id,
            display_name,
            created_at_ms,
        }
        .try_into()
    }

    pub(crate) async fn create_person_reference(
        &self,
        person: &crate::domain::PersonReference,
        created_at_ms: i64,
    ) -> Result<(), PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        sqlx::query("INSERT INTO person_references (id, subject_id, display_name, relationship_label, context_notes, created_at_ms) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(person.id().as_str())
            .bind(person.subject_id().as_str())
            .bind(person.display_name())
            .bind(person.relationship_label())
            .bind(person.context_notes())
            .bind(created_at_ms)
            .execute(&mut **connection.connection())
            .await
            .map_err(|error| write_error("create_person_reference", error))?;
        Ok(())
    }

    pub(crate) async fn load_person_reference(
        &self,
        id: &crate::domain::PersonReferenceId,
    ) -> Result<crate::domain::PersonReference, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let row: Option<(String, String, String, String, Option<String>, i64)> =
            sqlx::query_as("SELECT id, subject_id, display_name, relationship_label, context_notes, created_at_ms FROM person_references WHERE id = ?")
                .bind(id.as_str())
                .fetch_optional(&mut **connection.connection())
                .await
                .map_err(PersistenceError::from)?;
        let (id, subject_id, display_name, relationship_label, context_notes, created_at_ms) = row
            .ok_or_else(|| PersistenceError::NotFound {
                entity: "PersonReference",
                id: id.as_str().to_owned(),
            })?;
        PersonReferenceRow {
            id,
            subject_id,
            display_name,
            relationship_label,
            context_notes,
            created_at_ms,
        }
        .try_into()
    }

    pub(crate) async fn create_situation(
        &self,
        situation: &crate::domain::Situation,
        created_at_ms: i64,
    ) -> Result<(), PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        sqlx::query(
            "INSERT INTO situations (id, subject_id, description, created_at_ms) VALUES (?, ?, ?, ?)",
        )
        .bind(situation.id().as_str())
        .bind(situation.subject_id().as_str())
        .bind(situation.description())
        .bind(created_at_ms)
        .execute(&mut **connection.connection())
        .await
        .map_err(|error| write_error("create_situation", error))?;
        Ok(())
    }

    pub(crate) async fn load_situation(
        &self,
        id: &crate::domain::SituationId,
    ) -> Result<crate::domain::Situation, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let row: Option<(String, String, String, i64)> = sqlx::query_as(
            "SELECT id, subject_id, description, created_at_ms FROM situations WHERE id = ?",
        )
        .bind(id.as_str())
        .fetch_optional(&mut **connection.connection())
        .await
        .map_err(PersistenceError::from)?;
        let (id, subject_id, description, created_at_ms) =
            row.ok_or_else(|| PersistenceError::NotFound {
                entity: "Situation",
                id: id.as_str().to_owned(),
            })?;
        SituationRow {
            id,
            subject_id,
            description,
            created_at_ms,
        }
        .try_into()
    }

    pub(crate) async fn create_observation(
        &self,
        observation: &crate::domain::Observation,
        created_at_ms: i64,
    ) -> Result<(), PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        sqlx::query("INSERT INTO observations (id, subject_id, situation_id, content, created_at_ms) VALUES (?, ?, ?, ?, ?)")
            .bind(observation.id().as_str())
            .bind(observation.subject_id().as_str())
            .bind(observation.situation_id().map(crate::domain::SituationId::as_str))
            .bind(observation.content())
            .bind(created_at_ms)
            .execute(&mut **connection.connection())
            .await
            .map_err(|error| write_error("create_observation", error))?;
        Ok(())
    }

    pub(crate) async fn load_observation(
        &self,
        id: &crate::domain::ObservationId,
    ) -> Result<crate::domain::Observation, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let row: Option<(String, String, Option<String>, String, i64)> = sqlx::query_as(
            "SELECT id, subject_id, situation_id, content, created_at_ms FROM observations WHERE id = ?",
        )
        .bind(id.as_str())
        .fetch_optional(&mut **connection.connection())
        .await
        .map_err(PersistenceError::from)?;
        let (id, subject_id, situation_id, content, created_at_ms) =
            row.ok_or_else(|| PersistenceError::NotFound {
                entity: "Observation",
                id: id.as_str().to_owned(),
            })?;
        ObservationRow {
            id,
            subject_id,
            situation_id,
            content,
            created_at_ms,
        }
        .try_into()
    }

    pub(crate) async fn create_thought(
        &self,
        thought: &crate::domain::Thought,
        created_at_ms: i64,
    ) -> Result<(), PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        sqlx::query("INSERT INTO thoughts (id, subject_id, situation_id, content, confidence, created_at_ms) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(thought.id().as_str())
            .bind(thought.subject_id().as_str())
            .bind(thought.situation_id().map(crate::domain::SituationId::as_str))
            .bind(thought.content())
            .bind(thought.confidence().map(|value| i64::from(value.value())))
            .bind(created_at_ms)
            .execute(&mut **connection.connection())
            .await
            .map_err(|error| write_error("create_thought", error))?;
        Ok(())
    }

    pub(crate) async fn load_thought(
        &self,
        id: &crate::domain::ThoughtId,
    ) -> Result<crate::domain::Thought, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let row: Option<ThoughtDatabaseRow> = sqlx::query_as(
            "SELECT id, subject_id, situation_id, content, confidence, created_at_ms FROM thoughts WHERE id = ?",
        )
                .bind(id.as_str())
                .fetch_optional(&mut **connection.connection())
                .await
                .map_err(PersistenceError::from)?;
        let (id, subject_id, situation_id, content, confidence, created_at_ms) =
            row.ok_or_else(|| PersistenceError::NotFound {
                entity: "Thought",
                id: id.as_str().to_owned(),
            })?;
        ThoughtRow {
            id,
            subject_id,
            situation_id,
            content,
            confidence,
            created_at_ms,
        }
        .try_into()
    }

    pub(crate) async fn create_emotion(
        &self,
        emotion: &crate::domain::Emotion,
        created_at_ms: i64,
    ) -> Result<(), PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        sqlx::query("INSERT INTO emotions (id, subject_id, situation_id, label, intensity, created_at_ms) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(emotion.id().as_str())
            .bind(emotion.subject_id().as_str())
            .bind(emotion.situation_id().map(crate::domain::SituationId::as_str))
            .bind(emotion.label())
            .bind(i64::from(emotion.intensity().value()))
            .bind(created_at_ms)
            .execute(&mut **connection.connection())
            .await
            .map_err(|error| write_error("create_emotion", error))?;
        Ok(())
    }

    pub(crate) async fn load_emotion(
        &self,
        id: &crate::domain::EmotionId,
    ) -> Result<crate::domain::Emotion, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let row: Option<(String, String, Option<String>, String, i64, i64)> = sqlx::query_as(
            "SELECT id, subject_id, situation_id, label, intensity, created_at_ms FROM emotions WHERE id = ?",
        )
        .bind(id.as_str())
        .fetch_optional(&mut **connection.connection())
        .await
        .map_err(PersistenceError::from)?;
        let (id, subject_id, situation_id, label, intensity, created_at_ms) =
            row.ok_or_else(|| PersistenceError::NotFound {
                entity: "Emotion",
                id: id.as_str().to_owned(),
            })?;
        EmotionRow {
            id,
            subject_id,
            situation_id,
            label,
            intensity,
            created_at_ms,
        }
        .try_into()
    }

    pub(crate) async fn create_belief_with_initial_revision(
        &self,
        belief: &crate::domain::Belief,
        initial: InitialBeliefRevisionInput,
        created_at_ms: i64,
    ) -> Result<crate::domain::BeliefRevision, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let mut transaction = (**connection.connection())
            .begin_with("BEGIN IMMEDIATE")
            .await
            .map_err(PersistenceError::from)?;
        let revision_number =
            crate::domain::RevisionNumber::new(1).expect("one is always a valid revision number");
        let revision = match crate::domain::BeliefRevision::new(
            initial.id,
            belief.id().clone(),
            revision_number,
            initial.proposition,
            initial.endorsement,
            initial.change_note,
            crate::domain::RevisionOrigin::InitialUserEntry,
        ) {
            Ok(revision) => revision,
            Err(error) => {
                transaction
                    .rollback()
                    .await
                    .map_err(PersistenceError::from)?;
                return Err(invalid_write("create_belief_with_initial_revision", error));
            }
        };

        if let Err(error) =
            sqlx::query("INSERT INTO beliefs (id, subject_id, created_at_ms) VALUES (?, ?, ?)")
                .bind(belief.id().as_str())
                .bind(belief.subject_id().as_str())
                .bind(created_at_ms)
                .execute(&mut *transaction)
                .await
        {
            let error = write_error("create_belief_with_initial_revision", error);
            transaction
                .rollback()
                .await
                .map_err(PersistenceError::from)?;
            return Err(error);
        }

        if let Err(error) = sqlx::query("INSERT INTO belief_revisions (id, belief_id, revision_number, proposition, endorsement, change_note, origin, created_at_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(revision.id().as_str())
            .bind(revision.belief_id().as_str())
            .bind(i64::from(revision.revision_number().value()))
            .bind(revision.proposition())
            .bind(revision.endorsement().map(|value| i64::from(value.value())))
            .bind(revision.change_note())
            .bind("InitialUserEntry")
            .bind(created_at_ms)
            .execute(&mut *transaction)
            .await
        {
            let error = write_error("create_belief_with_initial_revision", error);
            transaction.rollback().await.map_err(PersistenceError::from)?;
            return Err(error);
        }

        transaction.commit().await.map_err(PersistenceError::from)?;
        Ok(revision)
    }

    pub(crate) async fn load_belief(
        &self,
        id: &crate::domain::BeliefId,
    ) -> Result<crate::domain::Belief, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let row: Option<(String, String, i64)> =
            sqlx::query_as("SELECT id, subject_id, created_at_ms FROM beliefs WHERE id = ?")
                .bind(id.as_str())
                .fetch_optional(&mut **connection.connection())
                .await
                .map_err(PersistenceError::from)?;
        let (id, subject_id, created_at_ms) = row.ok_or_else(|| PersistenceError::NotFound {
            entity: "Belief",
            id: id.as_str().to_owned(),
        })?;
        BeliefRow {
            id,
            subject_id,
            created_at_ms,
        }
        .try_into()
    }

    pub(crate) async fn append_belief_revision(
        &self,
        belief_id: &crate::domain::BeliefId,
        append: AppendBeliefRevisionInput,
        created_at_ms: i64,
    ) -> Result<crate::domain::BeliefRevision, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let mut transaction = (**connection.connection())
            .begin_with("BEGIN IMMEDIATE")
            .await
            .map_err(PersistenceError::from)?;
        let parent_exists: Option<String> =
            sqlx::query_scalar("SELECT id FROM beliefs WHERE id = ?")
                .bind(belief_id.as_str())
                .fetch_optional(&mut *transaction)
                .await
                .map_err(PersistenceError::from)?;
        if parent_exists.is_none() {
            transaction
                .rollback()
                .await
                .map_err(PersistenceError::from)?;
            return Err(PersistenceError::NotFound {
                entity: "Belief",
                id: belief_id.as_str().to_owned(),
            });
        }
        if append.origin == crate::domain::RevisionOrigin::InitialUserEntry {
            transaction
                .rollback()
                .await
                .map_err(PersistenceError::from)?;
            return Err(PersistenceError::ConstraintViolation {
                operation: "append_belief_revision",
                detail: "InitialUserEntry is valid only for revision 1".into(),
            });
        }

        let current: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(revision_number) FROM belief_revisions WHERE belief_id = ?",
        )
        .bind(belief_id.as_str())
        .fetch_one(&mut *transaction)
        .await
        .map_err(PersistenceError::from)?;
        let current = match current {
            Some(current) => current,
            None => {
                transaction
                    .rollback()
                    .await
                    .map_err(PersistenceError::from)?;
                return Err(PersistenceError::NotReady(
                    "Belief anchor has no initial revision".into(),
                ));
            }
        };
        let next = current
            .checked_add(1)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| PersistenceError::Storage("Belief revision sequence overflow".into()))?;
        let revision_number = crate::domain::RevisionNumber::new(next)
            .map_err(|error| invalid_write("append_belief_revision", error))?;
        let origin_name = match append.origin {
            crate::domain::RevisionOrigin::UserUpdate => "UserUpdate",
            crate::domain::RevisionOrigin::UserCorrection => "UserCorrection",
            crate::domain::RevisionOrigin::InitialUserEntry => unreachable!(),
        };
        let revision = match crate::domain::BeliefRevision::new(
            append.id,
            belief_id.clone(),
            revision_number,
            append.proposition,
            append.endorsement,
            append.change_note,
            append.origin,
        ) {
            Ok(revision) => revision,
            Err(error) => {
                transaction
                    .rollback()
                    .await
                    .map_err(PersistenceError::from)?;
                return Err(invalid_write("append_belief_revision", error));
            }
        };
        if let Err(error) = sqlx::query("INSERT INTO belief_revisions (id, belief_id, revision_number, proposition, endorsement, change_note, origin, created_at_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(revision.id().as_str())
            .bind(revision.belief_id().as_str())
            .bind(i64::from(revision.revision_number().value()))
            .bind(revision.proposition())
            .bind(revision.endorsement().map(|value| i64::from(value.value())))
            .bind(revision.change_note())
            .bind(origin_name)
            .bind(created_at_ms)
            .execute(&mut *transaction)
            .await
        {
            let error = write_error("append_belief_revision", error);
            transaction.rollback().await.map_err(PersistenceError::from)?;
            return Err(error);
        }
        transaction.commit().await.map_err(PersistenceError::from)?;
        Ok(revision)
    }

    pub(crate) async fn load_belief_history(
        &self,
        belief_id: &crate::domain::BeliefId,
    ) -> Result<Vec<crate::domain::BeliefRevision>, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let rows: Vec<BeliefRevisionDatabaseRow> = sqlx::query_as("SELECT id, belief_id, revision_number, proposition, endorsement, change_note, origin, created_at_ms FROM belief_revisions WHERE belief_id = ? ORDER BY revision_number ASC")
            .bind(belief_id.as_str())
            .fetch_all(&mut **connection.connection())
            .await
            .map_err(PersistenceError::from)?;
        rows.into_iter()
            .map(
                |(
                    id,
                    belief_id,
                    revision_number,
                    proposition,
                    endorsement,
                    change_note,
                    origin,
                    created_at_ms,
                )| {
                    BeliefRevisionRow {
                        id,
                        belief_id,
                        revision_number,
                        proposition,
                        endorsement,
                        change_note,
                        origin,
                        created_at_ms,
                    }
                    .try_into()
                },
            )
            .collect()
    }

    pub(crate) async fn create_value_with_initial_revision(
        &self,
        value: &crate::domain::Value,
        initial: InitialValueRevisionInput,
        created_at_ms: i64,
    ) -> Result<crate::domain::ValueRevision, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let mut transaction = (**connection.connection())
            .begin_with("BEGIN IMMEDIATE")
            .await
            .map_err(PersistenceError::from)?;
        let revision_number =
            crate::domain::RevisionNumber::new(1).expect("one is always a valid revision number");
        let revision = match crate::domain::ValueRevision::new(
            initial.id,
            value.id().clone(),
            revision_number,
            initial.label,
            initial.importance,
            initial.change_note,
            crate::domain::RevisionOrigin::InitialUserEntry,
        ) {
            Ok(revision) => revision,
            Err(error) => {
                transaction
                    .rollback()
                    .await
                    .map_err(PersistenceError::from)?;
                return Err(invalid_write("create_value_with_initial_revision", error));
            }
        };

        if let Err(error) =
            sqlx::query("INSERT INTO \"values\" (id, subject_id, created_at_ms) VALUES (?, ?, ?)")
                .bind(value.id().as_str())
                .bind(value.subject_id().as_str())
                .bind(created_at_ms)
                .execute(&mut *transaction)
                .await
        {
            let error = write_error("create_value_with_initial_revision", error);
            transaction
                .rollback()
                .await
                .map_err(PersistenceError::from)?;
            return Err(error);
        }

        if let Err(error) = sqlx::query("INSERT INTO value_revisions (id, value_id, revision_number, label, importance, change_note, origin, created_at_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(revision.id().as_str())
            .bind(revision.value_id().as_str())
            .bind(i64::from(revision.revision_number().value()))
            .bind(revision.label())
            .bind(revision.importance().map(|importance| i64::from(importance.value())))
            .bind(revision.change_note())
            .bind("InitialUserEntry")
            .bind(created_at_ms)
            .execute(&mut *transaction)
            .await
        {
            let error = write_error("create_value_with_initial_revision", error);
            transaction.rollback().await.map_err(PersistenceError::from)?;
            return Err(error);
        }

        transaction.commit().await.map_err(PersistenceError::from)?;
        Ok(revision)
    }

    pub(crate) async fn load_value(
        &self,
        id: &crate::domain::ValueId,
    ) -> Result<crate::domain::Value, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let row: Option<(String, String, i64)> =
            sqlx::query_as("SELECT id, subject_id, created_at_ms FROM \"values\" WHERE id = ?")
                .bind(id.as_str())
                .fetch_optional(&mut **connection.connection())
                .await
                .map_err(PersistenceError::from)?;
        let (id, subject_id, created_at_ms) = row.ok_or_else(|| PersistenceError::NotFound {
            entity: "Value",
            id: id.as_str().to_owned(),
        })?;
        ValueRow {
            id,
            subject_id,
            created_at_ms,
        }
        .try_into()
    }

    pub(crate) async fn append_value_revision(
        &self,
        value_id: &crate::domain::ValueId,
        append: AppendValueRevisionInput,
        created_at_ms: i64,
    ) -> Result<crate::domain::ValueRevision, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let mut transaction = (**connection.connection())
            .begin_with("BEGIN IMMEDIATE")
            .await
            .map_err(PersistenceError::from)?;
        let parent_exists: Option<String> =
            sqlx::query_scalar("SELECT id FROM \"values\" WHERE id = ?")
                .bind(value_id.as_str())
                .fetch_optional(&mut *transaction)
                .await
                .map_err(PersistenceError::from)?;
        if parent_exists.is_none() {
            transaction
                .rollback()
                .await
                .map_err(PersistenceError::from)?;
            return Err(PersistenceError::NotFound {
                entity: "Value",
                id: value_id.as_str().to_owned(),
            });
        }
        if append.origin == crate::domain::RevisionOrigin::InitialUserEntry {
            transaction
                .rollback()
                .await
                .map_err(PersistenceError::from)?;
            return Err(PersistenceError::ConstraintViolation {
                operation: "append_value_revision",
                detail: "InitialUserEntry is valid only for revision 1".into(),
            });
        }

        let current: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(revision_number) FROM value_revisions WHERE value_id = ?",
        )
        .bind(value_id.as_str())
        .fetch_one(&mut *transaction)
        .await
        .map_err(PersistenceError::from)?;
        let current = match current {
            Some(current) => current,
            None => {
                transaction
                    .rollback()
                    .await
                    .map_err(PersistenceError::from)?;
                return Err(PersistenceError::NotReady(
                    "Value anchor has no initial revision".into(),
                ));
            }
        };
        let next = current
            .checked_add(1)
            .and_then(|number| u32::try_from(number).ok())
            .ok_or_else(|| PersistenceError::Storage("Value revision sequence overflow".into()))?;
        let revision_number = crate::domain::RevisionNumber::new(next)
            .map_err(|error| invalid_write("append_value_revision", error))?;
        let origin_name = match append.origin {
            crate::domain::RevisionOrigin::UserUpdate => "UserUpdate",
            crate::domain::RevisionOrigin::UserCorrection => "UserCorrection",
            crate::domain::RevisionOrigin::InitialUserEntry => unreachable!(),
        };
        let revision = match crate::domain::ValueRevision::new(
            append.id,
            value_id.clone(),
            revision_number,
            append.label,
            append.importance,
            append.change_note,
            append.origin,
        ) {
            Ok(revision) => revision,
            Err(error) => {
                transaction
                    .rollback()
                    .await
                    .map_err(PersistenceError::from)?;
                return Err(invalid_write("append_value_revision", error));
            }
        };
        if let Err(error) = sqlx::query("INSERT INTO value_revisions (id, value_id, revision_number, label, importance, change_note, origin, created_at_ms) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(revision.id().as_str())
            .bind(revision.value_id().as_str())
            .bind(i64::from(revision.revision_number().value()))
            .bind(revision.label())
            .bind(revision.importance().map(|importance| i64::from(importance.value())))
            .bind(revision.change_note())
            .bind(origin_name)
            .bind(created_at_ms)
            .execute(&mut *transaction)
            .await
        {
            let error = write_error("append_value_revision", error);
            transaction.rollback().await.map_err(PersistenceError::from)?;
            return Err(error);
        }
        transaction.commit().await.map_err(PersistenceError::from)?;
        Ok(revision)
    }

    pub(crate) async fn load_value_history(
        &self,
        value_id: &crate::domain::ValueId,
    ) -> Result<Vec<crate::domain::ValueRevision>, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let rows: Vec<ValueRevisionDatabaseRow> = sqlx::query_as("SELECT id, value_id, revision_number, label, importance, change_note, origin, created_at_ms FROM value_revisions WHERE value_id = ? ORDER BY revision_number ASC")
            .bind(value_id.as_str())
            .fetch_all(&mut **connection.connection())
            .await
            .map_err(PersistenceError::from)?;
        rows.into_iter()
            .map(
                |(
                    id,
                    value_id,
                    revision_number,
                    label,
                    importance,
                    change_note,
                    origin,
                    created_at_ms,
                )| {
                    ValueRevisionRow {
                        id,
                        value_id,
                        revision_number,
                        label,
                        importance,
                        change_note,
                        origin,
                        created_at_ms,
                    }
                    .try_into()
                },
            )
            .collect()
    }
}

/// Persistence rows retain storage-only `created_at_ms` while reconstruction uses domain APIs.
#[derive(Clone, Debug)]
pub(crate) struct SelfSubjectRow {
    pub(crate) id: String,
    pub(crate) display_name: String,
    pub(crate) created_at_ms: i64,
}

impl TryFrom<SelfSubjectRow> for crate::domain::SelfSubject {
    type Error = PersistenceError;

    fn try_from(row: SelfSubjectRow) -> Result<Self, Self::Error> {
        let _ = row.created_at_ms;
        let id = reconstruct(
            "SelfSubject",
            "id",
            crate::domain::SelfSubjectId::new(row.id),
        )?;
        reconstruct(
            "SelfSubject",
            "display_name",
            Self::new(id, row.display_name),
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PersonReferenceRow {
    pub(crate) id: String,
    pub(crate) subject_id: String,
    pub(crate) display_name: String,
    pub(crate) relationship_label: String,
    pub(crate) context_notes: Option<String>,
    pub(crate) created_at_ms: i64,
}

impl TryFrom<PersonReferenceRow> for crate::domain::PersonReference {
    type Error = PersistenceError;

    fn try_from(row: PersonReferenceRow) -> Result<Self, Self::Error> {
        let _ = row.created_at_ms;
        let id = reconstruct(
            "PersonReference",
            "id",
            crate::domain::PersonReferenceId::new(row.id),
        )?;
        let subject_id = reconstruct(
            "PersonReference",
            "subject_id",
            crate::domain::SelfSubjectId::new(row.subject_id),
        )?;
        reconstruct(
            "PersonReference",
            "fields",
            Self::new(
                id,
                subject_id,
                row.display_name,
                row.relationship_label,
                row.context_notes,
            ),
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SituationRow {
    pub(crate) id: String,
    pub(crate) subject_id: String,
    pub(crate) description: String,
    pub(crate) created_at_ms: i64,
}

impl TryFrom<SituationRow> for crate::domain::Situation {
    type Error = PersistenceError;

    fn try_from(row: SituationRow) -> Result<Self, Self::Error> {
        let _ = row.created_at_ms;
        let id = reconstruct("Situation", "id", crate::domain::SituationId::new(row.id))?;
        let subject_id = reconstruct(
            "Situation",
            "subject_id",
            crate::domain::SelfSubjectId::new(row.subject_id),
        )?;
        reconstruct(
            "Situation",
            "description",
            Self::new(id, subject_id, row.description),
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ObservationRow {
    pub(crate) id: String,
    pub(crate) subject_id: String,
    pub(crate) situation_id: Option<String>,
    pub(crate) content: String,
    pub(crate) created_at_ms: i64,
}

impl TryFrom<ObservationRow> for crate::domain::Observation {
    type Error = PersistenceError;

    fn try_from(row: ObservationRow) -> Result<Self, Self::Error> {
        let _ = row.created_at_ms;
        let id = reconstruct(
            "Observation",
            "id",
            crate::domain::ObservationId::new(row.id),
        )?;
        let subject_id = reconstruct(
            "Observation",
            "subject_id",
            crate::domain::SelfSubjectId::new(row.subject_id),
        )?;
        let situation_id = row
            .situation_id
            .map(|id| {
                reconstruct(
                    "Observation",
                    "situation_id",
                    crate::domain::SituationId::new(id),
                )
            })
            .transpose()?;
        reconstruct(
            "Observation",
            "content",
            Self::new(id, subject_id, situation_id, row.content),
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ThoughtRow {
    pub(crate) id: String,
    pub(crate) subject_id: String,
    pub(crate) situation_id: Option<String>,
    pub(crate) content: String,
    pub(crate) confidence: Option<i64>,
    pub(crate) created_at_ms: i64,
}

impl TryFrom<ThoughtRow> for crate::domain::Thought {
    type Error = PersistenceError;

    fn try_from(row: ThoughtRow) -> Result<Self, Self::Error> {
        let _ = row.created_at_ms;
        let id = reconstruct("Thought", "id", crate::domain::ThoughtId::new(row.id))?;
        let subject_id = reconstruct(
            "Thought",
            "subject_id",
            crate::domain::SelfSubjectId::new(row.subject_id),
        )?;
        let situation_id = row
            .situation_id
            .map(|id| {
                reconstruct(
                    "Thought",
                    "situation_id",
                    crate::domain::SituationId::new(id),
                )
            })
            .transpose()?;
        let confidence = row
            .confidence
            .map(|value| {
                let value = percentage("Thought", "confidence", value)?;
                reconstruct(
                    "Thought",
                    "confidence",
                    crate::domain::ThoughtConfidence::new(value),
                )
            })
            .transpose()?;
        reconstruct(
            "Thought",
            "content",
            Self::new(id, subject_id, situation_id, row.content, confidence),
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EmotionRow {
    pub(crate) id: String,
    pub(crate) subject_id: String,
    pub(crate) situation_id: Option<String>,
    pub(crate) label: String,
    pub(crate) intensity: i64,
    pub(crate) created_at_ms: i64,
}

impl TryFrom<EmotionRow> for crate::domain::Emotion {
    type Error = PersistenceError;

    fn try_from(row: EmotionRow) -> Result<Self, Self::Error> {
        let _ = row.created_at_ms;
        let id = reconstruct("Emotion", "id", crate::domain::EmotionId::new(row.id))?;
        let subject_id = reconstruct(
            "Emotion",
            "subject_id",
            crate::domain::SelfSubjectId::new(row.subject_id),
        )?;
        let situation_id = row
            .situation_id
            .map(|id| {
                reconstruct(
                    "Emotion",
                    "situation_id",
                    crate::domain::SituationId::new(id),
                )
            })
            .transpose()?;
        let intensity = reconstruct(
            "Emotion",
            "intensity",
            crate::domain::EmotionIntensity::new(percentage(
                "Emotion",
                "intensity",
                row.intensity,
            )?),
        )?;
        reconstruct(
            "Emotion",
            "label",
            Self::new(id, subject_id, situation_id, row.label, intensity),
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct BeliefRevisionRow {
    pub(crate) id: String,
    pub(crate) belief_id: String,
    pub(crate) revision_number: i64,
    pub(crate) proposition: String,
    pub(crate) endorsement: Option<i64>,
    pub(crate) change_note: Option<String>,
    pub(crate) origin: String,
    pub(crate) created_at_ms: i64,
}

#[derive(Clone, Debug)]
pub(crate) struct BeliefRow {
    pub(crate) id: String,
    pub(crate) subject_id: String,
    pub(crate) created_at_ms: i64,
}

impl TryFrom<BeliefRow> for crate::domain::Belief {
    type Error = PersistenceError;

    fn try_from(row: BeliefRow) -> Result<Self, Self::Error> {
        let _ = row.created_at_ms;
        let id = reconstruct("Belief", "id", crate::domain::BeliefId::new(row.id))?;
        let subject_id = reconstruct(
            "Belief",
            "subject_id",
            crate::domain::SelfSubjectId::new(row.subject_id),
        )?;
        Ok(Self::new(id, subject_id))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ValueRevisionRow {
    pub(crate) id: String,
    pub(crate) value_id: String,
    pub(crate) revision_number: i64,
    pub(crate) label: String,
    pub(crate) importance: Option<i64>,
    pub(crate) change_note: Option<String>,
    pub(crate) origin: String,
    pub(crate) created_at_ms: i64,
}

#[derive(Clone, Debug)]
pub(crate) struct ValueRow {
    pub(crate) id: String,
    pub(crate) subject_id: String,
    pub(crate) created_at_ms: i64,
}

impl TryFrom<ValueRow> for crate::domain::Value {
    type Error = PersistenceError;

    fn try_from(row: ValueRow) -> Result<Self, Self::Error> {
        let _ = row.created_at_ms;
        let id = reconstruct("Value", "id", crate::domain::ValueId::new(row.id))?;
        let subject_id = reconstruct(
            "Value",
            "subject_id",
            crate::domain::SelfSubjectId::new(row.subject_id),
        )?;
        Ok(Self::new(id, subject_id))
    }
}

fn revision_origin(
    entity: &'static str,
    value: String,
) -> Result<crate::domain::RevisionOrigin, PersistenceError> {
    match value.as_str() {
        "InitialUserEntry" => Ok(crate::domain::RevisionOrigin::InitialUserEntry),
        "UserUpdate" => Ok(crate::domain::RevisionOrigin::UserUpdate),
        "UserCorrection" => Ok(crate::domain::RevisionOrigin::UserCorrection),
        _ => Err(PersistenceError::DomainReconstruction {
            entity,
            field: "origin",
            detail: format!("unknown revision origin: {value}"),
        }),
    }
}

fn validate_persisted_revision_origin(
    entity: &'static str,
    revision_number: u32,
    origin: crate::domain::RevisionOrigin,
) -> Result<(), PersistenceError> {
    let valid = match origin {
        crate::domain::RevisionOrigin::InitialUserEntry => revision_number == 1,
        crate::domain::RevisionOrigin::UserUpdate
        | crate::domain::RevisionOrigin::UserCorrection => revision_number > 1,
    };

    if valid {
        Ok(())
    } else {
        Err(PersistenceError::DomainReconstruction {
            entity,
            field: "origin",
            detail: format!(
                "revision {revision_number} is incompatible with persisted origin {origin:?}"
            ),
        })
    }
}

impl TryFrom<BeliefRevisionRow> for crate::domain::BeliefRevision {
    type Error = PersistenceError;

    fn try_from(row: BeliefRevisionRow) -> Result<Self, Self::Error> {
        let _ = row.created_at_ms;
        let id = reconstruct(
            "BeliefRevision",
            "id",
            crate::domain::BeliefRevisionId::new(row.id),
        )?;
        let belief_id = reconstruct(
            "BeliefRevision",
            "belief_id",
            crate::domain::BeliefId::new(row.belief_id),
        )?;
        let number = reconstruct(
            "BeliefRevision",
            "revision_number",
            crate::domain::RevisionNumber::new(revision_number(
                "BeliefRevision",
                row.revision_number,
            )?),
        )?;
        let endorsement = row
            .endorsement
            .map(|value| {
                reconstruct(
                    "BeliefRevision",
                    "endorsement",
                    crate::domain::BeliefEndorsement::new(percentage(
                        "BeliefRevision",
                        "endorsement",
                        value,
                    )?),
                )
            })
            .transpose()?;
        let origin = revision_origin("BeliefRevision", row.origin)?;
        validate_persisted_revision_origin("BeliefRevision", number.value(), origin)?;
        reconstruct(
            "BeliefRevision",
            "proposition",
            Self::new(
                id,
                belief_id,
                number,
                row.proposition,
                endorsement,
                row.change_note,
                origin,
            ),
        )
    }
}

impl TryFrom<ValueRevisionRow> for crate::domain::ValueRevision {
    type Error = PersistenceError;

    fn try_from(row: ValueRevisionRow) -> Result<Self, Self::Error> {
        let _ = row.created_at_ms;
        let id = reconstruct(
            "ValueRevision",
            "id",
            crate::domain::ValueRevisionId::new(row.id),
        )?;
        let value_id = reconstruct(
            "ValueRevision",
            "value_id",
            crate::domain::ValueId::new(row.value_id),
        )?;
        let number = reconstruct(
            "ValueRevision",
            "revision_number",
            crate::domain::RevisionNumber::new(revision_number(
                "ValueRevision",
                row.revision_number,
            )?),
        )?;
        let importance = row
            .importance
            .map(|value| {
                reconstruct(
                    "ValueRevision",
                    "importance",
                    crate::domain::ValueImportance::new(percentage(
                        "ValueRevision",
                        "importance",
                        value,
                    )?),
                )
            })
            .transpose()?;
        let origin = revision_origin("ValueRevision", row.origin)?;
        validate_persisted_revision_origin("ValueRevision", number.value(), origin)?;
        reconstruct(
            "ValueRevision",
            "label",
            Self::new(
                id,
                value_id,
                number,
                row.label,
                importance,
                row.change_note,
                origin,
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::PathBuf,
        str::FromStr,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

    static NEXT_TEMP_DATABASE: AtomicUsize = AtomicUsize::new(0);

    const MIGRATION_0001: &str = include_str!("../migrations/0001_initialize.sql");
    const MIGRATION_0002: &str = include_str!("../migrations/0002_create_self_model.sql");

    type ColumnExpectation = (&'static str, &'static str, i64, i64);
    type TableColumnExpectations = (&'static str, &'static [ColumnExpectation]);
    type ForeignKeyExpectation = (&'static str, &'static str, &'static str, &'static str);
    type TableForeignKeyExpectations = (&'static str, &'static [ForeignKeyExpectation]);

    struct TempDatabase {
        directory: PathBuf,
        pool: SqlitePool,
    }

    impl TempDatabase {
        async fn open() -> Self {
            let unique = NEXT_TEMP_DATABASE.fetch_add(1, Ordering::Relaxed);
            let directory = std::env::temp_dir().join(format!(
                "nous-task004-migration-{}-{unique}",
                std::process::id()
            ));
            fs::create_dir_all(&directory).expect("temporary database directory should be created");
            let database_path = directory.join("nous.db");
            let database_url = format!(
                "sqlite://{}",
                database_path.to_string_lossy().replace('\\', "/")
            );
            let options = SqliteConnectOptions::from_str(&database_url)
                .expect("temporary SQLite URL should be valid")
                .create_if_missing(true)
                .foreign_keys(true);
            let pool = SqlitePoolOptions::new()
                .max_connections(2)
                .connect_with(options)
                .await
                .expect("temporary SQLite database should open");
            Self { directory, pool }
        }

        async fn apply(&self, migration: &str) {
            sqlx::raw_sql(migration)
                .execute(&self.pool)
                .await
                .expect("checked-in migration SQL should execute");
        }

        async fn close(self) {
            self.pool.close().await;
            fs::remove_dir_all(self.directory)
                .expect("temporary database directory should be removed");
        }

        async fn reopen(self) -> Self {
            let directory = self.directory;
            self.pool.close().await;
            let database_path = directory.join("nous.db");
            let database_url = format!(
                "sqlite://{}",
                database_path.to_string_lossy().replace('\\', "/")
            );
            let options = SqliteConnectOptions::from_str(&database_url)
                .expect("existing temporary SQLite URL should be valid")
                .foreign_keys(true);
            let pool = SqlitePoolOptions::new()
                .max_connections(2)
                .connect_with(options)
                .await
                .expect("temporary SQLite database should reopen");
            Self { directory, pool }
        }
    }

    async fn migrated_database() -> TempDatabase {
        let database = TempDatabase::open().await;
        database.apply(MIGRATION_0001).await;
        database.apply(MIGRATION_0002).await;
        database
    }

    async fn assert_task004_tables_exist(pool: &SqlitePool) {
        let tables: Vec<String> =
            sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
                .fetch_all(pool)
                .await
                .unwrap();
        for table in [
            "self_subjects",
            "person_references",
            "situations",
            "observations",
            "thoughts",
            "emotions",
            "beliefs",
            "belief_revisions",
            "values",
            "value_revisions",
        ] {
            assert!(tables.iter().any(|found| found == table), "missing {table}");
        }
    }

    async fn table_columns(pool: &SqlitePool, table: &str) -> Vec<(String, String, i64, i64)> {
        sqlx::query_as("SELECT name, type, \"notnull\", pk FROM pragma_table_info(?) ORDER BY cid")
            .bind(table)
            .fetch_all(pool)
            .await
            .unwrap()
    }

    async fn foreign_keys(pool: &SqlitePool, table: &str) -> Vec<(String, String, String, String)> {
        let mut keys = sqlx::query_as(
            "SELECT \"table\", \"from\", \"to\", on_delete FROM pragma_foreign_key_list(?)",
        )
        .bind(table)
        .fetch_all(pool)
        .await
        .unwrap();
        keys.sort();
        keys
    }

    async fn unique_index_columns(pool: &SqlitePool, table: &str) -> Vec<Vec<String>> {
        let index_names: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM pragma_index_list(?) WHERE \"unique\" = 1 ORDER BY name",
        )
        .bind(table)
        .fetch_all(pool)
        .await
        .unwrap();
        let mut indexes = Vec::new();
        for index_name in index_names {
            let columns: Vec<String> =
                sqlx::query_scalar("SELECT name FROM pragma_index_info(?) ORDER BY seqno")
                    .bind(index_name)
                    .fetch_all(pool)
                    .await
                    .unwrap();
            indexes.push(columns);
        }
        indexes
    }

    fn repository(database: &TempDatabase) -> SqliteSelfModelRepository {
        SqliteSelfModelRepository::new(SharedSqlitePool {
            pool: database.pool.clone(),
        })
    }

    async fn create_test_subject(
        repository: &SqliteSelfModelRepository,
    ) -> crate::domain::SelfSubject {
        let subject = crate::domain::SelfSubject::new(
            crate::domain::SelfSubjectId::new("phase5-subject").unwrap(),
            "Phase 5 user",
        )
        .unwrap();
        repository.create_self_subject(&subject, 1).await.unwrap();
        subject
    }

    fn belief(id: &str, subject_id: &crate::domain::SelfSubjectId) -> crate::domain::Belief {
        crate::domain::Belief::new(
            crate::domain::BeliefId::new(id).unwrap(),
            subject_id.clone(),
        )
    }

    fn value(id: &str, subject_id: &crate::domain::SelfSubjectId) -> crate::domain::Value {
        crate::domain::Value::new(crate::domain::ValueId::new(id).unwrap(), subject_id.clone())
    }

    fn assert_domain_reconstruction<T>(result: Result<T, PersistenceError>) {
        assert!(matches!(
            result,
            Err(PersistenceError::DomainReconstruction { .. })
        ));
    }

    fn lazy_pool() -> SqlitePool {
        SqlitePool::connect_lazy("sqlite::memory:")
            .expect("the SQLx SQLite URL fixture should be valid")
    }

    #[test]
    fn resolves_the_exact_preloaded_database_as_the_direct_sqlx_pool_type() {
        tauri::async_runtime::block_on(async {
            let instances = DbInstances::default();
            {
                let mut databases = instances
                    .0
                    .try_write()
                    .expect("the database fixture should not be locked");
                databases.insert("sqlite:other.db".to_owned(), DbPool::Sqlite(lazy_pool()));
                databases.insert(DATABASE_URL.to_owned(), DbPool::Sqlite(lazy_pool()));
            }

            let database = SharedSqlitePool::from_instances(&instances)
                .expect("the exact preloaded database key should resolve");
            let pool: &SqlitePool = database.pool();

            assert!(!pool.is_closed());
        });
    }

    #[test]
    fn does_not_fall_back_to_another_database_entry() {
        tauri::async_runtime::block_on(async {
            let instances = DbInstances::default();
            instances
                .0
                .try_write()
                .expect("the database fixture should not be locked")
                .insert("sqlite:other.db".to_owned(), DbPool::Sqlite(lazy_pool()));

            assert!(matches!(
                SharedSqlitePool::from_instances(&instances),
                Err(SharedPoolError::DatabaseNotPreloaded(DATABASE_URL))
            ));
        });
    }

    #[test]
    fn checked_in_migrations_create_all_tables_and_update_schema_version() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            assert_task004_tables_exist(&database.pool).await;
            let version: String =
                sqlx::query_scalar("SELECT value FROM app_metadata WHERE key = 'schema_version'")
                    .fetch_one(&database.pool)
                    .await
                    .unwrap();
            assert_eq!(version, "2");
            database.close().await;
        });
    }

    #[test]
    fn checked_in_migrations_create_the_approved_columns_keys_and_indexes() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let expected_columns: &[TableColumnExpectations] = &[
                (
                    "self_subjects",
                    &[
                        ("id", "TEXT", 1, 1),
                        ("display_name", "TEXT", 1, 0),
                        ("created_at_ms", "INTEGER", 1, 0),
                    ],
                ),
                (
                    "person_references",
                    &[
                        ("id", "TEXT", 1, 1),
                        ("subject_id", "TEXT", 1, 0),
                        ("display_name", "TEXT", 1, 0),
                        ("relationship_label", "TEXT", 1, 0),
                        ("context_notes", "TEXT", 0, 0),
                        ("created_at_ms", "INTEGER", 1, 0),
                    ],
                ),
                (
                    "situations",
                    &[
                        ("id", "TEXT", 1, 1),
                        ("subject_id", "TEXT", 1, 0),
                        ("description", "TEXT", 1, 0),
                        ("created_at_ms", "INTEGER", 1, 0),
                    ],
                ),
                (
                    "observations",
                    &[
                        ("id", "TEXT", 1, 1),
                        ("subject_id", "TEXT", 1, 0),
                        ("situation_id", "TEXT", 0, 0),
                        ("content", "TEXT", 1, 0),
                        ("created_at_ms", "INTEGER", 1, 0),
                    ],
                ),
                (
                    "thoughts",
                    &[
                        ("id", "TEXT", 1, 1),
                        ("subject_id", "TEXT", 1, 0),
                        ("situation_id", "TEXT", 0, 0),
                        ("content", "TEXT", 1, 0),
                        ("confidence", "INTEGER", 0, 0),
                        ("created_at_ms", "INTEGER", 1, 0),
                    ],
                ),
                (
                    "emotions",
                    &[
                        ("id", "TEXT", 1, 1),
                        ("subject_id", "TEXT", 1, 0),
                        ("situation_id", "TEXT", 0, 0),
                        ("label", "TEXT", 1, 0),
                        ("intensity", "INTEGER", 1, 0),
                        ("created_at_ms", "INTEGER", 1, 0),
                    ],
                ),
                (
                    "beliefs",
                    &[
                        ("id", "TEXT", 1, 1),
                        ("subject_id", "TEXT", 1, 0),
                        ("created_at_ms", "INTEGER", 1, 0),
                    ],
                ),
                (
                    "belief_revisions",
                    &[
                        ("id", "TEXT", 1, 1),
                        ("belief_id", "TEXT", 1, 0),
                        ("revision_number", "INTEGER", 1, 0),
                        ("proposition", "TEXT", 1, 0),
                        ("endorsement", "INTEGER", 0, 0),
                        ("change_note", "TEXT", 0, 0),
                        ("origin", "TEXT", 1, 0),
                        ("created_at_ms", "INTEGER", 1, 0),
                    ],
                ),
                (
                    "values",
                    &[
                        ("id", "TEXT", 1, 1),
                        ("subject_id", "TEXT", 1, 0),
                        ("created_at_ms", "INTEGER", 1, 0),
                    ],
                ),
                (
                    "value_revisions",
                    &[
                        ("id", "TEXT", 1, 1),
                        ("value_id", "TEXT", 1, 0),
                        ("revision_number", "INTEGER", 1, 0),
                        ("label", "TEXT", 1, 0),
                        ("importance", "INTEGER", 0, 0),
                        ("change_note", "TEXT", 0, 0),
                        ("origin", "TEXT", 1, 0),
                        ("created_at_ms", "INTEGER", 1, 0),
                    ],
                ),
            ];
            for (table, expected) in expected_columns {
                let actual = table_columns(&database.pool, table).await;
                let expected = expected
                    .iter()
                    .map(|(name, kind, not_null, primary_key)| {
                        (
                            (*name).to_owned(),
                            (*kind).to_owned(),
                            *not_null,
                            *primary_key,
                        )
                    })
                    .collect::<Vec<_>>();
                assert_eq!(actual, expected, "column mismatch for {table}");
            }

            let expected_foreign_keys: &[TableForeignKeyExpectations] = &[
                (
                    "person_references",
                    &[("self_subjects", "subject_id", "id", "RESTRICT")],
                ),
                (
                    "situations",
                    &[("self_subjects", "subject_id", "id", "RESTRICT")],
                ),
                (
                    "observations",
                    &[
                        ("self_subjects", "subject_id", "id", "RESTRICT"),
                        ("situations", "situation_id", "id", "RESTRICT"),
                        ("situations", "subject_id", "subject_id", "RESTRICT"),
                    ],
                ),
                (
                    "thoughts",
                    &[
                        ("self_subjects", "subject_id", "id", "RESTRICT"),
                        ("situations", "situation_id", "id", "RESTRICT"),
                        ("situations", "subject_id", "subject_id", "RESTRICT"),
                    ],
                ),
                (
                    "emotions",
                    &[
                        ("self_subjects", "subject_id", "id", "RESTRICT"),
                        ("situations", "situation_id", "id", "RESTRICT"),
                        ("situations", "subject_id", "subject_id", "RESTRICT"),
                    ],
                ),
                (
                    "beliefs",
                    &[("self_subjects", "subject_id", "id", "RESTRICT")],
                ),
                (
                    "belief_revisions",
                    &[("beliefs", "belief_id", "id", "RESTRICT")],
                ),
                (
                    "values",
                    &[("self_subjects", "subject_id", "id", "RESTRICT")],
                ),
                (
                    "value_revisions",
                    &[("values", "value_id", "id", "RESTRICT")],
                ),
            ];
            for (table, expected) in expected_foreign_keys {
                let actual = foreign_keys(&database.pool, table).await;
                let mut expected = expected
                    .iter()
                    .map(|(parent, from, to, on_delete)| {
                        (
                            (*parent).to_owned(),
                            (*from).to_owned(),
                            (*to).to_owned(),
                            (*on_delete).to_owned(),
                        )
                    })
                    .collect::<Vec<_>>();
                expected.sort();
                assert_eq!(actual, expected, "foreign-key mismatch for {table}");
            }

            let named_indexes: Vec<String> = sqlx::query_scalar(
                "SELECT name FROM sqlite_master WHERE type = 'index' AND name NOT LIKE 'sqlite_autoindex_%' ORDER BY name",
            )
            .fetch_all(&database.pool)
            .await
            .unwrap();
            assert_eq!(
                named_indexes,
                [
                    "belief_revisions_belief_id_idx",
                    "beliefs_subject_id_idx",
                    "emotions_situation_subject_idx",
                    "emotions_subject_id_idx",
                    "observations_situation_subject_idx",
                    "observations_subject_id_idx",
                    "person_references_subject_id_idx",
                    "situations_subject_id_idx",
                    "thoughts_situation_subject_idx",
                    "thoughts_subject_id_idx",
                    "value_revisions_value_id_idx",
                    "values_subject_id_idx",
                ]
            );

            assert!(unique_index_columns(&database.pool, "situations")
                .await
                .contains(&vec!["id".into(), "subject_id".into()]));
            assert!(unique_index_columns(&database.pool, "belief_revisions")
                .await
                .contains(&vec!["belief_id".into(), "revision_number".into()]));
            assert!(unique_index_columns(&database.pool, "value_revisions")
                .await
                .contains(&vec!["value_id".into(), "revision_number".into()]));

            database.close().await;
        });
    }

    #[test]
    fn version_one_database_upgrades_through_the_real_second_migration() {
        tauri::async_runtime::block_on(async {
            let database = TempDatabase::open().await;
            database.apply(MIGRATION_0001).await;
            let version: String =
                sqlx::query_scalar("SELECT value FROM app_metadata WHERE key = 'schema_version'")
                    .fetch_one(&database.pool)
                    .await
                    .unwrap();
            assert_eq!(version, "1");
            database.apply(MIGRATION_0002).await;
            assert_task004_tables_exist(&database.pool).await;
            let version: String =
                sqlx::query_scalar("SELECT value FROM app_metadata WHERE key = 'schema_version'")
                    .fetch_one(&database.pool)
                    .await
                    .unwrap();
            assert_eq!(version, "2");
            database.close().await;
        });
    }

    #[test]
    fn real_second_migration_failure_is_surfaced_and_does_not_report_version_two() {
        tauri::async_runtime::block_on(async {
            let database = TempDatabase::open().await;
            database.apply(MIGRATION_0001).await;
            sqlx::query("CREATE TABLE self_subjects (id TEXT PRIMARY KEY)")
                .execute(&database.pool)
                .await
                .unwrap();

            let failure = sqlx::raw_sql(MIGRATION_0002).execute(&database.pool).await;
            assert!(failure.is_err());
            let version: String =
                sqlx::query_scalar("SELECT value FROM app_metadata WHERE key = 'schema_version'")
                    .fetch_one(&database.pool)
                    .await
                    .unwrap();
            assert_eq!(version, "1");

            database.close().await;
        });
    }

    #[test]
    fn migration_constraints_enforce_ownership_text_bounds_history_and_restrict_deletes() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            sqlx::query("INSERT INTO self_subjects VALUES ('subject-a', 'A', 1)")
                .execute(&database.pool)
                .await
                .unwrap();
            sqlx::query("INSERT INTO self_subjects VALUES ('subject-b', 'B', 1)")
                .execute(&database.pool)
                .await
                .unwrap();
            sqlx::query("INSERT INTO situations VALUES ('situation-a', 'subject-a', 'context', 1)")
                .execute(&database.pool)
                .await
                .unwrap();
            assert!(sqlx::query("INSERT INTO person_references VALUES ('missing-parent', 'missing', 'Name', 'friend', NULL, 1)")
                .execute(&database.pool).await.is_err());
            assert!(sqlx::query("INSERT INTO observations VALUES ('cross-subject', 'subject-b', 'situation-a', 'detail', 1)")
                .execute(&database.pool).await.is_err());
            assert!(
                sqlx::query("INSERT INTO self_subjects VALUES ('blank', '   ', 1)")
                    .execute(&database.pool)
                    .await
                    .is_err()
            );
            assert!(sqlx::query(
                "INSERT INTO emotions VALUES ('too-high', 'subject-a', NULL, 'fear', 101, 1)"
            )
            .execute(&database.pool)
            .await
            .is_err());
            assert!(sqlx::query(
                "INSERT INTO thoughts VALUES ('bad-thought', 'subject-a', NULL, ' ', -1, 1)"
            )
            .execute(&database.pool)
            .await
            .is_err());
            sqlx::query("INSERT INTO beliefs VALUES ('belief-a', 'subject-a', 1)")
                .execute(&database.pool)
                .await
                .unwrap();
            assert!(sqlx::query("INSERT INTO belief_revisions VALUES ('non-positive', 'belief-a', 0, 'p', NULL, NULL, 'UserUpdate', 1)")
                .execute(&database.pool).await.is_err());
            assert!(sqlx::query("INSERT INTO belief_revisions VALUES ('wrong-initial', 'belief-a', 1, 'p', NULL, NULL, 'UserUpdate', 1)")
                .execute(&database.pool).await.is_err());
            sqlx::query("INSERT INTO belief_revisions VALUES ('belief-r1', 'belief-a', 1, 'p', 0, NULL, 'InitialUserEntry', 1)")
                .execute(&database.pool).await.unwrap();
            assert!(sqlx::query("INSERT INTO belief_revisions VALUES ('invalid-origin', 'belief-a', 2, 'p', NULL, NULL, 'SystemInference', 1)")
                .execute(&database.pool).await.is_err());
            assert!(sqlx::query("INSERT INTO belief_revisions VALUES ('invalid-endorsement', 'belief-a', 2, 'p', 101, NULL, 'UserUpdate', 1)")
                .execute(&database.pool).await.is_err());
            assert!(sqlx::query("INSERT INTO belief_revisions VALUES ('duplicate', 'belief-a', 1, 'p', NULL, NULL, 'InitialUserEntry', 1)")
                .execute(&database.pool).await.is_err());
            assert!(sqlx::query("INSERT INTO belief_revisions VALUES ('wrong-later', 'belief-a', 2, 'p', NULL, NULL, 'InitialUserEntry', 1)")
                .execute(&database.pool).await.is_err());
            sqlx::query("INSERT INTO \"values\" VALUES ('value-a', 'subject-a', 1)")
                .execute(&database.pool)
                .await
                .unwrap();
            assert!(sqlx::query("INSERT INTO value_revisions VALUES ('wrong-value-initial', 'value-a', 1, 'learning', NULL, NULL, 'UserCorrection', 1)")
                .execute(&database.pool).await.is_err());
            sqlx::query("INSERT INTO value_revisions VALUES ('value-r1', 'value-a', 1, 'learning', 100, NULL, 'InitialUserEntry', 1)")
                .execute(&database.pool).await.unwrap();
            assert!(sqlx::query("INSERT INTO value_revisions VALUES ('wrong-value-later', 'value-a', 2, 'learning', NULL, NULL, 'InitialUserEntry', 1)")
                .execute(&database.pool).await.is_err());
            assert!(sqlx::query("INSERT INTO value_revisions VALUES ('invalid-importance', 'value-a', 2, 'learning', -1, NULL, 'UserUpdate', 1)")
                .execute(&database.pool).await.is_err());
            assert!(
                sqlx::query("DELETE FROM self_subjects WHERE id = 'subject-a'")
                    .execute(&database.pool)
                    .await
                    .is_err()
            );
            database.close().await;
        });
    }

    #[test]
    fn foreign_keys_are_checked_on_each_acquired_operation_connection() {
        tauri::async_runtime::block_on(async {
            let database = TempDatabase::open().await;
            let mut first = database.pool.acquire().await.unwrap();
            sqlx::query("PRAGMA foreign_keys = OFF")
                .execute(&mut *first)
                .await
                .unwrap();
            assert!(matches!(
                verify_foreign_keys(&mut first).await,
                Err(PersistenceError::NotReady(_))
            ));

            let shared = SharedSqlitePool {
                pool: database.pool.clone(),
            };
            let mut verified = shared.acquire_verified_connection().await.unwrap();
            let enabled: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
                .fetch_one(&mut **verified.connection())
                .await
                .unwrap();
            assert_eq!(enabled, 1);
            drop(first);
            drop(verified);
            database.close().await;
        });
    }

    #[test]
    fn task_two_entities_create_and_load_with_unicode_nullability_and_boundaries() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let repository = repository(&database);
            let subject = crate::domain::SelfSubject::new(
                crate::domain::SelfSubjectId::new("subject-李").unwrap(),
                "李 Ming",
            )
            .unwrap();
            repository.create_self_subject(&subject, 10).await.unwrap();
            assert_eq!(
                repository.load_self_subject(subject.id()).await.unwrap(),
                subject
            );

            let person = crate::domain::PersonReference::new(
                crate::domain::PersonReferenceId::new("person-朋友").unwrap(),
                subject.id().clone(),
                "小雨 Xiaoyu",
                "朋友 / friend",
                Some("大学时认识。We still talk weekly.".into()),
            )
            .unwrap();
            repository
                .create_person_reference(&person, 11)
                .await
                .unwrap();
            assert_eq!(
                repository.load_person_reference(person.id()).await.unwrap(),
                person
            );
            let person_without_notes = crate::domain::PersonReference::new(
                crate::domain::PersonReferenceId::new("person-no-notes").unwrap(),
                subject.id().clone(),
                "Alex",
                "friend",
                None,
            )
            .unwrap();
            repository
                .create_person_reference(&person_without_notes, 12)
                .await
                .unwrap();
            assert_eq!(
                repository
                    .load_person_reference(person_without_notes.id())
                    .await
                    .unwrap()
                    .context_notes(),
                None
            );

            let situation = crate::domain::Situation::new(
                crate::domain::SituationId::new("situation-考试").unwrap(),
                subject.id().clone(),
                "Preparing for an exam 考试",
            )
            .unwrap();
            repository.create_situation(&situation, 13).await.unwrap();
            assert_eq!(
                repository.load_situation(situation.id()).await.unwrap(),
                situation
            );

            let observation = crate::domain::Observation::new(
                crate::domain::ObservationId::new("observation-1").unwrap(),
                subject.id().clone(),
                Some(situation.id().clone()),
                "I completed 两章 today.",
            )
            .unwrap();
            repository
                .create_observation(&observation, 14)
                .await
                .unwrap();
            assert_eq!(
                repository.load_observation(observation.id()).await.unwrap(),
                observation
            );

            let thought_zero = crate::domain::Thought::new(
                crate::domain::ThoughtId::new("thought-zero").unwrap(),
                subject.id().clone(),
                Some(situation.id().clone()),
                "I may need another approach.",
                Some(crate::domain::ThoughtConfidence::new(0).unwrap()),
            )
            .unwrap();
            repository.create_thought(&thought_zero, 15).await.unwrap();
            assert_eq!(
                repository.load_thought(thought_zero.id()).await.unwrap(),
                thought_zero
            );

            let thought_hundred = crate::domain::Thought::new(
                crate::domain::ThoughtId::new("thought-hundred").unwrap(),
                subject.id().clone(),
                None,
                "学习 can continue tomorrow.",
                Some(crate::domain::ThoughtConfidence::new(100).unwrap()),
            )
            .unwrap();
            repository
                .create_thought(&thought_hundred, 16)
                .await
                .unwrap();
            assert_eq!(
                repository.load_thought(thought_hundred.id()).await.unwrap(),
                thought_hundred
            );

            let thought_unquantified = crate::domain::Thought::new(
                crate::domain::ThoughtId::new("thought-none").unwrap(),
                subject.id().clone(),
                None,
                "Confidence can remain unquantified.",
                None,
            )
            .unwrap();
            repository
                .create_thought(&thought_unquantified, 17)
                .await
                .unwrap();
            assert_eq!(
                repository
                    .load_thought(thought_unquantified.id())
                    .await
                    .unwrap(),
                thought_unquantified
            );

            for (id, label, intensity, situation_id) in [
                ("emotion-zero", "calm 平静", 0, None),
                (
                    "emotion-hundred",
                    "hope 希望",
                    100,
                    Some(situation.id().clone()),
                ),
            ] {
                let emotion = crate::domain::Emotion::new(
                    crate::domain::EmotionId::new(id).unwrap(),
                    subject.id().clone(),
                    situation_id,
                    label,
                    crate::domain::EmotionIntensity::new(intensity).unwrap(),
                )
                .unwrap();
                repository.create_emotion(&emotion, 18).await.unwrap();
                assert_eq!(
                    repository.load_emotion(emotion.id()).await.unwrap(),
                    emotion
                );
            }

            let observation_without_situation = crate::domain::Observation::new(
                crate::domain::ObservationId::new("observation-no-situation").unwrap(),
                subject.id().clone(),
                None,
                "A concrete detail without a Situation link.",
            )
            .unwrap();
            repository
                .create_observation(&observation_without_situation, 19)
                .await
                .unwrap();
            assert_eq!(
                repository
                    .load_observation(observation_without_situation.id())
                    .await
                    .unwrap()
                    .situation_id(),
                None
            );

            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn missing_loads_return_not_found_for_each_operation_family() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let repository = repository(&database);
            assert!(matches!(
                repository
                    .load_self_subject(&crate::domain::SelfSubjectId::new("missing").unwrap())
                    .await,
                Err(PersistenceError::NotFound {
                    entity: "SelfSubject",
                    ..
                })
            ));
            assert!(matches!(
                repository
                    .load_person_reference(
                        &crate::domain::PersonReferenceId::new("missing").unwrap()
                    )
                    .await,
                Err(PersistenceError::NotFound {
                    entity: "PersonReference",
                    ..
                })
            ));
            assert!(matches!(
                repository
                    .load_situation(&crate::domain::SituationId::new("missing").unwrap())
                    .await,
                Err(PersistenceError::NotFound {
                    entity: "Situation",
                    ..
                })
            ));
            assert!(matches!(
                repository
                    .load_observation(&crate::domain::ObservationId::new("missing").unwrap())
                    .await,
                Err(PersistenceError::NotFound {
                    entity: "Observation",
                    ..
                })
            ));
            assert!(matches!(
                repository
                    .load_thought(&crate::domain::ThoughtId::new("missing").unwrap())
                    .await,
                Err(PersistenceError::NotFound {
                    entity: "Thought",
                    ..
                })
            ));
            assert!(matches!(
                repository
                    .load_emotion(&crate::domain::EmotionId::new("missing").unwrap())
                    .await,
                Err(PersistenceError::NotFound {
                    entity: "Emotion",
                    ..
                })
            ));
            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn create_operations_translate_identity_and_ownership_constraints() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let repository = repository(&database);
            let subject_a = crate::domain::SelfSubject::new(
                crate::domain::SelfSubjectId::new("subject-a").unwrap(),
                "A",
            )
            .unwrap();
            let subject_b = crate::domain::SelfSubject::new(
                crate::domain::SelfSubjectId::new("subject-b").unwrap(),
                "B",
            )
            .unwrap();
            repository.create_self_subject(&subject_a, 1).await.unwrap();
            repository.create_self_subject(&subject_b, 1).await.unwrap();
            assert!(matches!(
                repository.create_self_subject(&subject_a, 2).await,
                Err(PersistenceError::ConstraintViolation {
                    operation: "create_self_subject",
                    ..
                })
            ));

            let missing_subject_person = crate::domain::PersonReference::new(
                crate::domain::PersonReferenceId::new("missing-subject-person").unwrap(),
                crate::domain::SelfSubjectId::new("missing-subject").unwrap(),
                "Context only",
                "friend",
                None,
            )
            .unwrap();
            assert!(matches!(
                repository
                    .create_person_reference(&missing_subject_person, 2)
                    .await,
                Err(PersistenceError::ConstraintViolation {
                    operation: "create_person_reference",
                    ..
                })
            ));

            let situation_a = crate::domain::Situation::new(
                crate::domain::SituationId::new("situation-a").unwrap(),
                subject_a.id().clone(),
                "Owned by A",
            )
            .unwrap();
            repository.create_situation(&situation_a, 3).await.unwrap();

            let cross_subject_observation = crate::domain::Observation::new(
                crate::domain::ObservationId::new("cross-observation").unwrap(),
                subject_b.id().clone(),
                Some(situation_a.id().clone()),
                "Cannot cross ownership.",
            )
            .unwrap();
            assert!(matches!(
                repository
                    .create_observation(&cross_subject_observation, 4)
                    .await,
                Err(PersistenceError::ConstraintViolation {
                    operation: "create_observation",
                    ..
                })
            ));
            let cross_subject_thought = crate::domain::Thought::new(
                crate::domain::ThoughtId::new("cross-thought").unwrap(),
                subject_b.id().clone(),
                Some(situation_a.id().clone()),
                "Cannot cross ownership.",
                None,
            )
            .unwrap();
            assert!(matches!(
                repository.create_thought(&cross_subject_thought, 4).await,
                Err(PersistenceError::ConstraintViolation {
                    operation: "create_thought",
                    ..
                })
            ));
            let cross_subject_emotion = crate::domain::Emotion::new(
                crate::domain::EmotionId::new("cross-emotion").unwrap(),
                subject_b.id().clone(),
                Some(situation_a.id().clone()),
                "uneasy",
                crate::domain::EmotionIntensity::new(50).unwrap(),
            )
            .unwrap();
            assert!(matches!(
                repository.create_emotion(&cross_subject_emotion, 4).await,
                Err(PersistenceError::ConstraintViolation {
                    operation: "create_emotion",
                    ..
                })
            ));

            let valid_same_subject = crate::domain::Observation::new(
                crate::domain::ObservationId::new("same-subject").unwrap(),
                subject_a.id().clone(),
                Some(situation_a.id().clone()),
                "The ownership matches.",
            )
            .unwrap();
            repository
                .create_observation(&valid_same_subject, 5)
                .await
                .unwrap();
            assert_eq!(
                repository
                    .load_observation(valid_same_subject.id())
                    .await
                    .unwrap(),
                valid_same_subject
            );

            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn belief_and_value_initial_creation_fixes_revision_one_and_round_trips_content() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let repository = repository(&database);
            let subject = create_test_subject(&repository).await;

            let cases = [
                ("belief-none", "学习很重要。", None, None),
                (
                    "belief-zero",
                    "Practice matters",
                    Some(crate::domain::BeliefEndorsement::new(0).unwrap()),
                    Some("Initial note".to_owned()),
                ),
                (
                    "belief-hundred",
                    "Learning 学习 matters 🌱",
                    Some(crate::domain::BeliefEndorsement::new(100).unwrap()),
                    None,
                ),
            ];
            for (index, (id, proposition, endorsement, change_note)) in
                cases.into_iter().enumerate()
            {
                let anchor = belief(id, subject.id());
                let revision = repository
                    .create_belief_with_initial_revision(
                        &anchor,
                        InitialBeliefRevisionInput::new(
                            crate::domain::BeliefRevisionId::new(format!("{id}-r1")).unwrap(),
                            proposition,
                            endorsement,
                            change_note.clone(),
                        ),
                        10 + i64::try_from(index).unwrap(),
                    )
                    .await
                    .unwrap();
                assert_eq!(repository.load_belief(anchor.id()).await.unwrap(), anchor);
                assert_eq!(revision.revision_number().value(), 1);
                assert_eq!(
                    revision.origin(),
                    crate::domain::RevisionOrigin::InitialUserEntry
                );
                assert_eq!(revision.proposition(), proposition);
                assert_eq!(revision.endorsement(), endorsement);
                assert_eq!(revision.change_note(), change_note.as_deref());
                assert_eq!(
                    repository.load_belief_history(anchor.id()).await.unwrap(),
                    vec![revision]
                );
            }

            let cases = [
                ("value-none", "学习", None, None),
                (
                    "value-zero",
                    "Autonomy",
                    Some(crate::domain::ValueImportance::new(0).unwrap()),
                    Some("Initial note".to_owned()),
                ),
                (
                    "value-hundred",
                    "Learning 学习 🌱",
                    Some(crate::domain::ValueImportance::new(100).unwrap()),
                    None,
                ),
            ];
            for (index, (id, label, importance, change_note)) in cases.into_iter().enumerate() {
                let anchor = value(id, subject.id());
                let revision = repository
                    .create_value_with_initial_revision(
                        &anchor,
                        InitialValueRevisionInput::new(
                            crate::domain::ValueRevisionId::new(format!("{id}-r1")).unwrap(),
                            label,
                            importance,
                            change_note.clone(),
                        ),
                        20 + i64::try_from(index).unwrap(),
                    )
                    .await
                    .unwrap();
                assert_eq!(repository.load_value(anchor.id()).await.unwrap(), anchor);
                assert_eq!(revision.revision_number().value(), 1);
                assert_eq!(
                    revision.origin(),
                    crate::domain::RevisionOrigin::InitialUserEntry
                );
                assert_eq!(revision.label(), label);
                assert_eq!(revision.importance(), importance);
                assert_eq!(revision.change_note(), change_note.as_deref());
                assert_eq!(
                    repository.load_value_history(anchor.id()).await.unwrap(),
                    vec![revision]
                );
            }

            assert!(matches!(
                repository
                    .load_belief(&crate::domain::BeliefId::new("missing-belief").unwrap())
                    .await,
                Err(PersistenceError::NotFound {
                    entity: "Belief",
                    ..
                })
            ));
            assert!(matches!(
                repository
                    .load_value(&crate::domain::ValueId::new("missing-value").unwrap())
                    .await,
                Err(PersistenceError::NotFound {
                    entity: "Value",
                    ..
                })
            ));

            // Initial inputs intentionally expose neither revision number nor origin. The
            // operation fixes both canonical values before either insert can commit.
            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn failed_initial_revision_insert_rolls_back_belief_and_value_anchors() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let repository = repository(&database);
            let subject = create_test_subject(&repository).await;
            sqlx::query("CREATE TRIGGER fail_belief_revision BEFORE INSERT ON belief_revisions WHEN NEW.id = 'forced-belief-failure' BEGIN SELECT RAISE(ABORT, 'forced belief revision failure'); END")
                .execute(&database.pool)
                .await
                .unwrap();
            sqlx::query("CREATE TRIGGER fail_value_revision BEFORE INSERT ON value_revisions WHEN NEW.id = 'forced-value-failure' BEGIN SELECT RAISE(ABORT, 'forced value revision failure'); END")
                .execute(&database.pool)
                .await
                .unwrap();

            let failed_belief = belief("rolled-back-belief", subject.id());
            assert!(repository
                .create_belief_with_initial_revision(
                    &failed_belief,
                    InitialBeliefRevisionInput::new(
                        crate::domain::BeliefRevisionId::new("forced-belief-failure").unwrap(),
                        "Must roll back",
                        None,
                        None,
                    ),
                    30,
                )
                .await
                .is_err());
            let belief_anchor_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM beliefs WHERE id = ?")
                    .bind(failed_belief.id().as_str())
                    .fetch_one(&database.pool)
                    .await
                    .unwrap();
            let belief_revision_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM belief_revisions WHERE belief_id = ?")
                    .bind(failed_belief.id().as_str())
                    .fetch_one(&database.pool)
                    .await
                    .unwrap();
            assert_eq!((belief_anchor_count, belief_revision_count), (0, 0));

            let failed_value = value("rolled-back-value", subject.id());
            assert!(repository
                .create_value_with_initial_revision(
                    &failed_value,
                    InitialValueRevisionInput::new(
                        crate::domain::ValueRevisionId::new("forced-value-failure").unwrap(),
                        "Must roll back",
                        None,
                        None,
                    ),
                    31,
                )
                .await
                .is_err());
            let value_anchor_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM \"values\" WHERE id = ?")
                    .bind(failed_value.id().as_str())
                    .fetch_one(&database.pool)
                    .await
                    .unwrap();
            let value_revision_count: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM value_revisions WHERE value_id = ?")
                    .bind(failed_value.id().as_str())
                    .fetch_one(&database.pool)
                    .await
                    .unwrap();
            assert_eq!((value_anchor_count, value_revision_count), (0, 0));

            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn append_generates_ordered_immutable_belief_and_value_history() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let repository = repository(&database);
            let subject = create_test_subject(&repository).await;

            let belief_anchor = belief("append-belief", subject.id());
            let belief_initial = repository
                .create_belief_with_initial_revision(
                    &belief_anchor,
                    InitialBeliefRevisionInput::new(
                        crate::domain::BeliefRevisionId::new("append-belief-r1").unwrap(),
                        "I can learn.",
                        Some(crate::domain::BeliefEndorsement::new(50).unwrap()),
                        None,
                    ),
                    40,
                )
                .await
                .unwrap();
            let belief_before = belief_initial.clone();
            let belief_update = repository
                .append_belief_revision(
                    belief_anchor.id(),
                    AppendBeliefRevisionInput::new(
                        crate::domain::BeliefRevisionId::new("append-belief-r2").unwrap(),
                        "I learn through practice.",
                        Some(crate::domain::BeliefEndorsement::new(75).unwrap()),
                        Some("My view changed.".into()),
                        crate::domain::RevisionOrigin::UserUpdate,
                    ),
                    41,
                )
                .await
                .unwrap();
            let belief_correction = repository
                .append_belief_revision(
                    belief_anchor.id(),
                    AppendBeliefRevisionInput::new(
                        crate::domain::BeliefRevisionId::new("append-belief-r3").unwrap(),
                        "I learn through deliberate practice.",
                        Some(crate::domain::BeliefEndorsement::new(80).unwrap()),
                        Some("Corrected the wording.".into()),
                        crate::domain::RevisionOrigin::UserCorrection,
                    ),
                    42,
                )
                .await
                .unwrap();
            assert_eq!(belief_update.revision_number().value(), 2);
            assert_eq!(belief_correction.revision_number().value(), 3);
            let belief_history = repository
                .load_belief_history(belief_anchor.id())
                .await
                .unwrap();
            assert_eq!(
                belief_history
                    .iter()
                    .map(|revision| revision.revision_number().value())
                    .collect::<Vec<_>>(),
                vec![1, 2, 3]
            );
            assert_eq!(belief_history[0], belief_before);
            assert_eq!(
                belief_history[1].origin(),
                crate::domain::RevisionOrigin::UserUpdate
            );
            assert_eq!(
                belief_history[2].origin(),
                crate::domain::RevisionOrigin::UserCorrection
            );
            assert!(matches!(
                repository
                    .append_belief_revision(
                        belief_anchor.id(),
                        AppendBeliefRevisionInput::new(
                            crate::domain::BeliefRevisionId::new("invalid-belief-initial").unwrap(),
                            "Invalid",
                            None,
                            None,
                            crate::domain::RevisionOrigin::InitialUserEntry,
                        ),
                        43,
                    )
                    .await,
                Err(PersistenceError::ConstraintViolation {
                    operation: "append_belief_revision",
                    ..
                })
            ));
            assert!(matches!(
                repository
                    .append_belief_revision(
                        &crate::domain::BeliefId::new("missing-belief").unwrap(),
                        AppendBeliefRevisionInput::new(
                            crate::domain::BeliefRevisionId::new("missing-belief-r2").unwrap(),
                            "Missing",
                            None,
                            None,
                            crate::domain::RevisionOrigin::UserUpdate,
                        ),
                        43,
                    )
                    .await,
                Err(PersistenceError::NotFound {
                    entity: "Belief",
                    ..
                })
            ));
            assert!(sqlx::query("INSERT INTO belief_revisions (id, belief_id, revision_number, proposition, endorsement, change_note, origin, created_at_ms) VALUES ('duplicate-belief-sequence', ?, 2, 'duplicate', NULL, NULL, 'UserUpdate', 44)")
                .bind(belief_anchor.id().as_str())
                .execute(&database.pool)
                .await
                .is_err());

            let value_anchor = value("append-value", subject.id());
            let value_initial = repository
                .create_value_with_initial_revision(
                    &value_anchor,
                    InitialValueRevisionInput::new(
                        crate::domain::ValueRevisionId::new("append-value-r1").unwrap(),
                        "Learning",
                        Some(crate::domain::ValueImportance::new(60).unwrap()),
                        None,
                    ),
                    45,
                )
                .await
                .unwrap();
            let value_before = value_initial.clone();
            let value_update = repository
                .append_value_revision(
                    value_anchor.id(),
                    AppendValueRevisionInput::new(
                        crate::domain::ValueRevisionId::new("append-value-r2").unwrap(),
                        "Learning with others",
                        Some(crate::domain::ValueImportance::new(70).unwrap()),
                        Some("Expanded over time.".into()),
                        crate::domain::RevisionOrigin::UserUpdate,
                    ),
                    46,
                )
                .await
                .unwrap();
            let value_correction = repository
                .append_value_revision(
                    value_anchor.id(),
                    AppendValueRevisionInput::new(
                        crate::domain::ValueRevisionId::new("append-value-r3").unwrap(),
                        "Collaborative learning",
                        Some(crate::domain::ValueImportance::new(70).unwrap()),
                        Some("Corrected the label.".into()),
                        crate::domain::RevisionOrigin::UserCorrection,
                    ),
                    47,
                )
                .await
                .unwrap();
            assert_eq!(value_update.revision_number().value(), 2);
            assert_eq!(value_correction.revision_number().value(), 3);
            let value_history = repository
                .load_value_history(value_anchor.id())
                .await
                .unwrap();
            assert_eq!(
                value_history
                    .iter()
                    .map(|revision| revision.revision_number().value())
                    .collect::<Vec<_>>(),
                vec![1, 2, 3]
            );
            assert_eq!(value_history[0], value_before);
            assert_eq!(
                value_history[1].origin(),
                crate::domain::RevisionOrigin::UserUpdate
            );
            assert_eq!(
                value_history[2].origin(),
                crate::domain::RevisionOrigin::UserCorrection
            );
            assert!(matches!(
                repository
                    .append_value_revision(
                        value_anchor.id(),
                        AppendValueRevisionInput::new(
                            crate::domain::ValueRevisionId::new("invalid-value-initial").unwrap(),
                            "Invalid",
                            None,
                            None,
                            crate::domain::RevisionOrigin::InitialUserEntry,
                        ),
                        48,
                    )
                    .await,
                Err(PersistenceError::ConstraintViolation {
                    operation: "append_value_revision",
                    ..
                })
            ));
            assert!(matches!(
                repository
                    .append_value_revision(
                        &crate::domain::ValueId::new("missing-value").unwrap(),
                        AppendValueRevisionInput::new(
                            crate::domain::ValueRevisionId::new("missing-value-r2").unwrap(),
                            "Missing",
                            None,
                            None,
                            crate::domain::RevisionOrigin::UserUpdate,
                        ),
                        48,
                    )
                    .await,
                Err(PersistenceError::NotFound {
                    entity: "Value",
                    ..
                })
            ));
            assert!(sqlx::query("INSERT INTO value_revisions (id, value_id, revision_number, label, importance, change_note, origin, created_at_ms) VALUES ('duplicate-value-sequence', ?, 2, 'duplicate', NULL, NULL, 'UserUpdate', 49)")
                .bind(value_anchor.id().as_str())
                .execute(&database.pool)
                .await
                .is_err());

            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn competing_appends_serialize_to_distinct_belief_and_value_revision_numbers() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let repository = repository(&database);
            let subject = create_test_subject(&repository).await;
            let belief_anchor = belief("concurrent-belief", subject.id());
            repository
                .create_belief_with_initial_revision(
                    &belief_anchor,
                    InitialBeliefRevisionInput::new(
                        crate::domain::BeliefRevisionId::new("concurrent-belief-r1").unwrap(),
                        "Initial",
                        None,
                        None,
                    ),
                    50,
                )
                .await
                .unwrap();
            let value_anchor = value("concurrent-value", subject.id());
            repository
                .create_value_with_initial_revision(
                    &value_anchor,
                    InitialValueRevisionInput::new(
                        crate::domain::ValueRevisionId::new("concurrent-value-r1").unwrap(),
                        "Initial",
                        None,
                        None,
                    ),
                    50,
                )
                .await
                .unwrap();

            let belief_id_a = belief_anchor.id().clone();
            let belief_id_b = belief_anchor.id().clone();
            let repository_a = repository.clone();
            let repository_b = repository.clone();
            let belief_task_a = tauri::async_runtime::spawn(async move {
                repository_a
                    .append_belief_revision(
                        &belief_id_a,
                        AppendBeliefRevisionInput::new(
                            crate::domain::BeliefRevisionId::new("concurrent-belief-a").unwrap(),
                            "Concurrent A",
                            None,
                            None,
                            crate::domain::RevisionOrigin::UserUpdate,
                        ),
                        51,
                    )
                    .await
            });
            let belief_task_b = tauri::async_runtime::spawn(async move {
                repository_b
                    .append_belief_revision(
                        &belief_id_b,
                        AppendBeliefRevisionInput::new(
                            crate::domain::BeliefRevisionId::new("concurrent-belief-b").unwrap(),
                            "Concurrent B",
                            None,
                            None,
                            crate::domain::RevisionOrigin::UserCorrection,
                        ),
                        52,
                    )
                    .await
            });
            let mut belief_numbers = vec![
                belief_task_a
                    .await
                    .unwrap()
                    .unwrap()
                    .revision_number()
                    .value(),
                belief_task_b
                    .await
                    .unwrap()
                    .unwrap()
                    .revision_number()
                    .value(),
            ];
            belief_numbers.sort_unstable();
            assert_eq!(belief_numbers, vec![2, 3]);

            let value_id_a = value_anchor.id().clone();
            let value_id_b = value_anchor.id().clone();
            let repository_a = repository.clone();
            let repository_b = repository.clone();
            let value_task_a = tauri::async_runtime::spawn(async move {
                repository_a
                    .append_value_revision(
                        &value_id_a,
                        AppendValueRevisionInput::new(
                            crate::domain::ValueRevisionId::new("concurrent-value-a").unwrap(),
                            "Concurrent A",
                            None,
                            None,
                            crate::domain::RevisionOrigin::UserUpdate,
                        ),
                        53,
                    )
                    .await
            });
            let value_task_b = tauri::async_runtime::spawn(async move {
                repository_b
                    .append_value_revision(
                        &value_id_b,
                        AppendValueRevisionInput::new(
                            crate::domain::ValueRevisionId::new("concurrent-value-b").unwrap(),
                            "Concurrent B",
                            None,
                            None,
                            crate::domain::RevisionOrigin::UserCorrection,
                        ),
                        54,
                    )
                    .await
            });
            let mut value_numbers = vec![
                value_task_a
                    .await
                    .unwrap()
                    .unwrap()
                    .revision_number()
                    .value(),
                value_task_b
                    .await
                    .unwrap()
                    .unwrap()
                    .revision_number()
                    .value(),
            ];
            value_numbers.sort_unstable();
            assert_eq!(value_numbers, vec![2, 3]);

            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn begin_immediate_lock_contention_surfaces_as_storage_without_retry() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let repository = repository(&database);
            let subject = create_test_subject(&repository).await;
            let belief_anchor = belief("busy-belief", subject.id());
            repository
                .create_belief_with_initial_revision(
                    &belief_anchor,
                    InitialBeliefRevisionInput::new(
                        crate::domain::BeliefRevisionId::new("busy-belief-r1").unwrap(),
                        "Initial",
                        None,
                        None,
                    ),
                    60,
                )
                .await
                .unwrap();

            let mut blocker = repository
                .database
                .acquire_verified_connection()
                .await
                .unwrap();
            let blocking_transaction = (**blocker.connection())
                .begin_with("BEGIN IMMEDIATE")
                .await
                .unwrap();
            let mut available_connection = database.pool.acquire().await.unwrap();
            sqlx::query("PRAGMA busy_timeout = 0")
                .execute(&mut *available_connection)
                .await
                .unwrap();
            drop(available_connection);

            let error = repository
                .append_belief_revision(
                    belief_anchor.id(),
                    AppendBeliefRevisionInput::new(
                        crate::domain::BeliefRevisionId::new("busy-belief-r2").unwrap(),
                        "Contended",
                        None,
                        None,
                        crate::domain::RevisionOrigin::UserUpdate,
                    ),
                    61,
                )
                .await
                .unwrap_err();
            assert!(matches!(error, PersistenceError::Storage(_)));
            blocking_transaction.rollback().await.unwrap();
            drop(blocker);
            let history = repository
                .load_belief_history(belief_anchor.id())
                .await
                .unwrap();
            assert_eq!(history.len(), 1);

            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn belief_and_value_history_survive_close_and_reopen() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let repository = repository(&database);
            let subject = create_test_subject(&repository).await;
            let situation = crate::domain::Situation::new(
                crate::domain::SituationId::new("durable-situation").unwrap(),
                subject.id().clone(),
                "A durable Task 002 context",
            )
            .unwrap();
            repository.create_situation(&situation, 69).await.unwrap();
            let thought = crate::domain::Thought::new(
                crate::domain::ThoughtId::new("durable-thought").unwrap(),
                subject.id().clone(),
                Some(situation.id().clone()),
                "This thought persists 持久。",
                Some(crate::domain::ThoughtConfidence::new(100).unwrap()),
            )
            .unwrap();
            repository.create_thought(&thought, 69).await.unwrap();
            let belief_anchor = belief("durable-belief", subject.id());
            repository
                .create_belief_with_initial_revision(
                    &belief_anchor,
                    InitialBeliefRevisionInput::new(
                        crate::domain::BeliefRevisionId::new("durable-belief-r1").unwrap(),
                        "持久的 belief",
                        None,
                        None,
                    ),
                    70,
                )
                .await
                .unwrap();
            repository
                .append_belief_revision(
                    belief_anchor.id(),
                    AppendBeliefRevisionInput::new(
                        crate::domain::BeliefRevisionId::new("durable-belief-r2").unwrap(),
                        "持久的 updated belief",
                        None,
                        None,
                        crate::domain::RevisionOrigin::UserUpdate,
                    ),
                    71,
                )
                .await
                .unwrap();
            let value_anchor = value("durable-value", subject.id());
            repository
                .create_value_with_initial_revision(
                    &value_anchor,
                    InitialValueRevisionInput::new(
                        crate::domain::ValueRevisionId::new("durable-value-r1").unwrap(),
                        "持久的 value",
                        None,
                        None,
                    ),
                    72,
                )
                .await
                .unwrap();
            repository
                .append_value_revision(
                    value_anchor.id(),
                    AppendValueRevisionInput::new(
                        crate::domain::ValueRevisionId::new("durable-value-r2").unwrap(),
                        "持久的 corrected value",
                        None,
                        None,
                        crate::domain::RevisionOrigin::UserCorrection,
                    ),
                    73,
                )
                .await
                .unwrap();
            drop(repository);

            let database = database.reopen().await;
            let reopened_repository = SqliteSelfModelRepository::new(SharedSqlitePool {
                pool: database.pool.clone(),
            });
            assert_eq!(
                reopened_repository
                    .load_self_subject(subject.id())
                    .await
                    .unwrap(),
                subject
            );
            assert_eq!(
                reopened_repository
                    .load_situation(situation.id())
                    .await
                    .unwrap(),
                situation
            );
            assert_eq!(
                reopened_repository
                    .load_thought(thought.id())
                    .await
                    .unwrap(),
                thought
            );
            assert_eq!(
                reopened_repository
                    .load_belief(belief_anchor.id())
                    .await
                    .unwrap(),
                belief_anchor
            );
            assert_eq!(
                reopened_repository
                    .load_value(value_anchor.id())
                    .await
                    .unwrap(),
                value_anchor
            );
            assert_eq!(
                reopened_repository
                    .load_belief_history(belief_anchor.id())
                    .await
                    .unwrap()
                    .iter()
                    .map(|revision| revision.revision_number().value())
                    .collect::<Vec<_>>(),
                vec![1, 2]
            );
            assert_eq!(
                reopened_repository
                    .load_value_history(value_anchor.id())
                    .await
                    .unwrap()
                    .iter()
                    .map(|revision| revision.revision_number().value())
                    .collect::<Vec<_>>(),
                vec![1, 2]
            );

            drop(reopened_repository);
            database.close().await;
        });
    }

    #[test]
    fn rows_reconstruct_validated_unicode_nullable_and_boundary_domain_values() {
        let subject = crate::domain::SelfSubject::try_from(SelfSubjectRow {
            id: "self-李".into(),
            display_name: "李 Ming".into(),
            created_at_ms: 1,
        })
        .unwrap();
        assert_eq!(subject.display_name(), "李 Ming");
        let thought = crate::domain::Thought::try_from(ThoughtRow {
            id: "thought-混合".into(),
            subject_id: "self-李".into(),
            situation_id: None,
            content: "I can continue 一步 at a time.".into(),
            confidence: Some(0),
            created_at_ms: 1,
        })
        .unwrap();
        assert_eq!(thought.situation_id(), None);
        assert_eq!(thought.confidence().unwrap().value(), 0);
        let emotion = crate::domain::Emotion::try_from(EmotionRow {
            id: "emotion-1".into(),
            subject_id: "self-李".into(),
            situation_id: None,
            label: "希望 hope".into(),
            intensity: 100,
            created_at_ms: 1,
        })
        .unwrap();
        assert_eq!(emotion.intensity().value(), 100);
        let revision = crate::domain::BeliefRevision::try_from(BeliefRevisionRow {
            id: "br-1".into(),
            belief_id: "belief-1".into(),
            revision_number: 1,
            proposition: "学习 matters 学习".into(),
            endorsement: None,
            change_note: None,
            origin: "InitialUserEntry".into(),
            created_at_ms: 1,
        })
        .unwrap();
        assert_eq!(revision.endorsement(), None);
        let value = crate::domain::ValueRevision::try_from(ValueRevisionRow {
            id: "vr-1".into(),
            value_id: "value-1".into(),
            revision_number: 1,
            label: "Autonomy 自主".into(),
            importance: Some(100),
            change_note: None,
            origin: "InitialUserEntry".into(),
            created_at_ms: 1,
        })
        .unwrap();
        assert_eq!(value.importance().unwrap().value(), 100);
    }

    #[test]
    fn repository_loads_surface_controlled_corrupt_rows_instead_of_repairing_or_skipping() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let repository = repository(&database);
            let subject = create_test_subject(&repository).await;
            let mut connection = database.pool.acquire().await.unwrap();
            sqlx::query("PRAGMA ignore_check_constraints = ON")
                .execute(&mut *connection)
                .await
                .unwrap();
            sqlx::query("INSERT INTO person_references VALUES ('corrupt-person', ?, 'Name', 'friend', ' ', 1)")
                .bind(subject.id().as_str())
                .execute(&mut *connection)
                .await
                .unwrap();
            sqlx::query("INSERT INTO situations VALUES ('corrupt-situation', ?, ' ', 1)")
                .bind(subject.id().as_str())
                .execute(&mut *connection)
                .await
                .unwrap();
            sqlx::query("INSERT INTO observations VALUES ('corrupt-observation', ?, NULL, ' ', 1)")
                .bind(subject.id().as_str())
                .execute(&mut *connection)
                .await
                .unwrap();
            sqlx::query(
                "INSERT INTO thoughts VALUES ('corrupt-thought', ?, NULL, 'Content', 101, 1)",
            )
            .bind(subject.id().as_str())
            .execute(&mut *connection)
            .await
            .unwrap();
            sqlx::query("INSERT INTO emotions VALUES ('corrupt-emotion', ?, NULL, 'calm', -1, 1)")
                .bind(subject.id().as_str())
                .execute(&mut *connection)
                .await
                .unwrap();
            sqlx::query("INSERT INTO beliefs VALUES ('corrupt-belief', ?, 1)")
                .bind(subject.id().as_str())
                .execute(&mut *connection)
                .await
                .unwrap();
            sqlx::query("INSERT INTO belief_revisions VALUES ('corrupt-belief-r1', 'corrupt-belief', 1, 'Proposition', NULL, NULL, 'UserUpdate', 1)")
                .execute(&mut *connection)
                .await
                .unwrap();
            sqlx::query("INSERT INTO \"values\" VALUES ('corrupt-value', ?, 1)")
                .bind(subject.id().as_str())
                .execute(&mut *connection)
                .await
                .unwrap();
            sqlx::query("INSERT INTO value_revisions VALUES ('corrupt-value-r1', 'corrupt-value', 1, 'Value', NULL, NULL, 'SystemInference', 1)")
                .execute(&mut *connection)
                .await
                .unwrap();
            drop(connection);

            assert_domain_reconstruction(
                repository
                    .load_person_reference(
                        &crate::domain::PersonReferenceId::new("corrupt-person").unwrap(),
                    )
                    .await,
            );
            assert_domain_reconstruction(
                repository
                    .load_situation(&crate::domain::SituationId::new("corrupt-situation").unwrap())
                    .await,
            );
            assert_domain_reconstruction(
                repository
                    .load_observation(
                        &crate::domain::ObservationId::new("corrupt-observation").unwrap(),
                    )
                    .await,
            );
            assert_domain_reconstruction(
                repository
                    .load_thought(&crate::domain::ThoughtId::new("corrupt-thought").unwrap())
                    .await,
            );
            assert_domain_reconstruction(
                repository
                    .load_emotion(&crate::domain::EmotionId::new("corrupt-emotion").unwrap())
                    .await,
            );
            assert_domain_reconstruction(
                repository
                    .load_belief_history(&crate::domain::BeliefId::new("corrupt-belief").unwrap())
                    .await,
            );
            assert_domain_reconstruction(
                repository
                    .load_value_history(&crate::domain::ValueId::new("corrupt-value").unwrap())
                    .await,
            );

            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn corrupt_rows_are_not_repaired_or_converted_to_none() {
        let invalid_id = SelfSubjectRow {
            id: " ".into(),
            display_name: "name".into(),
            created_at_ms: 1,
        };
        assert!(matches!(
            crate::domain::SelfSubject::try_from(invalid_id),
            Err(PersistenceError::DomainReconstruction { .. })
        ));
        assert_domain_reconstruction(crate::domain::PersonReference::try_from(
            PersonReferenceRow {
                id: "person".into(),
                subject_id: "subject".into(),
                display_name: "Name".into(),
                relationship_label: "friend".into(),
                context_notes: Some("  ".into()),
                created_at_ms: 1,
            },
        ));
        assert_domain_reconstruction(crate::domain::Situation::try_from(SituationRow {
            id: "situation".into(),
            subject_id: "subject".into(),
            description: "\t".into(),
            created_at_ms: 1,
        }));
        assert_domain_reconstruction(crate::domain::Observation::try_from(ObservationRow {
            id: "observation".into(),
            subject_id: "subject".into(),
            situation_id: Some(" ".into()),
            content: "Concrete detail".into(),
            created_at_ms: 1,
        }));
        let invalid_text = ThoughtRow {
            id: "t".into(),
            subject_id: "s".into(),
            situation_id: None,
            content: "\t".into(),
            confidence: None,
            created_at_ms: 1,
        };
        assert!(matches!(
            crate::domain::Thought::try_from(invalid_text),
            Err(PersistenceError::DomainReconstruction { .. })
        ));
        assert_domain_reconstruction(crate::domain::Thought::try_from(ThoughtRow {
            id: "thought-percentage".into(),
            subject_id: "subject".into(),
            situation_id: None,
            content: "Content".into(),
            confidence: Some(-1),
            created_at_ms: 1,
        }));
        let invalid_percentage = EmotionRow {
            id: "e".into(),
            subject_id: "s".into(),
            situation_id: None,
            label: "fear".into(),
            intensity: 101,
            created_at_ms: 1,
        };
        assert!(matches!(
            crate::domain::Emotion::try_from(invalid_percentage),
            Err(PersistenceError::DomainReconstruction { .. })
        ));
        assert_domain_reconstruction(crate::domain::Belief::try_from(BeliefRow {
            id: " ".into(),
            subject_id: "subject".into(),
            created_at_ms: 1,
        }));
        assert_domain_reconstruction(crate::domain::Value::try_from(ValueRow {
            id: "value".into(),
            subject_id: "\t".into(),
            created_at_ms: 1,
        }));
        let invalid_number = BeliefRevisionRow {
            id: "br".into(),
            belief_id: "b".into(),
            revision_number: 0,
            proposition: "p".into(),
            endorsement: None,
            change_note: None,
            origin: "InitialUserEntry".into(),
            created_at_ms: 1,
        };
        assert!(matches!(
            crate::domain::BeliefRevision::try_from(invalid_number),
            Err(PersistenceError::DomainReconstruction { .. })
        ));
        assert_domain_reconstruction(crate::domain::BeliefRevision::try_from(BeliefRevisionRow {
            id: "br-percentage".into(),
            belief_id: "belief".into(),
            revision_number: 2,
            proposition: "Proposition".into(),
            endorsement: Some(101),
            change_note: None,
            origin: "UserUpdate".into(),
            created_at_ms: 1,
        }));
        assert_domain_reconstruction(crate::domain::BeliefRevision::try_from(BeliefRevisionRow {
            id: "br-wrong-origin".into(),
            belief_id: "belief".into(),
            revision_number: 1,
            proposition: "Proposition".into(),
            endorsement: None,
            change_note: None,
            origin: "UserUpdate".into(),
            created_at_ms: 1,
        }));
        let invalid_origin = ValueRevisionRow {
            id: "vr".into(),
            value_id: "v".into(),
            revision_number: 1,
            label: "value".into(),
            importance: None,
            change_note: None,
            origin: "SystemInference".into(),
            created_at_ms: 1,
        };
        assert!(matches!(
            crate::domain::ValueRevision::try_from(invalid_origin),
            Err(PersistenceError::DomainReconstruction { .. })
        ));
        assert_domain_reconstruction(crate::domain::ValueRevision::try_from(ValueRevisionRow {
            id: "vr-percentage".into(),
            value_id: "value".into(),
            revision_number: 2,
            label: "Value".into(),
            importance: Some(-1),
            change_note: None,
            origin: "UserCorrection".into(),
            created_at_ms: 1,
        }));
        assert_domain_reconstruction(crate::domain::ValueRevision::try_from(ValueRevisionRow {
            id: "vr-wrong-origin".into(),
            value_id: "value".into(),
            revision_number: 2,
            label: "Value".into(),
            importance: None,
            change_note: None,
            origin: "InitialUserEntry".into(),
            created_at_ms: 1,
        }));
    }
}
