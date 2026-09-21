use std::{error::Error, fmt};

use sqlx::{
    error::ErrorKind, pool::PoolConnection, sqlite::SqliteRow, Connection, Row, Sqlite, SqlitePool,
};
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
    SubjectInvariant(String),
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
            Self::SubjectInvariant(detail) => {
                write!(formatter, "self-subject invariant failure: {detail}")
            }
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
type MemoryDatabaseRow = (String, String, Option<String>, String, Option<String>, i64);
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

    pub(crate) async fn load_single_self_subject_for_capture(
        &self,
    ) -> Result<Option<crate::domain::SelfSubject>, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let rows: Vec<(String, String, i64)> = sqlx::query_as(
            "SELECT id, display_name, created_at_ms FROM self_subjects ORDER BY created_at_ms, id LIMIT 2",
        )
        .fetch_all(&mut **connection.connection())
        .await
        .map_err(PersistenceError::from)?;

        match rows.as_slice() {
            [] => Ok(None),
            [(id, display_name, created_at_ms)] => SelfSubjectRow {
                id: id.clone(),
                display_name: display_name.clone(),
                created_at_ms: *created_at_ms,
            }
            .try_into()
            .map(Some),
            _ => Err(PersistenceError::SubjectInvariant(
                "more than one SelfSubject exists".into(),
            )),
        }
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

    pub(crate) async fn create_structured_capture_atomic(
        &self,
        expected_subject_id: &crate::domain::SelfSubjectId,
        bootstrap_subject: Option<&crate::domain::SelfSubject>,
        situation: Option<&crate::domain::Situation>,
        observations: &[crate::domain::Observation],
        thoughts: &[crate::domain::Thought],
        created_at_ms: i64,
    ) -> Result<(), PersistenceError> {
        const OPERATION: &str = "create_structured_capture_atomic";

        if situation.is_none() && observations.is_empty() && thoughts.is_empty() {
            return Err(PersistenceError::ConstraintViolation {
                operation: OPERATION,
                detail: "a structured capture must contain at least one record".into(),
            });
        }
        if bootstrap_subject.is_some_and(|subject| subject.id() != expected_subject_id) {
            return Err(PersistenceError::SubjectInvariant(
                "bootstrap SelfSubject does not match the expected current subject".into(),
            ));
        }
        if situation.is_some_and(|value| value.subject_id() != expected_subject_id)
            || observations
                .iter()
                .any(|value| value.subject_id() != expected_subject_id)
            || thoughts
                .iter()
                .any(|value| value.subject_id() != expected_subject_id)
        {
            return Err(PersistenceError::SubjectInvariant(
                "capture records do not all belong to the current SelfSubject".into(),
            ));
        }

        let expected_situation_id = situation.map(crate::domain::Situation::id);
        if observations
            .iter()
            .any(|value| value.situation_id() != expected_situation_id)
            || thoughts
                .iter()
                .any(|value| value.situation_id() != expected_situation_id)
        {
            return Err(PersistenceError::ConstraintViolation {
                operation: OPERATION,
                detail: "capture records have an invalid Situation relationship".into(),
            });
        }

        let mut connection = self.database.acquire_verified_connection().await?;
        let mut transaction = (**connection.connection())
            .begin_with("BEGIN IMMEDIATE")
            .await
            .map_err(PersistenceError::from)?;

        let subject_rows: Vec<(String,)> =
            match sqlx::query_as("SELECT id FROM self_subjects ORDER BY created_at_ms, id LIMIT 2")
                .fetch_all(&mut *transaction)
                .await
            {
                Ok(rows) => rows,
                Err(error) => {
                    let error = PersistenceError::from(error);
                    transaction
                        .rollback()
                        .await
                        .map_err(PersistenceError::from)?;
                    return Err(error);
                }
            };

        match subject_rows.as_slice() {
            [] => {
                let Some(subject) = bootstrap_subject else {
                    transaction
                        .rollback()
                        .await
                        .map_err(PersistenceError::from)?;
                    return Err(PersistenceError::SubjectInvariant(
                        "no current SelfSubject exists and no bootstrap subject was supplied"
                            .into(),
                    ));
                };
                if let Err(error) = sqlx::query(
                    "INSERT INTO self_subjects (id, display_name, created_at_ms) VALUES (?, ?, ?)",
                )
                .bind(subject.id().as_str())
                .bind(subject.display_name())
                .bind(created_at_ms)
                .execute(&mut *transaction)
                .await
                {
                    let error = write_error(OPERATION, error);
                    transaction
                        .rollback()
                        .await
                        .map_err(PersistenceError::from)?;
                    return Err(error);
                }
            }
            [(subject_id,)] if subject_id == expected_subject_id.as_str() => {}
            [(_subject_id,)] => {
                transaction
                    .rollback()
                    .await
                    .map_err(PersistenceError::from)?;
                return Err(PersistenceError::SubjectInvariant(
                    "the current SelfSubject changed before capture persistence".into(),
                ));
            }
            _ => {
                transaction
                    .rollback()
                    .await
                    .map_err(PersistenceError::from)?;
                return Err(PersistenceError::SubjectInvariant(
                    "more than one SelfSubject exists".into(),
                ));
            }
        }

        if let Some(situation) = situation {
            if let Err(error) = sqlx::query(
                "INSERT INTO situations (id, subject_id, description, created_at_ms) VALUES (?, ?, ?, ?)",
            )
            .bind(situation.id().as_str())
            .bind(situation.subject_id().as_str())
            .bind(situation.description())
            .bind(created_at_ms)
            .execute(&mut *transaction)
            .await
            {
                let error = write_error(OPERATION, error);
                transaction.rollback().await.map_err(PersistenceError::from)?;
                return Err(error);
            }
        }

        for observation in observations {
            if let Err(error) = sqlx::query(
                "INSERT INTO observations (id, subject_id, situation_id, content, created_at_ms) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(observation.id().as_str())
            .bind(observation.subject_id().as_str())
            .bind(observation.situation_id().map(crate::domain::SituationId::as_str))
            .bind(observation.content())
            .bind(created_at_ms)
            .execute(&mut *transaction)
            .await
            {
                let error = write_error(OPERATION, error);
                transaction.rollback().await.map_err(PersistenceError::from)?;
                return Err(error);
            }
        }

        for thought in thoughts {
            if let Err(error) = sqlx::query(
                "INSERT INTO thoughts (id, subject_id, situation_id, content, confidence, created_at_ms) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(thought.id().as_str())
            .bind(thought.subject_id().as_str())
            .bind(thought.situation_id().map(crate::domain::SituationId::as_str))
            .bind(thought.content())
            .bind(thought.confidence().map(|value| i64::from(value.value())))
            .bind(created_at_ms)
            .execute(&mut *transaction)
            .await
            {
                let error = write_error(OPERATION, error);
                transaction.rollback().await.map_err(PersistenceError::from)?;
                return Err(error);
            }
        }

        transaction.commit().await.map_err(PersistenceError::from)
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

    pub(crate) async fn create_memory(
        &self,
        memory: &crate::domain::Memory,
        created_at_ms: i64,
    ) -> Result<(), PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        sqlx::query("INSERT INTO memories (id, subject_id, situation_id, description, user_meaning, created_at_ms) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(memory.id().as_str())
            .bind(memory.subject_id().as_str())
            .bind(memory.situation_id().map(crate::domain::SituationId::as_str))
            .bind(memory.description())
            .bind(memory.user_meaning())
            .bind(created_at_ms)
            .execute(&mut **connection.connection())
            .await
            .map_err(|error| write_error("create_memory", error))?;
        Ok(())
    }

    pub(crate) async fn load_memory(
        &self,
        id: &crate::domain::MemoryId,
    ) -> Result<crate::domain::Memory, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let row: Option<MemoryDatabaseRow> =
            sqlx::query_as("SELECT id, subject_id, situation_id, description, user_meaning, created_at_ms FROM memories WHERE id = ?")
                .bind(id.as_str())
                .fetch_optional(&mut **connection.connection())
                .await
                .map_err(PersistenceError::from)?;
        let (id, subject_id, situation_id, description, user_meaning, created_at_ms) = row
            .ok_or_else(|| PersistenceError::NotFound {
                entity: "Memory",
                id: id.as_str().to_owned(),
            })?;
        MemoryRow {
            id,
            subject_id,
            situation_id,
            description,
            user_meaning,
            created_at_ms,
        }
        .try_into()
    }

    pub(crate) async fn create_decision(
        &self,
        decision: &crate::domain::Decision,
        created_at_ms: i64,
    ) -> Result<(), PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        sqlx::query("INSERT INTO decisions (id, subject_id, situation_id, description, created_at_ms) VALUES (?, ?, ?, ?, ?)")
            .bind(decision.id().as_str())
            .bind(decision.subject_id().as_str())
            .bind(decision.situation_id().map(crate::domain::SituationId::as_str))
            .bind(decision.description())
            .bind(created_at_ms)
            .execute(&mut **connection.connection())
            .await
            .map_err(|error| write_error("create_decision", error))?;
        Ok(())
    }

    pub(crate) async fn load_decision(
        &self,
        id: &crate::domain::DecisionId,
    ) -> Result<crate::domain::Decision, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let row: Option<(String, String, Option<String>, String, i64)> = sqlx::query_as(
            "SELECT id, subject_id, situation_id, description, created_at_ms FROM decisions WHERE id = ?",
        )
        .bind(id.as_str())
        .fetch_optional(&mut **connection.connection())
        .await
        .map_err(PersistenceError::from)?;
        let (id, subject_id, situation_id, description, created_at_ms) =
            row.ok_or_else(|| PersistenceError::NotFound {
                entity: "Decision",
                id: id.as_str().to_owned(),
            })?;
        DecisionRow {
            id,
            subject_id,
            situation_id,
            description,
            created_at_ms,
        }
        .try_into()
    }

    pub(crate) async fn create_outcome(
        &self,
        outcome: &crate::domain::Outcome,
        created_at_ms: i64,
    ) -> Result<(), PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        sqlx::query("INSERT INTO outcomes (id, subject_id, decision_id, description, created_at_ms) VALUES (?, ?, ?, ?, ?)")
            .bind(outcome.id().as_str())
            .bind(outcome.subject_id().as_str())
            .bind(outcome.decision_id().as_str())
            .bind(outcome.description())
            .bind(created_at_ms)
            .execute(&mut **connection.connection())
            .await
            .map_err(|error| write_error("create_outcome", error))?;
        Ok(())
    }

    pub(crate) async fn load_outcome(
        &self,
        id: &crate::domain::OutcomeId,
    ) -> Result<crate::domain::Outcome, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let row: Option<(String, String, String, String, i64)> = sqlx::query_as(
            "SELECT id, subject_id, decision_id, description, created_at_ms FROM outcomes WHERE id = ?",
        )
        .bind(id.as_str())
        .fetch_optional(&mut **connection.connection())
        .await
        .map_err(PersistenceError::from)?;
        let (id, subject_id, decision_id, description, created_at_ms) =
            row.ok_or_else(|| PersistenceError::NotFound {
                entity: "Outcome",
                id: id.as_str().to_owned(),
            })?;
        OutcomeRow {
            id,
            subject_id,
            decision_id,
            description,
            created_at_ms,
        }
        .try_into()
    }

    pub(crate) async fn load_outcomes_for_decision(
        &self,
        decision_id: &crate::domain::DecisionId,
    ) -> Result<Vec<crate::domain::Outcome>, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let exists: Option<i64> = sqlx::query_scalar("SELECT 1 FROM decisions WHERE id = ?")
            .bind(decision_id.as_str())
            .fetch_optional(&mut **connection.connection())
            .await
            .map_err(PersistenceError::from)?;
        if exists.is_none() {
            return Err(PersistenceError::NotFound {
                entity: "Decision",
                id: decision_id.as_str().to_owned(),
            });
        }

        let rows: Vec<(String, String, String, String, i64)> = sqlx::query_as(
            "SELECT id, subject_id, decision_id, description, created_at_ms FROM outcomes WHERE decision_id = ? ORDER BY created_at_ms ASC, id ASC",
        )
        .bind(decision_id.as_str())
        .fetch_all(&mut **connection.connection())
        .await
        .map_err(PersistenceError::from)?;
        rows.into_iter()
            .map(
                |(id, subject_id, decision_id, description, created_at_ms)| {
                    OutcomeRow {
                        id,
                        subject_id,
                        decision_id,
                        description,
                        created_at_ms,
                    }
                    .try_into()
                },
            )
            .collect()
    }

    pub(crate) async fn create_evidence_link(
        &self,
        link: &crate::domain::EvidenceLink,
        created_at_ms: i64,
    ) -> Result<(), PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let (
            source_kind,
            source_observation_id,
            source_thought_id,
            source_emotion_id,
            source_situation_id,
            source_memory_id,
            source_decision_id,
            source_outcome_id,
        ) = evidence_source_columns(link.source());
        let (
            target_kind,
            target_belief_id,
            target_belief_revision_id,
            target_value_id,
            target_value_revision_id,
        ) = evidence_target_columns(link.target());

        sqlx::query(
            "INSERT INTO evidence_links (
                id, subject_id, relationship_kind, provenance, source_kind,
                source_observation_id, source_thought_id, source_emotion_id,
                source_situation_id, source_memory_id, source_decision_id, source_outcome_id,
                target_kind, target_belief_id, target_belief_revision_id,
                target_value_id, target_value_revision_id, user_note, created_at_ms
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(link.id().as_str())
        .bind(link.subject_id().as_str())
        .bind(evidence_relationship_name(link.relationship()))
        .bind(evidence_provenance_name(link.provenance()))
        .bind(source_kind)
        .bind(source_observation_id)
        .bind(source_thought_id)
        .bind(source_emotion_id)
        .bind(source_situation_id)
        .bind(source_memory_id)
        .bind(source_decision_id)
        .bind(source_outcome_id)
        .bind(target_kind)
        .bind(target_belief_id)
        .bind(target_belief_revision_id)
        .bind(target_value_id)
        .bind(target_value_revision_id)
        .bind(link.user_note())
        .bind(created_at_ms)
        .execute(&mut **connection.connection())
        .await
        .map_err(|error| write_error("create_evidence_link", error))?;
        Ok(())
    }

    pub(crate) async fn load_evidence_link(
        &self,
        id: &crate::domain::EvidenceLinkId,
    ) -> Result<crate::domain::EvidenceLink, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let row: Option<EvidenceLinkRow> = sqlx::query_as(
            "SELECT id, subject_id, relationship_kind, provenance, source_kind,
                    source_observation_id, source_thought_id, source_emotion_id,
                    source_situation_id, source_memory_id, source_decision_id,
                    source_outcome_id, target_kind, target_belief_id,
                    target_belief_revision_id, target_value_id,
                    target_value_revision_id, user_note, created_at_ms
             FROM evidence_links WHERE id = ?",
        )
        .bind(id.as_str())
        .fetch_optional(&mut **connection.connection())
        .await
        .map_err(PersistenceError::from)?;

        row.ok_or_else(|| PersistenceError::NotFound {
            entity: "EvidenceLink",
            id: id.as_str().to_owned(),
        })?
        .try_into()
    }

    pub(crate) async fn load_evidence_links_for_belief_revision(
        &self,
        revision_id: &crate::domain::BeliefRevisionId,
    ) -> Result<Vec<crate::domain::EvidenceLink>, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let exists: Option<i64> = sqlx::query_scalar("SELECT 1 FROM belief_revisions WHERE id = ?")
            .bind(revision_id.as_str())
            .fetch_optional(&mut **connection.connection())
            .await
            .map_err(PersistenceError::from)?;
        if exists.is_none() {
            return Err(PersistenceError::NotFound {
                entity: "BeliefRevision",
                id: revision_id.as_str().to_owned(),
            });
        }

        let rows: Vec<EvidenceLinkRow> = sqlx::query_as(
            "SELECT id, subject_id, relationship_kind, provenance, source_kind,
                    source_observation_id, source_thought_id, source_emotion_id,
                    source_situation_id, source_memory_id, source_decision_id,
                    source_outcome_id, target_kind, target_belief_id,
                    target_belief_revision_id, target_value_id,
                    target_value_revision_id, user_note, created_at_ms
             FROM evidence_links
             WHERE target_kind = 'BeliefRevision' AND target_belief_revision_id = ?
             ORDER BY created_at_ms ASC, id ASC",
        )
        .bind(revision_id.as_str())
        .fetch_all(&mut **connection.connection())
        .await
        .map_err(PersistenceError::from)?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    pub(crate) async fn load_evidence_links_for_value_revision(
        &self,
        revision_id: &crate::domain::ValueRevisionId,
    ) -> Result<Vec<crate::domain::EvidenceLink>, PersistenceError> {
        let mut connection = self.database.acquire_verified_connection().await?;
        let exists: Option<i64> = sqlx::query_scalar("SELECT 1 FROM value_revisions WHERE id = ?")
            .bind(revision_id.as_str())
            .fetch_optional(&mut **connection.connection())
            .await
            .map_err(PersistenceError::from)?;
        if exists.is_none() {
            return Err(PersistenceError::NotFound {
                entity: "ValueRevision",
                id: revision_id.as_str().to_owned(),
            });
        }

        let rows: Vec<EvidenceLinkRow> = sqlx::query_as(
            "SELECT id, subject_id, relationship_kind, provenance, source_kind,
                    source_observation_id, source_thought_id, source_emotion_id,
                    source_situation_id, source_memory_id, source_decision_id,
                    source_outcome_id, target_kind, target_belief_id,
                    target_belief_revision_id, target_value_id,
                    target_value_revision_id, user_note, created_at_ms
             FROM evidence_links
             WHERE target_kind = 'ValueRevision' AND target_value_revision_id = ?
             ORDER BY created_at_ms ASC, id ASC",
        )
        .bind(revision_id.as_str())
        .fetch_all(&mut **connection.connection())
        .await
        .map_err(PersistenceError::from)?;
        rows.into_iter().map(TryInto::try_into).collect()
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
struct MemoryRow {
    id: String,
    subject_id: String,
    situation_id: Option<String>,
    description: String,
    user_meaning: Option<String>,
    created_at_ms: i64,
}

impl TryFrom<MemoryRow> for crate::domain::Memory {
    type Error = PersistenceError;

    fn try_from(row: MemoryRow) -> Result<Self, Self::Error> {
        let _ = row.created_at_ms;
        let id = reconstruct("Memory", "id", crate::domain::MemoryId::new(row.id))?;
        let subject_id = reconstruct(
            "Memory",
            "subject_id",
            crate::domain::SelfSubjectId::new(row.subject_id),
        )?;
        let situation_id = row
            .situation_id
            .map(|id| {
                reconstruct(
                    "Memory",
                    "situation_id",
                    crate::domain::SituationId::new(id),
                )
            })
            .transpose()?;
        reconstruct(
            "Memory",
            "description_or_user_meaning",
            Self::new(
                id,
                subject_id,
                situation_id,
                row.description,
                row.user_meaning,
            ),
        )
    }
}

#[derive(Clone, Debug)]
struct DecisionRow {
    id: String,
    subject_id: String,
    situation_id: Option<String>,
    description: String,
    created_at_ms: i64,
}

impl TryFrom<DecisionRow> for crate::domain::Decision {
    type Error = PersistenceError;

    fn try_from(row: DecisionRow) -> Result<Self, Self::Error> {
        let _ = row.created_at_ms;
        let id = reconstruct("Decision", "id", crate::domain::DecisionId::new(row.id))?;
        let subject_id = reconstruct(
            "Decision",
            "subject_id",
            crate::domain::SelfSubjectId::new(row.subject_id),
        )?;
        let situation_id = row
            .situation_id
            .map(|id| {
                reconstruct(
                    "Decision",
                    "situation_id",
                    crate::domain::SituationId::new(id),
                )
            })
            .transpose()?;
        reconstruct(
            "Decision",
            "description",
            Self::new(id, subject_id, situation_id, row.description),
        )
    }
}

#[derive(Clone, Debug)]
struct OutcomeRow {
    id: String,
    subject_id: String,
    decision_id: String,
    description: String,
    created_at_ms: i64,
}

impl TryFrom<OutcomeRow> for crate::domain::Outcome {
    type Error = PersistenceError;

    fn try_from(row: OutcomeRow) -> Result<Self, Self::Error> {
        let _ = row.created_at_ms;
        let id = reconstruct("Outcome", "id", crate::domain::OutcomeId::new(row.id))?;
        let subject_id = reconstruct(
            "Outcome",
            "subject_id",
            crate::domain::SelfSubjectId::new(row.subject_id),
        )?;
        let decision_id = reconstruct(
            "Outcome",
            "decision_id",
            crate::domain::DecisionId::new(row.decision_id),
        )?;
        reconstruct(
            "Outcome",
            "description",
            Self::new(id, subject_id, decision_id, row.description),
        )
    }
}

type EvidenceSourceColumns<'a> = (
    &'static str,
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a str>,
);

fn evidence_source_columns(source: &crate::domain::EvidenceSource) -> EvidenceSourceColumns<'_> {
    match source {
        crate::domain::EvidenceSource::Observation(id) => (
            "Observation",
            Some(id.as_str()),
            None,
            None,
            None,
            None,
            None,
            None,
        ),
        crate::domain::EvidenceSource::Thought(id) => (
            "Thought",
            None,
            Some(id.as_str()),
            None,
            None,
            None,
            None,
            None,
        ),
        crate::domain::EvidenceSource::Emotion(id) => (
            "Emotion",
            None,
            None,
            Some(id.as_str()),
            None,
            None,
            None,
            None,
        ),
        crate::domain::EvidenceSource::Situation(id) => (
            "Situation",
            None,
            None,
            None,
            Some(id.as_str()),
            None,
            None,
            None,
        ),
        crate::domain::EvidenceSource::Memory(id) => (
            "Memory",
            None,
            None,
            None,
            None,
            Some(id.as_str()),
            None,
            None,
        ),
        crate::domain::EvidenceSource::Decision(id) => (
            "Decision",
            None,
            None,
            None,
            None,
            None,
            Some(id.as_str()),
            None,
        ),
        crate::domain::EvidenceSource::Outcome(id) => (
            "Outcome",
            None,
            None,
            None,
            None,
            None,
            None,
            Some(id.as_str()),
        ),
    }
}

type EvidenceTargetColumns<'a> = (
    &'static str,
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a str>,
);

fn evidence_target_columns(source: &crate::domain::EvidenceTarget) -> EvidenceTargetColumns<'_> {
    match source {
        crate::domain::EvidenceTarget::BeliefRevision {
            belief_id,
            revision_id,
        } => (
            "BeliefRevision",
            Some(belief_id.as_str()),
            Some(revision_id.as_str()),
            None,
            None,
        ),
        crate::domain::EvidenceTarget::ValueRevision {
            value_id,
            revision_id,
        } => (
            "ValueRevision",
            None,
            None,
            Some(value_id.as_str()),
            Some(revision_id.as_str()),
        ),
    }
}

fn evidence_relationship_name(relationship: crate::domain::EvidenceRelationKind) -> &'static str {
    match relationship {
        crate::domain::EvidenceRelationKind::Supports => "Supports",
        crate::domain::EvidenceRelationKind::Contradicts => "Contradicts",
        crate::domain::EvidenceRelationKind::Complicates => "Complicates",
        crate::domain::EvidenceRelationKind::Contextualizes => "Contextualizes",
    }
}

fn evidence_provenance_name(provenance: crate::domain::EvidenceProvenance) -> &'static str {
    match provenance {
        crate::domain::EvidenceProvenance::UserAuthored => "UserAuthored",
    }
}

#[derive(Clone, Debug)]
struct EvidenceLinkRow {
    id: String,
    subject_id: String,
    relationship_kind: String,
    provenance: String,
    source_kind: String,
    source_observation_id: Option<String>,
    source_thought_id: Option<String>,
    source_emotion_id: Option<String>,
    source_situation_id: Option<String>,
    source_memory_id: Option<String>,
    source_decision_id: Option<String>,
    source_outcome_id: Option<String>,
    target_kind: String,
    target_belief_id: Option<String>,
    target_belief_revision_id: Option<String>,
    target_value_id: Option<String>,
    target_value_revision_id: Option<String>,
    user_note: Option<String>,
    created_at_ms: i64,
}

impl<'row> sqlx::FromRow<'row, SqliteRow> for EvidenceLinkRow {
    fn from_row(row: &'row SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            subject_id: row.try_get("subject_id")?,
            relationship_kind: row.try_get("relationship_kind")?,
            provenance: row.try_get("provenance")?,
            source_kind: row.try_get("source_kind")?,
            source_observation_id: row.try_get("source_observation_id")?,
            source_thought_id: row.try_get("source_thought_id")?,
            source_emotion_id: row.try_get("source_emotion_id")?,
            source_situation_id: row.try_get("source_situation_id")?,
            source_memory_id: row.try_get("source_memory_id")?,
            source_decision_id: row.try_get("source_decision_id")?,
            source_outcome_id: row.try_get("source_outcome_id")?,
            target_kind: row.try_get("target_kind")?,
            target_belief_id: row.try_get("target_belief_id")?,
            target_belief_revision_id: row.try_get("target_belief_revision_id")?,
            target_value_id: row.try_get("target_value_id")?,
            target_value_revision_id: row.try_get("target_value_revision_id")?,
            user_note: row.try_get("user_note")?,
            created_at_ms: row.try_get("created_at_ms")?,
        })
    }
}

fn evidence_reconstruction(field: &'static str, detail: impl Into<String>) -> PersistenceError {
    PersistenceError::DomainReconstruction {
        entity: "EvidenceLink",
        field,
        detail: detail.into(),
    }
}

impl TryFrom<EvidenceLinkRow> for crate::domain::EvidenceLink {
    type Error = PersistenceError;

    fn try_from(row: EvidenceLinkRow) -> Result<Self, Self::Error> {
        let _ = row.created_at_ms;
        let id = reconstruct(
            "EvidenceLink",
            "id",
            crate::domain::EvidenceLinkId::new(row.id),
        )?;
        let subject_id = reconstruct(
            "EvidenceLink",
            "subject_id",
            crate::domain::SelfSubjectId::new(row.subject_id),
        )?;

        let source_count = [
            row.source_observation_id.is_some(),
            row.source_thought_id.is_some(),
            row.source_emotion_id.is_some(),
            row.source_situation_id.is_some(),
            row.source_memory_id.is_some(),
            row.source_decision_id.is_some(),
            row.source_outcome_id.is_some(),
        ]
        .into_iter()
        .filter(|present| *present)
        .count();
        if source_count != 1 {
            return Err(evidence_reconstruction(
                "source",
                format!("expected exactly one typed source column, found {source_count}"),
            ));
        }
        let source = match row.source_kind.as_str() {
            "Observation"
                if row.source_thought_id.is_none()
                    && row.source_emotion_id.is_none()
                    && row.source_situation_id.is_none()
                    && row.source_memory_id.is_none()
                    && row.source_decision_id.is_none()
                    && row.source_outcome_id.is_none() =>
            {
                let value = row.source_observation_id.ok_or_else(|| {
                    evidence_reconstruction("source", "Observation source ID is absent")
                })?;
                crate::domain::EvidenceSource::Observation(reconstruct(
                    "EvidenceLink",
                    "source_observation_id",
                    crate::domain::ObservationId::new(value),
                )?)
            }
            "Thought"
                if row.source_observation_id.is_none()
                    && row.source_emotion_id.is_none()
                    && row.source_situation_id.is_none()
                    && row.source_memory_id.is_none()
                    && row.source_decision_id.is_none()
                    && row.source_outcome_id.is_none() =>
            {
                let value = row.source_thought_id.ok_or_else(|| {
                    evidence_reconstruction("source", "Thought source ID is absent")
                })?;
                crate::domain::EvidenceSource::Thought(reconstruct(
                    "EvidenceLink",
                    "source_thought_id",
                    crate::domain::ThoughtId::new(value),
                )?)
            }
            "Emotion"
                if row.source_observation_id.is_none()
                    && row.source_thought_id.is_none()
                    && row.source_situation_id.is_none()
                    && row.source_memory_id.is_none()
                    && row.source_decision_id.is_none()
                    && row.source_outcome_id.is_none() =>
            {
                let value = row.source_emotion_id.ok_or_else(|| {
                    evidence_reconstruction("source", "Emotion source ID is absent")
                })?;
                crate::domain::EvidenceSource::Emotion(reconstruct(
                    "EvidenceLink",
                    "source_emotion_id",
                    crate::domain::EmotionId::new(value),
                )?)
            }
            "Situation"
                if row.source_observation_id.is_none()
                    && row.source_thought_id.is_none()
                    && row.source_emotion_id.is_none()
                    && row.source_memory_id.is_none()
                    && row.source_decision_id.is_none()
                    && row.source_outcome_id.is_none() =>
            {
                let value = row.source_situation_id.ok_or_else(|| {
                    evidence_reconstruction("source", "Situation source ID is absent")
                })?;
                crate::domain::EvidenceSource::Situation(reconstruct(
                    "EvidenceLink",
                    "source_situation_id",
                    crate::domain::SituationId::new(value),
                )?)
            }
            "Memory"
                if row.source_observation_id.is_none()
                    && row.source_thought_id.is_none()
                    && row.source_emotion_id.is_none()
                    && row.source_situation_id.is_none()
                    && row.source_decision_id.is_none()
                    && row.source_outcome_id.is_none() =>
            {
                let value = row.source_memory_id.ok_or_else(|| {
                    evidence_reconstruction("source", "Memory source ID is absent")
                })?;
                crate::domain::EvidenceSource::Memory(reconstruct(
                    "EvidenceLink",
                    "source_memory_id",
                    crate::domain::MemoryId::new(value),
                )?)
            }
            "Decision"
                if row.source_observation_id.is_none()
                    && row.source_thought_id.is_none()
                    && row.source_emotion_id.is_none()
                    && row.source_situation_id.is_none()
                    && row.source_memory_id.is_none()
                    && row.source_outcome_id.is_none() =>
            {
                let value = row.source_decision_id.ok_or_else(|| {
                    evidence_reconstruction("source", "Decision source ID is absent")
                })?;
                crate::domain::EvidenceSource::Decision(reconstruct(
                    "EvidenceLink",
                    "source_decision_id",
                    crate::domain::DecisionId::new(value),
                )?)
            }
            "Outcome"
                if row.source_observation_id.is_none()
                    && row.source_thought_id.is_none()
                    && row.source_emotion_id.is_none()
                    && row.source_situation_id.is_none()
                    && row.source_memory_id.is_none()
                    && row.source_decision_id.is_none() =>
            {
                let value = row.source_outcome_id.ok_or_else(|| {
                    evidence_reconstruction("source", "Outcome source ID is absent")
                })?;
                crate::domain::EvidenceSource::Outcome(reconstruct(
                    "EvidenceLink",
                    "source_outcome_id",
                    crate::domain::OutcomeId::new(value),
                )?)
            }
            _ => {
                return Err(evidence_reconstruction(
                    "source",
                    format!(
                        "unknown source kind or inconsistent typed source columns: {}",
                        row.source_kind
                    ),
                ));
            }
        };

        let target = match row.target_kind.as_str() {
            "BeliefRevision"
                if row.target_value_id.is_none() && row.target_value_revision_id.is_none() =>
            {
                let belief_id = row.target_belief_id.ok_or_else(|| {
                    evidence_reconstruction("target", "Belief target anchor ID is absent")
                })?;
                let revision_id = row.target_belief_revision_id.ok_or_else(|| {
                    evidence_reconstruction("target", "Belief target revision ID is absent")
                })?;
                crate::domain::EvidenceTarget::BeliefRevision {
                    belief_id: reconstruct(
                        "EvidenceLink",
                        "target_belief_id",
                        crate::domain::BeliefId::new(belief_id),
                    )?,
                    revision_id: reconstruct(
                        "EvidenceLink",
                        "target_belief_revision_id",
                        crate::domain::BeliefRevisionId::new(revision_id),
                    )?,
                }
            }
            "ValueRevision"
                if row.target_belief_id.is_none() && row.target_belief_revision_id.is_none() =>
            {
                let value_id = row.target_value_id.ok_or_else(|| {
                    evidence_reconstruction("target", "Value target anchor ID is absent")
                })?;
                let revision_id = row.target_value_revision_id.ok_or_else(|| {
                    evidence_reconstruction("target", "Value target revision ID is absent")
                })?;
                crate::domain::EvidenceTarget::ValueRevision {
                    value_id: reconstruct(
                        "EvidenceLink",
                        "target_value_id",
                        crate::domain::ValueId::new(value_id),
                    )?,
                    revision_id: reconstruct(
                        "EvidenceLink",
                        "target_value_revision_id",
                        crate::domain::ValueRevisionId::new(revision_id),
                    )?,
                }
            }
            _ => {
                return Err(evidence_reconstruction(
                    "target",
                    format!(
                        "unknown target kind or inconsistent typed target columns: {}",
                        row.target_kind
                    ),
                ));
            }
        };

        let relationship = match row.relationship_kind.as_str() {
            "Supports" => crate::domain::EvidenceRelationKind::Supports,
            "Contradicts" => crate::domain::EvidenceRelationKind::Contradicts,
            "Complicates" => crate::domain::EvidenceRelationKind::Complicates,
            "Contextualizes" => crate::domain::EvidenceRelationKind::Contextualizes,
            _ => {
                return Err(evidence_reconstruction(
                    "relationship",
                    format!("unknown relationship kind: {}", row.relationship_kind),
                ));
            }
        };
        let provenance = match row.provenance.as_str() {
            "UserAuthored" => crate::domain::EvidenceProvenance::UserAuthored,
            _ => {
                return Err(evidence_reconstruction(
                    "provenance",
                    format!("unknown provenance: {}", row.provenance),
                ));
            }
        };

        reconstruct(
            "EvidenceLink",
            "fields",
            crate::domain::EvidenceLink::new(
                id,
                subject_id,
                source,
                target,
                relationship,
                provenance,
                row.user_note,
            ),
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
    const MIGRATION_0003: &str =
        include_str!("../migrations/0003_create_lived_experience_records.sql");
    const MIGRATION_0004: &str = include_str!("../migrations/0004_create_evidence_links.sql");

    type ColumnExpectation = (&'static str, &'static str, i64, i64);
    type TableColumnExpectations = (&'static str, &'static [ColumnExpectation]);
    type ForeignKeyExpectation = (&'static str, &'static str, &'static str, &'static str);
    type TableForeignKeyExpectations = (&'static str, &'static [ForeignKeyExpectation]);
    type ForeignKeyGroup = (String, Vec<(String, String)>, String);

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
        database.apply(MIGRATION_0003).await;
        database.apply(MIGRATION_0004).await;
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

    async fn assert_task005_tables_exist(pool: &SqlitePool) {
        let tables: Vec<String> =
            sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
                .fetch_all(pool)
                .await
                .unwrap();
        for table in ["memories", "decisions", "outcomes"] {
            assert!(tables.iter().any(|found| found == table), "missing {table}");
        }
    }

    async fn assert_task006_table_exists(pool: &SqlitePool) {
        let tables: Vec<String> =
            sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
                .fetch_all(pool)
                .await
                .unwrap();
        assert!(tables.iter().any(|found| found == "evidence_links"));
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

    async fn index_columns(pool: &SqlitePool, index: &str) -> Vec<String> {
        sqlx::query_scalar("SELECT name FROM pragma_index_info(?) ORDER BY seqno")
            .bind(index)
            .fetch_all(pool)
            .await
            .unwrap()
    }

    async fn foreign_key_groups(pool: &SqlitePool, table: &str) -> Vec<ForeignKeyGroup> {
        let rows: Vec<(i64, i64, String, String, String, String)> = sqlx::query_as(
            "SELECT id, seq, \"table\", \"from\", \"to\", on_delete \
             FROM pragma_foreign_key_list(?) ORDER BY id, seq",
        )
        .bind(table)
        .fetch_all(pool)
        .await
        .unwrap();
        let mut groups: Vec<ForeignKeyGroup> = Vec::new();
        for (id, _, parent, from, to, on_delete) in rows {
            if let Some((_, columns, _)) = groups.get_mut(id as usize) {
                columns.push((from, to));
            } else {
                assert_eq!(id as usize, groups.len());
                groups.push((parent, vec![(from, to)], on_delete));
            }
        }
        groups
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

    async fn create_subject_fixture(
        repository: &SqliteSelfModelRepository,
        id: &str,
    ) -> crate::domain::SelfSubject {
        let subject = crate::domain::SelfSubject::new(
            crate::domain::SelfSubjectId::new(id).unwrap(),
            format!("Subject {id}"),
        )
        .unwrap();
        repository.create_self_subject(&subject, 1).await.unwrap();
        subject
    }

    async fn create_situation_fixture(
        repository: &SqliteSelfModelRepository,
        subject: &crate::domain::SelfSubject,
        id: &str,
    ) -> crate::domain::Situation {
        let situation = crate::domain::Situation::new(
            crate::domain::SituationId::new(id).unwrap(),
            subject.id().clone(),
            format!("Situation {id}"),
        )
        .unwrap();
        repository.create_situation(&situation, 2).await.unwrap();
        situation
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

    async fn seed_evidence_records(pool: &SqlitePool) {
        sqlx::raw_sql(
            "INSERT INTO self_subjects (id, display_name, created_at_ms) VALUES
                ('subject-a', 'Subject A', 1),
                ('subject-b', 'Subject B', 2);
             INSERT INTO situations (id, subject_id, description, created_at_ms) VALUES
                ('situation-a', 'subject-a', 'Situation A', 3),
                ('situation-b', 'subject-b', 'Situation B', 4);
             INSERT INTO observations
                (id, subject_id, situation_id, content, created_at_ms) VALUES
                ('observation-a', 'subject-a', NULL, 'Observation A', 5),
                ('observation-b', 'subject-b', NULL, 'Observation B', 6);
             INSERT INTO thoughts
                (id, subject_id, situation_id, content, confidence, created_at_ms) VALUES
                ('thought-a', 'subject-a', NULL, 'Thought A', NULL, 7),
                ('thought-b', 'subject-b', NULL, 'Thought B', NULL, 8);
             INSERT INTO emotions
                (id, subject_id, situation_id, label, intensity, created_at_ms) VALUES
                ('emotion-a', 'subject-a', NULL, 'Emotion A', 50, 9),
                ('emotion-b', 'subject-b', NULL, 'Emotion B', 50, 10);
             INSERT INTO memories
                (id, subject_id, situation_id, description, user_meaning, created_at_ms) VALUES
                ('memory-a', 'subject-a', NULL, 'Memory A', NULL, 11),
                ('memory-b', 'subject-b', NULL, 'Memory B', NULL, 12);
             INSERT INTO decisions
                (id, subject_id, situation_id, description, created_at_ms) VALUES
                ('decision-a', 'subject-a', NULL, 'Decision A', 13),
                ('decision-b', 'subject-b', NULL, 'Decision B', 14);
             INSERT INTO outcomes
                (id, subject_id, decision_id, description, created_at_ms) VALUES
                ('outcome-a', 'subject-a', 'decision-a', 'Outcome A', 15),
                ('outcome-b', 'subject-b', 'decision-b', 'Outcome B', 16);
             INSERT INTO beliefs (id, subject_id, created_at_ms) VALUES
                ('belief-a', 'subject-a', 17),
                ('belief-a-other', 'subject-a', 18),
                ('belief-b', 'subject-b', 19);
             INSERT INTO belief_revisions
                (id, belief_id, revision_number, proposition, endorsement, change_note, origin, created_at_ms) VALUES
                ('belief-revision-a', 'belief-a', 1, 'Belief A', NULL, NULL, 'InitialUserEntry', 20),
                ('belief-revision-a-other', 'belief-a-other', 1, 'Other Belief A', NULL, NULL, 'InitialUserEntry', 21),
                ('belief-revision-b', 'belief-b', 1, 'Belief B', NULL, NULL, 'InitialUserEntry', 22);
             INSERT INTO \"values\" (id, subject_id, created_at_ms) VALUES
                ('value-a', 'subject-a', 23),
                ('value-a-other', 'subject-a', 24),
                ('value-b', 'subject-b', 25);
             INSERT INTO value_revisions
                (id, value_id, revision_number, label, importance, change_note, origin, created_at_ms) VALUES
                ('value-revision-a', 'value-a', 1, 'Value A', NULL, NULL, 'InitialUserEntry', 26),
                ('value-revision-a-other', 'value-a-other', 1, 'Other Value A', NULL, NULL, 'InitialUserEntry', 27),
                ('value-revision-b', 'value-b', 1, 'Value B', NULL, NULL, 'InitialUserEntry', 28);",
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[derive(Clone)]
    struct EvidenceInsert {
        id: String,
        subject_id: String,
        relationship_kind: String,
        provenance: String,
        source_kind: String,
        source_observation_id: Option<String>,
        source_thought_id: Option<String>,
        source_emotion_id: Option<String>,
        source_situation_id: Option<String>,
        source_memory_id: Option<String>,
        source_decision_id: Option<String>,
        source_outcome_id: Option<String>,
        target_kind: String,
        target_belief_id: Option<String>,
        target_belief_revision_id: Option<String>,
        target_value_id: Option<String>,
        target_value_revision_id: Option<String>,
        user_note: Option<String>,
        created_at_ms: i64,
    }

    impl EvidenceInsert {
        fn new(
            id: impl Into<String>,
            subject_id: &str,
            source_kind: &str,
            source_id: &str,
            target_kind: &str,
            relationship_kind: &str,
        ) -> Self {
            let mut row = Self {
                id: id.into(),
                subject_id: subject_id.to_owned(),
                relationship_kind: relationship_kind.to_owned(),
                provenance: "UserAuthored".to_owned(),
                source_kind: source_kind.to_owned(),
                source_observation_id: None,
                source_thought_id: None,
                source_emotion_id: None,
                source_situation_id: None,
                source_memory_id: None,
                source_decision_id: None,
                source_outcome_id: None,
                target_kind: target_kind.to_owned(),
                target_belief_id: None,
                target_belief_revision_id: None,
                target_value_id: None,
                target_value_revision_id: None,
                user_note: None,
                created_at_ms: 100,
            };
            match source_kind {
                "Observation" => row.source_observation_id = Some(source_id.to_owned()),
                "Thought" => row.source_thought_id = Some(source_id.to_owned()),
                "Emotion" => row.source_emotion_id = Some(source_id.to_owned()),
                "Situation" => row.source_situation_id = Some(source_id.to_owned()),
                "Memory" => row.source_memory_id = Some(source_id.to_owned()),
                "Decision" => row.source_decision_id = Some(source_id.to_owned()),
                "Outcome" => row.source_outcome_id = Some(source_id.to_owned()),
                other => panic!("unsupported evidence fixture source {other}"),
            }
            match (target_kind, subject_id) {
                ("BeliefRevision", "subject-a") => {
                    row.target_belief_id = Some("belief-a".to_owned());
                    row.target_belief_revision_id = Some("belief-revision-a".to_owned());
                }
                ("BeliefRevision", "subject-b") => {
                    row.target_belief_id = Some("belief-b".to_owned());
                    row.target_belief_revision_id = Some("belief-revision-b".to_owned());
                }
                ("ValueRevision", "subject-a") => {
                    row.target_value_id = Some("value-a".to_owned());
                    row.target_value_revision_id = Some("value-revision-a".to_owned());
                }
                ("ValueRevision", "subject-b") => {
                    row.target_value_id = Some("value-b".to_owned());
                    row.target_value_revision_id = Some("value-revision-b".to_owned());
                }
                _ => panic!("unsupported evidence fixture target"),
            }
            row
        }

        async fn execute(
            &self,
            pool: &SqlitePool,
        ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
            sqlx::query(
                "INSERT INTO evidence_links (
                    id, subject_id, relationship_kind, provenance, source_kind,
                    source_observation_id, source_thought_id, source_emotion_id,
                    source_situation_id, source_memory_id, source_decision_id, source_outcome_id,
                    target_kind, target_belief_id, target_belief_revision_id,
                    target_value_id, target_value_revision_id, user_note, created_at_ms
                 ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&self.id)
            .bind(&self.subject_id)
            .bind(&self.relationship_kind)
            .bind(&self.provenance)
            .bind(&self.source_kind)
            .bind(self.source_observation_id.as_deref())
            .bind(self.source_thought_id.as_deref())
            .bind(self.source_emotion_id.as_deref())
            .bind(self.source_situation_id.as_deref())
            .bind(self.source_memory_id.as_deref())
            .bind(self.source_decision_id.as_deref())
            .bind(self.source_outcome_id.as_deref())
            .bind(&self.target_kind)
            .bind(self.target_belief_id.as_deref())
            .bind(self.target_belief_revision_id.as_deref())
            .bind(self.target_value_id.as_deref())
            .bind(self.target_value_revision_id.as_deref())
            .bind(self.user_note.as_deref())
            .bind(self.created_at_ms)
            .execute(pool)
            .await
        }
    }

    fn evidence_link(
        id: &str,
        subject_id: &str,
        source: crate::domain::EvidenceSource,
        target: crate::domain::EvidenceTarget,
        relationship: crate::domain::EvidenceRelationKind,
        user_note: Option<&str>,
    ) -> crate::domain::EvidenceLink {
        crate::domain::EvidenceLink::new(
            crate::domain::EvidenceLinkId::new(id).unwrap(),
            crate::domain::SelfSubjectId::new(subject_id).unwrap(),
            source,
            target,
            relationship,
            crate::domain::EvidenceProvenance::UserAuthored,
            user_note.map(str::to_owned),
        )
        .unwrap()
    }

    fn belief_evidence_target(belief_id: &str, revision_id: &str) -> crate::domain::EvidenceTarget {
        crate::domain::EvidenceTarget::BeliefRevision {
            belief_id: crate::domain::BeliefId::new(belief_id).unwrap(),
            revision_id: crate::domain::BeliefRevisionId::new(revision_id).unwrap(),
        }
    }

    fn value_evidence_target(value_id: &str, revision_id: &str) -> crate::domain::EvidenceTarget {
        crate::domain::EvidenceTarget::ValueRevision {
            value_id: crate::domain::ValueId::new(value_id).unwrap(),
            revision_id: crate::domain::ValueRevisionId::new(revision_id).unwrap(),
        }
    }

    async fn apply_test_only_evidence_corruption(pool: &SqlitePool, sql: &str) {
        let mut connection = pool.acquire().await.unwrap();
        sqlx::query("PRAGMA foreign_keys = OFF")
            .execute(&mut *connection)
            .await
            .unwrap();
        sqlx::query("PRAGMA ignore_check_constraints = ON")
            .execute(&mut *connection)
            .await
            .unwrap();
        sqlx::raw_sql(sql).execute(&mut *connection).await.unwrap();
        sqlx::query("PRAGMA ignore_check_constraints = OFF")
            .execute(&mut *connection)
            .await
            .unwrap();
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&mut *connection)
            .await
            .unwrap();
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
    fn checked_in_migrations_create_all_domain_tables_and_update_schema_version() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            assert_task004_tables_exist(&database.pool).await;
            assert_task005_tables_exist(&database.pool).await;
            assert_task006_table_exists(&database.pool).await;
            let tables: Vec<String> = sqlx::query_scalar(
                "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
            )
            .fetch_all(&database.pool)
            .await
            .unwrap();
            assert_eq!(
                tables,
                [
                    "app_metadata",
                    "belief_revisions",
                    "beliefs",
                    "decisions",
                    "emotions",
                    "evidence_links",
                    "memories",
                    "observations",
                    "outcomes",
                    "person_references",
                    "self_subjects",
                    "situations",
                    "thoughts",
                    "value_revisions",
                    "values",
                ]
            );
            let version: String =
                sqlx::query_scalar("SELECT value FROM app_metadata WHERE key = 'schema_version'")
                    .fetch_one(&database.pool)
                    .await
                    .unwrap();
            assert_eq!(version, "4");
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
                (
                    "memories",
                    &[
                        ("id", "TEXT", 1, 1),
                        ("subject_id", "TEXT", 1, 0),
                        ("situation_id", "TEXT", 0, 0),
                        ("description", "TEXT", 1, 0),
                        ("user_meaning", "TEXT", 0, 0),
                        ("created_at_ms", "INTEGER", 1, 0),
                    ],
                ),
                (
                    "decisions",
                    &[
                        ("id", "TEXT", 1, 1),
                        ("subject_id", "TEXT", 1, 0),
                        ("situation_id", "TEXT", 0, 0),
                        ("description", "TEXT", 1, 0),
                        ("created_at_ms", "INTEGER", 1, 0),
                    ],
                ),
                (
                    "outcomes",
                    &[
                        ("id", "TEXT", 1, 1),
                        ("subject_id", "TEXT", 1, 0),
                        ("decision_id", "TEXT", 1, 0),
                        ("description", "TEXT", 1, 0),
                        ("created_at_ms", "INTEGER", 1, 0),
                    ],
                ),
                (
                    "evidence_links",
                    &[
                        ("id", "TEXT", 1, 1),
                        ("subject_id", "TEXT", 1, 0),
                        ("relationship_kind", "TEXT", 1, 0),
                        ("provenance", "TEXT", 1, 0),
                        ("source_kind", "TEXT", 1, 0),
                        ("source_observation_id", "TEXT", 0, 0),
                        ("source_thought_id", "TEXT", 0, 0),
                        ("source_emotion_id", "TEXT", 0, 0),
                        ("source_situation_id", "TEXT", 0, 0),
                        ("source_memory_id", "TEXT", 0, 0),
                        ("source_decision_id", "TEXT", 0, 0),
                        ("source_outcome_id", "TEXT", 0, 0),
                        ("target_kind", "TEXT", 1, 0),
                        ("target_belief_id", "TEXT", 0, 0),
                        ("target_belief_revision_id", "TEXT", 0, 0),
                        ("target_value_id", "TEXT", 0, 0),
                        ("target_value_revision_id", "TEXT", 0, 0),
                        ("user_note", "TEXT", 0, 0),
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
                (
                    "memories",
                    &[
                        ("self_subjects", "subject_id", "id", "RESTRICT"),
                        ("situations", "situation_id", "id", "RESTRICT"),
                        ("situations", "subject_id", "subject_id", "RESTRICT"),
                    ],
                ),
                (
                    "decisions",
                    &[
                        ("self_subjects", "subject_id", "id", "RESTRICT"),
                        ("situations", "situation_id", "id", "RESTRICT"),
                        ("situations", "subject_id", "subject_id", "RESTRICT"),
                    ],
                ),
                (
                    "outcomes",
                    &[
                        ("self_subjects", "subject_id", "id", "RESTRICT"),
                        ("decisions", "decision_id", "id", "RESTRICT"),
                        ("decisions", "subject_id", "subject_id", "RESTRICT"),
                    ],
                ),
                (
                    "evidence_links",
                    &[
                        ("self_subjects", "subject_id", "id", "RESTRICT"),
                        ("observations", "source_observation_id", "id", "RESTRICT"),
                        ("observations", "subject_id", "subject_id", "RESTRICT"),
                        ("thoughts", "source_thought_id", "id", "RESTRICT"),
                        ("thoughts", "subject_id", "subject_id", "RESTRICT"),
                        ("emotions", "source_emotion_id", "id", "RESTRICT"),
                        ("emotions", "subject_id", "subject_id", "RESTRICT"),
                        ("situations", "source_situation_id", "id", "RESTRICT"),
                        ("situations", "subject_id", "subject_id", "RESTRICT"),
                        ("memories", "source_memory_id", "id", "RESTRICT"),
                        ("memories", "subject_id", "subject_id", "RESTRICT"),
                        ("decisions", "source_decision_id", "id", "RESTRICT"),
                        ("decisions", "subject_id", "subject_id", "RESTRICT"),
                        ("outcomes", "source_outcome_id", "id", "RESTRICT"),
                        ("outcomes", "subject_id", "subject_id", "RESTRICT"),
                        ("beliefs", "target_belief_id", "id", "RESTRICT"),
                        ("beliefs", "subject_id", "subject_id", "RESTRICT"),
                        (
                            "belief_revisions",
                            "target_belief_revision_id",
                            "id",
                            "RESTRICT",
                        ),
                        (
                            "belief_revisions",
                            "target_belief_id",
                            "belief_id",
                            "RESTRICT",
                        ),
                        ("values", "target_value_id", "id", "RESTRICT"),
                        ("values", "subject_id", "subject_id", "RESTRICT"),
                        (
                            "value_revisions",
                            "target_value_revision_id",
                            "id",
                            "RESTRICT",
                        ),
                        ("value_revisions", "target_value_id", "value_id", "RESTRICT"),
                    ],
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
                    "belief_revisions_id_belief_unique_idx",
                    "beliefs_id_subject_unique_idx",
                    "beliefs_subject_id_idx",
                    "decisions_situation_subject_idx",
                    "decisions_subject_id_idx",
                    "emotions_id_subject_unique_idx",
                    "emotions_situation_subject_idx",
                    "emotions_subject_id_idx",
                    "evidence_links_belief_revision_target_idx",
                    "evidence_links_source_decision_subject_idx",
                    "evidence_links_source_emotion_subject_idx",
                    "evidence_links_source_memory_subject_idx",
                    "evidence_links_source_observation_subject_idx",
                    "evidence_links_source_outcome_subject_idx",
                    "evidence_links_source_situation_subject_idx",
                    "evidence_links_source_thought_subject_idx",
                    "evidence_links_subject_id_idx",
                    "evidence_links_value_revision_target_idx",
                    "memories_id_subject_unique_idx",
                    "memories_situation_subject_idx",
                    "memories_subject_id_idx",
                    "observations_id_subject_unique_idx",
                    "observations_situation_subject_idx",
                    "observations_subject_id_idx",
                    "outcomes_decision_subject_idx",
                    "outcomes_id_subject_unique_idx",
                    "outcomes_subject_id_idx",
                    "person_references_subject_id_idx",
                    "situations_subject_id_idx",
                    "thoughts_id_subject_unique_idx",
                    "thoughts_situation_subject_idx",
                    "thoughts_subject_id_idx",
                    "value_revisions_id_value_unique_idx",
                    "value_revisions_value_id_idx",
                    "values_id_subject_unique_idx",
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
            assert!(unique_index_columns(&database.pool, "decisions")
                .await
                .contains(&vec!["id".into(), "subject_id".into()]));
            for table in [
                "observations",
                "thoughts",
                "emotions",
                "memories",
                "outcomes",
                "beliefs",
                "values",
            ] {
                assert!(unique_index_columns(&database.pool, table)
                    .await
                    .contains(&vec!["id".into(), "subject_id".into()]));
            }
            assert!(unique_index_columns(&database.pool, "belief_revisions")
                .await
                .contains(&vec!["id".into(), "belief_id".into()]));
            assert!(unique_index_columns(&database.pool, "value_revisions")
                .await
                .contains(&vec!["id".into(), "value_id".into()]));

            for (index, expected) in [
                (
                    "observations_id_subject_unique_idx",
                    vec!["id", "subject_id"],
                ),
                ("thoughts_id_subject_unique_idx", vec!["id", "subject_id"]),
                ("emotions_id_subject_unique_idx", vec!["id", "subject_id"]),
                ("memories_id_subject_unique_idx", vec!["id", "subject_id"]),
                ("outcomes_id_subject_unique_idx", vec!["id", "subject_id"]),
                ("beliefs_id_subject_unique_idx", vec!["id", "subject_id"]),
                (
                    "belief_revisions_id_belief_unique_idx",
                    vec!["id", "belief_id"],
                ),
                ("values_id_subject_unique_idx", vec!["id", "subject_id"]),
                (
                    "value_revisions_id_value_unique_idx",
                    vec!["id", "value_id"],
                ),
                (
                    "evidence_links_source_observation_subject_idx",
                    vec!["source_observation_id", "subject_id"],
                ),
                (
                    "evidence_links_source_thought_subject_idx",
                    vec!["source_thought_id", "subject_id"],
                ),
                (
                    "evidence_links_source_emotion_subject_idx",
                    vec!["source_emotion_id", "subject_id"],
                ),
                (
                    "evidence_links_source_situation_subject_idx",
                    vec!["source_situation_id", "subject_id"],
                ),
                (
                    "evidence_links_source_memory_subject_idx",
                    vec!["source_memory_id", "subject_id"],
                ),
                (
                    "evidence_links_source_decision_subject_idx",
                    vec!["source_decision_id", "subject_id"],
                ),
                (
                    "evidence_links_source_outcome_subject_idx",
                    vec!["source_outcome_id", "subject_id"],
                ),
                (
                    "evidence_links_belief_revision_target_idx",
                    vec!["target_belief_revision_id", "target_belief_id"],
                ),
                (
                    "evidence_links_value_revision_target_idx",
                    vec!["target_value_revision_id", "target_value_id"],
                ),
            ] {
                assert_eq!(index_columns(&database.pool, index).await, expected);
            }

            let evidence_foreign_keys = foreign_key_groups(&database.pool, "evidence_links").await;
            for expected in [
                (
                    "observations".to_owned(),
                    vec![
                        ("source_observation_id".to_owned(), "id".to_owned()),
                        ("subject_id".to_owned(), "subject_id".to_owned()),
                    ],
                    "RESTRICT".to_owned(),
                ),
                (
                    "situations".to_owned(),
                    vec![
                        ("source_situation_id".to_owned(), "id".to_owned()),
                        ("subject_id".to_owned(), "subject_id".to_owned()),
                    ],
                    "RESTRICT".to_owned(),
                ),
                (
                    "thoughts".to_owned(),
                    vec![
                        ("source_thought_id".to_owned(), "id".to_owned()),
                        ("subject_id".to_owned(), "subject_id".to_owned()),
                    ],
                    "RESTRICT".to_owned(),
                ),
                (
                    "emotions".to_owned(),
                    vec![
                        ("source_emotion_id".to_owned(), "id".to_owned()),
                        ("subject_id".to_owned(), "subject_id".to_owned()),
                    ],
                    "RESTRICT".to_owned(),
                ),
                (
                    "memories".to_owned(),
                    vec![
                        ("source_memory_id".to_owned(), "id".to_owned()),
                        ("subject_id".to_owned(), "subject_id".to_owned()),
                    ],
                    "RESTRICT".to_owned(),
                ),
                (
                    "decisions".to_owned(),
                    vec![
                        ("source_decision_id".to_owned(), "id".to_owned()),
                        ("subject_id".to_owned(), "subject_id".to_owned()),
                    ],
                    "RESTRICT".to_owned(),
                ),
                (
                    "outcomes".to_owned(),
                    vec![
                        ("source_outcome_id".to_owned(), "id".to_owned()),
                        ("subject_id".to_owned(), "subject_id".to_owned()),
                    ],
                    "RESTRICT".to_owned(),
                ),
                (
                    "beliefs".to_owned(),
                    vec![
                        ("target_belief_id".to_owned(), "id".to_owned()),
                        ("subject_id".to_owned(), "subject_id".to_owned()),
                    ],
                    "RESTRICT".to_owned(),
                ),
                (
                    "belief_revisions".to_owned(),
                    vec![
                        ("target_belief_revision_id".to_owned(), "id".to_owned()),
                        ("target_belief_id".to_owned(), "belief_id".to_owned()),
                    ],
                    "RESTRICT".to_owned(),
                ),
                (
                    "values".to_owned(),
                    vec![
                        ("target_value_id".to_owned(), "id".to_owned()),
                        ("subject_id".to_owned(), "subject_id".to_owned()),
                    ],
                    "RESTRICT".to_owned(),
                ),
                (
                    "value_revisions".to_owned(),
                    vec![
                        ("target_value_revision_id".to_owned(), "id".to_owned()),
                        ("target_value_id".to_owned(), "value_id".to_owned()),
                    ],
                    "RESTRICT".to_owned(),
                ),
            ] {
                assert!(
                    evidence_foreign_keys.contains(&expected),
                    "missing evidence FK {expected:?}"
                );
            }

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
    fn version_two_database_upgrades_through_the_real_third_migration() {
        tauri::async_runtime::block_on(async {
            let database = TempDatabase::open().await;
            database.apply(MIGRATION_0001).await;
            database.apply(MIGRATION_0002).await;
            let version: String =
                sqlx::query_scalar("SELECT value FROM app_metadata WHERE key = 'schema_version'")
                    .fetch_one(&database.pool)
                    .await
                    .unwrap();
            assert_eq!(version, "2");

            database.apply(MIGRATION_0003).await;
            assert_task004_tables_exist(&database.pool).await;
            assert_task005_tables_exist(&database.pool).await;
            let version: String =
                sqlx::query_scalar("SELECT value FROM app_metadata WHERE key = 'schema_version'")
                    .fetch_one(&database.pool)
                    .await
                    .unwrap();
            assert_eq!(version, "3");

            database.close().await;
        });
    }

    #[test]
    fn version_three_database_upgrades_through_the_real_fourth_migration() {
        tauri::async_runtime::block_on(async {
            let database = TempDatabase::open().await;
            database.apply(MIGRATION_0001).await;
            database.apply(MIGRATION_0002).await;
            database.apply(MIGRATION_0003).await;
            sqlx::query(
                "INSERT INTO self_subjects (id, display_name, created_at_ms) \
                 VALUES ('preserved-subject', 'Preserved', 1)",
            )
            .execute(&database.pool)
            .await
            .unwrap();
            let before: Vec<String> = sqlx::query_scalar(
                "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
            )
            .fetch_all(&database.pool)
            .await
            .unwrap();
            let version: String =
                sqlx::query_scalar("SELECT value FROM app_metadata WHERE key = 'schema_version'")
                    .fetch_one(&database.pool)
                    .await
                    .unwrap();
            assert_eq!(version, "3");

            database.apply(MIGRATION_0004).await;
            let after: Vec<String> = sqlx::query_scalar(
                "SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name",
            )
            .fetch_all(&database.pool)
            .await
            .unwrap();
            let added = after
                .iter()
                .filter(|table| !before.contains(table))
                .cloned()
                .collect::<Vec<_>>();
            assert_eq!(added, ["evidence_links"]);
            let display_name: String =
                sqlx::query_scalar("SELECT display_name FROM self_subjects WHERE id = ?")
                    .bind("preserved-subject")
                    .fetch_one(&database.pool)
                    .await
                    .unwrap();
            assert_eq!(display_name, "Preserved");
            let version: String =
                sqlx::query_scalar("SELECT value FROM app_metadata WHERE key = 'schema_version'")
                    .fetch_one(&database.pool)
                    .await
                    .unwrap();
            assert_eq!(version, "4");

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
    fn real_third_migration_failure_is_surfaced_and_does_not_report_version_three() {
        tauri::async_runtime::block_on(async {
            let database = TempDatabase::open().await;
            database.apply(MIGRATION_0001).await;
            database.apply(MIGRATION_0002).await;
            sqlx::query("CREATE TABLE memories (id TEXT PRIMARY KEY)")
                .execute(&database.pool)
                .await
                .unwrap();

            let failure = sqlx::raw_sql(MIGRATION_0003).execute(&database.pool).await;
            assert!(failure.is_err());
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
    fn real_fourth_migration_failure_is_surfaced_and_does_not_report_version_four() {
        tauri::async_runtime::block_on(async {
            let database = TempDatabase::open().await;
            database.apply(MIGRATION_0001).await;
            database.apply(MIGRATION_0002).await;
            database.apply(MIGRATION_0003).await;
            sqlx::query("CREATE TABLE evidence_links (id TEXT PRIMARY KEY)")
                .execute(&database.pool)
                .await
                .unwrap();

            let failure = sqlx::raw_sql(MIGRATION_0004).execute(&database.pool).await;
            assert!(failure.is_err());
            let version: String =
                sqlx::query_scalar("SELECT value FROM app_metadata WHERE key = 'schema_version'")
                    .fetch_one(&database.pool)
                    .await
                    .unwrap();
            assert_eq!(version, "3");

            database.close().await;
        });
    }

    #[test]
    fn fourth_migration_enforces_closed_matrix_subject_ownership_and_duplicate_policy() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            seed_evidence_records(&database.pool).await;
            let sources = [
                ("Observation", "observation-a"),
                ("Thought", "thought-a"),
                ("Emotion", "emotion-a"),
                ("Situation", "situation-a"),
                ("Memory", "memory-a"),
                ("Decision", "decision-a"),
                ("Outcome", "outcome-a"),
            ];
            let relationships = ["Supports", "Contradicts", "Complicates", "Contextualizes"];

            for (source_index, (source_kind, source_id)) in sources.iter().enumerate() {
                for (target_index, target_kind) in
                    ["BeliefRevision", "ValueRevision"].iter().enumerate()
                {
                    for (relationship_index, relationship) in relationships.iter().enumerate() {
                        let id =
                            format!("matrix-{source_index}-{target_index}-{relationship_index}");
                        let result = EvidenceInsert::new(
                            id,
                            "subject-a",
                            source_kind,
                            source_id,
                            target_kind,
                            relationship,
                        )
                        .execute(&database.pool)
                        .await;
                        let allowed = !matches!(*source_kind, "Situation" | "Emotion")
                            || *relationship == "Contextualizes";
                        assert_eq!(
                            result.is_ok(),
                            allowed,
                            "unexpected matrix result for {source_kind}/{relationship}/{target_kind}"
                        );
                    }
                }
            }

            for (source_index, (source_kind, source_id)) in sources.iter().enumerate() {
                let relationship = if matches!(*source_kind, "Situation" | "Emotion") {
                    "Contextualizes"
                } else {
                    "Supports"
                };
                for (target_index, target_kind) in
                    ["BeliefRevision", "ValueRevision"].iter().enumerate()
                {
                    let result = EvidenceInsert::new(
                        format!("cross-source-{source_index}-{target_index}"),
                        "subject-b",
                        source_kind,
                        source_id,
                        target_kind,
                        relationship,
                    )
                    .execute(&database.pool)
                    .await;
                    assert!(result.is_err(), "cross-subject {source_kind} must fail");
                }
            }

            for target_kind in ["BeliefRevision", "ValueRevision"] {
                let mut cross_target = EvidenceInsert::new(
                    format!("cross-target-{target_kind}"),
                    "subject-b",
                    "Observation",
                    "observation-b",
                    target_kind,
                    "Supports",
                );
                if target_kind == "BeliefRevision" {
                    cross_target.target_belief_id = Some("belief-a".to_owned());
                    cross_target.target_belief_revision_id = Some("belief-revision-a".to_owned());
                } else {
                    cross_target.target_value_id = Some("value-a".to_owned());
                    cross_target.target_value_revision_id = Some("value-revision-a".to_owned());
                }
                assert!(cross_target.execute(&database.pool).await.is_err());
            }

            let semantic_count_before: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM evidence_links
                 WHERE subject_id = 'subject-a'
                   AND source_kind = 'Observation'
                   AND source_observation_id = 'observation-a'
                   AND relationship_kind = 'Supports'
                   AND target_kind = 'BeliefRevision'
                   AND target_belief_revision_id = 'belief-revision-a'
                   AND provenance = 'UserAuthored'",
            )
            .fetch_one(&database.pool)
            .await
            .unwrap();
            for id in ["duplicate-assertion-1", "duplicate-assertion-2"] {
                EvidenceInsert::new(
                    id,
                    "subject-a",
                    "Observation",
                    "observation-a",
                    "BeliefRevision",
                    "Supports",
                )
                .execute(&database.pool)
                .await
                .unwrap();
            }
            let semantic_count_after: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM evidence_links
                 WHERE subject_id = 'subject-a'
                   AND source_kind = 'Observation'
                   AND source_observation_id = 'observation-a'
                   AND relationship_kind = 'Supports'
                   AND target_kind = 'BeliefRevision'
                   AND target_belief_revision_id = 'belief-revision-a'
                   AND provenance = 'UserAuthored'",
            )
            .fetch_one(&database.pool)
            .await
            .unwrap();
            assert_eq!(semantic_count_after, semantic_count_before + 2);

            let mut with_note = EvidenceInsert::new(
                "valid-user-note",
                "subject-a",
                "Memory",
                "memory-a",
                "ValueRevision",
                "Complicates",
            );
            with_note.user_note = Some("用户说明 with mixed Unicode 🌱".to_owned());
            with_note.execute(&database.pool).await.unwrap();

            database.close().await;
        });
    }

    #[test]
    fn fourth_migration_rejects_invalid_shapes_references_and_closed_values() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            seed_evidence_records(&database.pool).await;
            let base = EvidenceInsert::new(
                "invalid-fixture",
                "subject-a",
                "Observation",
                "observation-a",
                "BeliefRevision",
                "Supports",
            );

            let mut zero_sources = base.clone();
            zero_sources.source_observation_id = None;
            assert!(zero_sources.execute(&database.pool).await.is_err());

            let mut multiple_sources = base.clone();
            multiple_sources.source_thought_id = Some("thought-a".to_owned());
            assert!(multiple_sources.execute(&database.pool).await.is_err());

            let mut mismatched_source = base.clone();
            mismatched_source.source_kind = "Thought".to_owned();
            assert!(mismatched_source.execute(&database.pool).await.is_err());

            let mut unknown_source = base.clone();
            unknown_source.source_kind = "PersonReference".to_owned();
            assert!(unknown_source.execute(&database.pool).await.is_err());

            let mut missing_source = base.clone();
            missing_source.source_observation_id = Some("missing-observation".to_owned());
            assert!(missing_source.execute(&database.pool).await.is_err());

            let mut no_target = base.clone();
            no_target.target_belief_id = None;
            no_target.target_belief_revision_id = None;
            assert!(no_target.execute(&database.pool).await.is_err());

            let mut both_targets = base.clone();
            both_targets.target_value_id = Some("value-a".to_owned());
            both_targets.target_value_revision_id = Some("value-revision-a".to_owned());
            assert!(both_targets.execute(&database.pool).await.is_err());

            let mut mismatched_target = base.clone();
            mismatched_target.target_kind = "ValueRevision".to_owned();
            assert!(mismatched_target.execute(&database.pool).await.is_err());

            let mut anchor_only = base.clone();
            anchor_only.target_belief_revision_id = None;
            assert!(anchor_only.execute(&database.pool).await.is_err());

            let mut revision_only = base.clone();
            revision_only.target_belief_id = None;
            assert!(revision_only.execute(&database.pool).await.is_err());

            let mut unknown_target = base.clone();
            unknown_target.target_kind = "Belief".to_owned();
            assert!(unknown_target.execute(&database.pool).await.is_err());

            let mut missing_anchor = base.clone();
            missing_anchor.target_belief_id = Some("missing-belief".to_owned());
            missing_anchor.target_belief_revision_id = Some("missing-revision".to_owned());
            assert!(missing_anchor.execute(&database.pool).await.is_err());

            let mut missing_revision = base.clone();
            missing_revision.target_belief_revision_id = Some("missing-revision".to_owned());
            assert!(missing_revision.execute(&database.pool).await.is_err());

            let mut wrong_belief_anchor = base.clone();
            wrong_belief_anchor.target_belief_id = Some("belief-a-other".to_owned());
            assert!(wrong_belief_anchor.execute(&database.pool).await.is_err());

            let mut wrong_value_anchor = EvidenceInsert::new(
                "wrong-value-anchor",
                "subject-a",
                "Observation",
                "observation-a",
                "ValueRevision",
                "Supports",
            );
            wrong_value_anchor.target_value_id = Some("value-a-other".to_owned());
            assert!(wrong_value_anchor.execute(&database.pool).await.is_err());

            let mut missing_value_anchor = EvidenceInsert::new(
                "missing-value-anchor",
                "subject-a",
                "Observation",
                "observation-a",
                "ValueRevision",
                "Supports",
            );
            missing_value_anchor.target_value_id = Some("missing-value".to_owned());
            missing_value_anchor.target_value_revision_id = Some("missing-revision".to_owned());
            assert!(missing_value_anchor.execute(&database.pool).await.is_err());

            let mut missing_value_revision = EvidenceInsert::new(
                "missing-value-revision",
                "subject-a",
                "Observation",
                "observation-a",
                "ValueRevision",
                "Supports",
            );
            missing_value_revision.target_value_revision_id = Some("missing-revision".to_owned());
            assert!(missing_value_revision
                .execute(&database.pool)
                .await
                .is_err());

            let mut unknown_relationship = base.clone();
            unknown_relationship.relationship_kind = "Implies".to_owned();
            assert!(unknown_relationship.execute(&database.pool).await.is_err());

            let mut unknown_provenance = base.clone();
            unknown_provenance.provenance = "SystemProposed".to_owned();
            assert!(unknown_provenance.execute(&database.pool).await.is_err());

            for (index, note) in ["", "   ", "\t\r\n"].iter().enumerate() {
                let mut blank_note = base.clone();
                blank_note.id = format!("blank-note-{index}");
                blank_note.user_note = Some((*note).to_owned());
                assert!(blank_note.execute(&database.pool).await.is_err());
            }

            database.close().await;
        });
    }

    #[test]
    fn fourth_migration_restricts_deletion_of_referenced_evidence_records() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            seed_evidence_records(&database.pool).await;
            EvidenceInsert::new(
                "restrict-belief-link",
                "subject-a",
                "Observation",
                "observation-a",
                "BeliefRevision",
                "Supports",
            )
            .execute(&database.pool)
            .await
            .unwrap();
            EvidenceInsert::new(
                "restrict-value-link",
                "subject-a",
                "Memory",
                "memory-a",
                "ValueRevision",
                "Contextualizes",
            )
            .execute(&database.pool)
            .await
            .unwrap();

            for (table, id) in [
                ("observations", "observation-a"),
                ("memories", "memory-a"),
                ("belief_revisions", "belief-revision-a"),
                ("value_revisions", "value-revision-a"),
                ("beliefs", "belief-a"),
                ("values", "value-a"),
                ("self_subjects", "subject-a"),
            ] {
                let sql = format!("DELETE FROM \"{table}\" WHERE id = ?");
                assert!(sqlx::query(&sql)
                    .bind(id)
                    .execute(&database.pool)
                    .await
                    .is_err());
            }

            database.close().await;
        });
    }

    #[test]
    fn evidence_repository_round_trips_every_source_both_targets_and_optional_unicode_notes() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            seed_evidence_records(&database.pool).await;
            let repository = repository(&database);
            let links = [
                evidence_link(
                    "roundtrip-observation",
                    "subject-a",
                    crate::domain::EvidenceSource::Observation(
                        crate::domain::ObservationId::new("observation-a").unwrap(),
                    ),
                    belief_evidence_target("belief-a", "belief-revision-a"),
                    crate::domain::EvidenceRelationKind::Supports,
                    None,
                ),
                evidence_link(
                    "roundtrip-thought",
                    "subject-a",
                    crate::domain::EvidenceSource::Thought(
                        crate::domain::ThoughtId::new("thought-a").unwrap(),
                    ),
                    value_evidence_target("value-a", "value-revision-a"),
                    crate::domain::EvidenceRelationKind::Contradicts,
                    Some("Explicit user-authored tension."),
                ),
                evidence_link(
                    "roundtrip-emotion",
                    "subject-a",
                    crate::domain::EvidenceSource::Emotion(
                        crate::domain::EmotionId::new("emotion-a").unwrap(),
                    ),
                    belief_evidence_target("belief-a", "belief-revision-a"),
                    crate::domain::EvidenceRelationKind::Contextualizes,
                    Some("用户说明：这是感受背景。"),
                ),
                evidence_link(
                    "roundtrip-situation",
                    "subject-a",
                    crate::domain::EvidenceSource::Situation(
                        crate::domain::SituationId::new("situation-a").unwrap(),
                    ),
                    value_evidence_target("value-a", "value-revision-a"),
                    crate::domain::EvidenceRelationKind::Contextualizes,
                    Some("Context 背景 🌱"),
                ),
                evidence_link(
                    "roundtrip-memory",
                    "subject-a",
                    crate::domain::EvidenceSource::Memory(
                        crate::domain::MemoryId::new("memory-a").unwrap(),
                    ),
                    belief_evidence_target("belief-a", "belief-revision-a"),
                    crate::domain::EvidenceRelationKind::Complicates,
                    None,
                ),
                evidence_link(
                    "roundtrip-decision",
                    "subject-a",
                    crate::domain::EvidenceSource::Decision(
                        crate::domain::DecisionId::new("decision-a").unwrap(),
                    ),
                    value_evidence_target("value-a", "value-revision-a"),
                    crate::domain::EvidenceRelationKind::Supports,
                    None,
                ),
                evidence_link(
                    "roundtrip-outcome",
                    "subject-a",
                    crate::domain::EvidenceSource::Outcome(
                        crate::domain::OutcomeId::new("outcome-a").unwrap(),
                    ),
                    belief_evidence_target("belief-a", "belief-revision-a"),
                    crate::domain::EvidenceRelationKind::Contradicts,
                    None,
                ),
            ];

            for (offset, link) in links.iter().enumerate() {
                repository
                    .create_evidence_link(link, 100 + i64::try_from(offset).unwrap())
                    .await
                    .unwrap();
                assert_eq!(
                    repository.load_evidence_link(link.id()).await.unwrap(),
                    *link
                );
            }

            assert!(matches!(
                repository
                    .load_evidence_link(
                        &crate::domain::EvidenceLinkId::new("missing-link").unwrap()
                    )
                    .await,
                Err(PersistenceError::NotFound {
                    entity: "EvidenceLink",
                    ..
                })
            ));

            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn evidence_repository_lists_exact_revisions_in_deterministic_storage_order() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            seed_evidence_records(&database.pool).await;
            sqlx::raw_sql(
                "INSERT INTO belief_revisions
                    (id, belief_id, revision_number, proposition, endorsement, change_note, origin, created_at_ms)
                 VALUES ('belief-revision-a-2', 'belief-a', 2, 'Belief A revised', NULL, NULL, 'UserUpdate', 29);
                 INSERT INTO value_revisions
                    (id, value_id, revision_number, label, importance, change_note, origin, created_at_ms)
                 VALUES ('value-revision-a-2', 'value-a', 2, 'Value A revised', NULL, NULL, 'UserUpdate', 30);",
            )
            .execute(&database.pool)
            .await
            .unwrap();
            let repository = repository(&database);

            for (id, timestamp) in [
                ("belief-order-z", 10),
                ("belief-order-a", 10),
                ("belief-order-m", 20),
            ] {
                let link = evidence_link(
                    id,
                    "subject-a",
                    crate::domain::EvidenceSource::Observation(
                        crate::domain::ObservationId::new("observation-a").unwrap(),
                    ),
                    belief_evidence_target("belief-a", "belief-revision-a"),
                    crate::domain::EvidenceRelationKind::Supports,
                    None,
                );
                repository
                    .create_evidence_link(&link, timestamp)
                    .await
                    .unwrap();
            }
            let later_belief_link = evidence_link(
                "belief-later-revision-only",
                "subject-a",
                crate::domain::EvidenceSource::Memory(
                    crate::domain::MemoryId::new("memory-a").unwrap(),
                ),
                belief_evidence_target("belief-a", "belief-revision-a-2"),
                crate::domain::EvidenceRelationKind::Complicates,
                None,
            );
            repository
                .create_evidence_link(&later_belief_link, 5)
                .await
                .unwrap();

            for (id, timestamp) in [("value-order-z", 30), ("value-order-a", 30)] {
                let link = evidence_link(
                    id,
                    "subject-a",
                    crate::domain::EvidenceSource::Decision(
                        crate::domain::DecisionId::new("decision-a").unwrap(),
                    ),
                    value_evidence_target("value-a", "value-revision-a"),
                    crate::domain::EvidenceRelationKind::Contextualizes,
                    None,
                );
                repository
                    .create_evidence_link(&link, timestamp)
                    .await
                    .unwrap();
            }
            let later_value_link = evidence_link(
                "value-later-revision-only",
                "subject-a",
                crate::domain::EvidenceSource::Outcome(
                    crate::domain::OutcomeId::new("outcome-a").unwrap(),
                ),
                value_evidence_target("value-a", "value-revision-a-2"),
                crate::domain::EvidenceRelationKind::Supports,
                None,
            );
            repository
                .create_evidence_link(&later_value_link, 5)
                .await
                .unwrap();

            let belief_links = repository
                .load_evidence_links_for_belief_revision(
                    &crate::domain::BeliefRevisionId::new("belief-revision-a").unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                belief_links
                    .iter()
                    .map(|link| link.id().as_str())
                    .collect::<Vec<_>>(),
                ["belief-order-a", "belief-order-z", "belief-order-m"]
            );
            assert_eq!(
                repository
                    .load_evidence_links_for_belief_revision(
                        &crate::domain::BeliefRevisionId::new("belief-revision-a-2").unwrap(),
                    )
                    .await
                    .unwrap(),
                [later_belief_link]
            );
            assert!(repository
                .load_evidence_links_for_belief_revision(
                    &crate::domain::BeliefRevisionId::new("belief-revision-a-other").unwrap(),
                )
                .await
                .unwrap()
                .is_empty());

            let value_links = repository
                .load_evidence_links_for_value_revision(
                    &crate::domain::ValueRevisionId::new("value-revision-a").unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                value_links
                    .iter()
                    .map(|link| link.id().as_str())
                    .collect::<Vec<_>>(),
                ["value-order-a", "value-order-z"]
            );
            assert_eq!(
                repository
                    .load_evidence_links_for_value_revision(
                        &crate::domain::ValueRevisionId::new("value-revision-a-2").unwrap(),
                    )
                    .await
                    .unwrap(),
                [later_value_link]
            );
            assert!(repository
                .load_evidence_links_for_value_revision(
                    &crate::domain::ValueRevisionId::new("value-revision-a-other").unwrap(),
                )
                .await
                .unwrap()
                .is_empty());

            assert!(matches!(
                repository
                    .load_evidence_links_for_belief_revision(
                        &crate::domain::BeliefRevisionId::new("missing-belief-revision").unwrap(),
                    )
                    .await,
                Err(PersistenceError::NotFound {
                    entity: "BeliefRevision",
                    ..
                })
            ));
            assert!(matches!(
                repository
                    .load_evidence_links_for_value_revision(
                        &crate::domain::ValueRevisionId::new("missing-value-revision").unwrap(),
                    )
                    .await,
                Err(PersistenceError::NotFound {
                    entity: "ValueRevision",
                    ..
                })
            ));

            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn evidence_repository_surfaces_constraints_and_allows_equivalent_assertions() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            seed_evidence_records(&database.pool).await;
            let repository = repository(&database);
            let valid = evidence_link(
                "repository-constraint-link",
                "subject-a",
                crate::domain::EvidenceSource::Observation(
                    crate::domain::ObservationId::new("observation-a").unwrap(),
                ),
                belief_evidence_target("belief-a", "belief-revision-a"),
                crate::domain::EvidenceRelationKind::Supports,
                Some("The same assertion may have another distinct record."),
            );
            repository.create_evidence_link(&valid, 1).await.unwrap();
            assert!(matches!(
                repository.create_evidence_link(&valid, 2).await,
                Err(PersistenceError::ConstraintViolation {
                    operation: "create_evidence_link",
                    ..
                })
            ));

            let equivalent = evidence_link(
                "repository-equivalent-link",
                "subject-a",
                valid.source().clone(),
                valid.target().clone(),
                valid.relationship(),
                valid.user_note(),
            );
            repository
                .create_evidence_link(&equivalent, 2)
                .await
                .unwrap();

            let invalid_links = [
                evidence_link(
                    "missing-source-link",
                    "subject-a",
                    crate::domain::EvidenceSource::Observation(
                        crate::domain::ObservationId::new("missing-observation").unwrap(),
                    ),
                    belief_evidence_target("belief-a", "belief-revision-a"),
                    crate::domain::EvidenceRelationKind::Supports,
                    None,
                ),
                evidence_link(
                    "cross-source-link",
                    "subject-b",
                    crate::domain::EvidenceSource::Observation(
                        crate::domain::ObservationId::new("observation-a").unwrap(),
                    ),
                    belief_evidence_target("belief-b", "belief-revision-b"),
                    crate::domain::EvidenceRelationKind::Supports,
                    None,
                ),
                evidence_link(
                    "cross-target-link",
                    "subject-b",
                    crate::domain::EvidenceSource::Observation(
                        crate::domain::ObservationId::new("observation-b").unwrap(),
                    ),
                    belief_evidence_target("belief-a", "belief-revision-a"),
                    crate::domain::EvidenceRelationKind::Supports,
                    None,
                ),
                evidence_link(
                    "missing-target-link",
                    "subject-a",
                    crate::domain::EvidenceSource::Observation(
                        crate::domain::ObservationId::new("observation-a").unwrap(),
                    ),
                    belief_evidence_target("missing-belief", "missing-belief-revision"),
                    crate::domain::EvidenceRelationKind::Supports,
                    None,
                ),
                evidence_link(
                    "missing-revision-link",
                    "subject-a",
                    crate::domain::EvidenceSource::Observation(
                        crate::domain::ObservationId::new("observation-a").unwrap(),
                    ),
                    belief_evidence_target("belief-a", "missing-belief-revision"),
                    crate::domain::EvidenceRelationKind::Supports,
                    None,
                ),
                evidence_link(
                    "mismatched-anchor-link",
                    "subject-a",
                    crate::domain::EvidenceSource::Observation(
                        crate::domain::ObservationId::new("observation-a").unwrap(),
                    ),
                    belief_evidence_target("belief-a-other", "belief-revision-a"),
                    crate::domain::EvidenceRelationKind::Supports,
                    None,
                ),
            ];
            for link in invalid_links {
                assert!(matches!(
                    repository.create_evidence_link(&link, 3).await,
                    Err(PersistenceError::ConstraintViolation {
                        operation: "create_evidence_link",
                        ..
                    })
                ));
            }

            let links = repository
                .load_evidence_links_for_belief_revision(
                    &crate::domain::BeliefRevisionId::new("belief-revision-a").unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(links, [valid, equivalent]);

            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn evidence_repository_rejects_operation_connections_with_foreign_keys_disabled() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let mut first = database.pool.acquire().await.unwrap();
            let mut second = database.pool.acquire().await.unwrap();
            for connection in [&mut first, &mut second] {
                sqlx::query("PRAGMA foreign_keys = OFF")
                    .execute(&mut **connection)
                    .await
                    .unwrap();
            }
            drop(first);
            drop(second);

            let repository = repository(&database);
            let link = evidence_link(
                "fk-disabled-evidence",
                "subject-a",
                crate::domain::EvidenceSource::Observation(
                    crate::domain::ObservationId::new("observation-a").unwrap(),
                ),
                belief_evidence_target("belief-a", "belief-revision-a"),
                crate::domain::EvidenceRelationKind::Supports,
                None,
            );
            assert!(matches!(
                repository.create_evidence_link(&link, 1).await,
                Err(PersistenceError::NotReady(_))
            ));

            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn evidence_repository_rejects_every_representative_corrupt_stored_field() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            seed_evidence_records(&database.pool).await;
            let repository = repository(&database);

            let corrupt_ids = [
                "corrupt-subject",
                "corrupt-source-kind",
                "corrupt-source-id",
                "corrupt-source-shape",
                "corrupt-target-kind",
                "corrupt-belief-id",
                "corrupt-belief-revision-id",
                "corrupt-relationship",
                "corrupt-provenance",
                "corrupt-note",
            ];
            for id in corrupt_ids {
                let link = evidence_link(
                    id,
                    "subject-a",
                    crate::domain::EvidenceSource::Observation(
                        crate::domain::ObservationId::new("observation-a").unwrap(),
                    ),
                    belief_evidence_target("belief-a", "belief-revision-a"),
                    crate::domain::EvidenceRelationKind::Supports,
                    None,
                );
                repository.create_evidence_link(&link, 10).await.unwrap();
            }
            for id in ["corrupt-value-id", "corrupt-value-revision-id"] {
                let link = evidence_link(
                    id,
                    "subject-a",
                    crate::domain::EvidenceSource::Thought(
                        crate::domain::ThoughtId::new("thought-a").unwrap(),
                    ),
                    value_evidence_target("value-a", "value-revision-a"),
                    crate::domain::EvidenceRelationKind::Complicates,
                    None,
                );
                repository.create_evidence_link(&link, 10).await.unwrap();
            }
            let illegal_triplet = evidence_link(
                "corrupt-illegal-triplet",
                "subject-a",
                crate::domain::EvidenceSource::Situation(
                    crate::domain::SituationId::new("situation-a").unwrap(),
                ),
                belief_evidence_target("belief-a", "belief-revision-a"),
                crate::domain::EvidenceRelationKind::Contextualizes,
                None,
            );
            repository
                .create_evidence_link(&illegal_triplet, 10)
                .await
                .unwrap();
            let invalid_id = evidence_link(
                "corrupt-id-before",
                "subject-a",
                crate::domain::EvidenceSource::Observation(
                    crate::domain::ObservationId::new("observation-a").unwrap(),
                ),
                belief_evidence_target("belief-a-other", "belief-revision-a-other"),
                crate::domain::EvidenceRelationKind::Supports,
                None,
            );
            repository
                .create_evidence_link(&invalid_id, 10)
                .await
                .unwrap();

            apply_test_only_evidence_corruption(
                &database.pool,
                "UPDATE evidence_links SET subject_id = ' ' WHERE id = 'corrupt-subject';
                 UPDATE evidence_links SET source_kind = 'PersonReference' WHERE id = 'corrupt-source-kind';
                 UPDATE evidence_links SET source_observation_id = ' ' WHERE id = 'corrupt-source-id';
                 UPDATE evidence_links SET source_kind = 'Thought' WHERE id = 'corrupt-source-shape';
                 UPDATE evidence_links SET target_kind = 'Belief' WHERE id = 'corrupt-target-kind';
                 UPDATE evidence_links SET target_belief_id = ' ' WHERE id = 'corrupt-belief-id';
                 UPDATE evidence_links SET target_belief_revision_id = ' ' WHERE id = 'corrupt-belief-revision-id';
                 UPDATE evidence_links SET target_value_id = ' ' WHERE id = 'corrupt-value-id';
                 UPDATE evidence_links SET target_value_revision_id = ' ' WHERE id = 'corrupt-value-revision-id';
                 UPDATE evidence_links SET relationship_kind = 'Implies' WHERE id = 'corrupt-relationship';
                 UPDATE evidence_links SET provenance = 'SystemProposed' WHERE id = 'corrupt-provenance';
                 UPDATE evidence_links SET user_note = ' ' WHERE id = 'corrupt-note';
                 UPDATE evidence_links SET relationship_kind = 'Supports' WHERE id = 'corrupt-illegal-triplet';
                 UPDATE evidence_links SET id = ' ' WHERE id = 'corrupt-id-before';",
            )
            .await;

            for id in corrupt_ids.into_iter().chain([
                "corrupt-value-id",
                "corrupt-value-revision-id",
                "corrupt-illegal-triplet",
            ]) {
                assert_domain_reconstruction(
                    repository
                        .load_evidence_link(&crate::domain::EvidenceLinkId::new(id).unwrap())
                        .await,
                );
            }
            assert_domain_reconstruction(
                repository
                    .load_evidence_links_for_belief_revision(
                        &crate::domain::BeliefRevisionId::new("belief-revision-a-other").unwrap(),
                    )
                    .await,
            );

            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn evidence_repository_collection_loads_fail_when_any_row_is_corrupt() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            seed_evidence_records(&database.pool).await;
            let repository = repository(&database);

            for (id, target) in [
                (
                    "belief-collection-valid",
                    belief_evidence_target("belief-a", "belief-revision-a"),
                ),
                (
                    "belief-collection-corrupt",
                    belief_evidence_target("belief-a", "belief-revision-a"),
                ),
                (
                    "value-collection-valid",
                    value_evidence_target("value-a", "value-revision-a"),
                ),
                (
                    "value-collection-corrupt",
                    value_evidence_target("value-a", "value-revision-a"),
                ),
            ] {
                let link = evidence_link(
                    id,
                    "subject-a",
                    crate::domain::EvidenceSource::Memory(
                        crate::domain::MemoryId::new("memory-a").unwrap(),
                    ),
                    target,
                    crate::domain::EvidenceRelationKind::Contextualizes,
                    Some("Valid before controlled corruption"),
                );
                repository.create_evidence_link(&link, 10).await.unwrap();
            }
            apply_test_only_evidence_corruption(
                &database.pool,
                "UPDATE evidence_links SET user_note = ' ' WHERE id = 'belief-collection-corrupt';
                 UPDATE evidence_links SET provenance = 'SystemInferred' WHERE id = 'value-collection-corrupt';",
            )
            .await;

            assert_domain_reconstruction(
                repository
                    .load_evidence_links_for_belief_revision(
                        &crate::domain::BeliefRevisionId::new("belief-revision-a").unwrap(),
                    )
                    .await,
            );
            assert_domain_reconstruction(
                repository
                    .load_evidence_links_for_value_revision(
                        &crate::domain::ValueRevisionId::new("value-revision-a").unwrap(),
                    )
                    .await,
            );

            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn evidence_links_and_exact_revision_lists_survive_close_and_reopen() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            seed_evidence_records(&database.pool).await;
            let repository_before_reopen = repository(&database);

            let belief_revision_two = repository_before_reopen
                .append_belief_revision(
                    &crate::domain::BeliefId::new("belief-a").unwrap(),
                    AppendBeliefRevisionInput::new(
                        crate::domain::BeliefRevisionId::new("belief-revision-a-2").unwrap(),
                        "Belief A revised",
                        None,
                        None,
                        crate::domain::RevisionOrigin::UserUpdate,
                    ),
                    40,
                )
                .await
                .unwrap();
            let value_revision_two = repository_before_reopen
                .append_value_revision(
                    &crate::domain::ValueId::new("value-a").unwrap(),
                    AppendValueRevisionInput::new(
                        crate::domain::ValueRevisionId::new("value-revision-a-2").unwrap(),
                        "Value A revised",
                        None,
                        None,
                        crate::domain::RevisionOrigin::UserUpdate,
                    ),
                    41,
                )
                .await
                .unwrap();

            let duplicate_a = evidence_link(
                "durable-duplicate-a",
                "subject-a",
                crate::domain::EvidenceSource::Observation(
                    crate::domain::ObservationId::new("observation-a").unwrap(),
                ),
                belief_evidence_target("belief-a", "belief-revision-a"),
                crate::domain::EvidenceRelationKind::Supports,
                None,
            );
            let duplicate_z = evidence_link(
                "durable-duplicate-z",
                "subject-a",
                duplicate_a.source().clone(),
                duplicate_a.target().clone(),
                duplicate_a.relationship(),
                duplicate_a.user_note(),
            );
            let unicode_note = "用户说明：Memory complicates this belief — 保留原文 🌱";
            let belief_context = evidence_link(
                "durable-memory-m",
                "subject-a",
                crate::domain::EvidenceSource::Memory(
                    crate::domain::MemoryId::new("memory-a").unwrap(),
                ),
                belief_evidence_target("belief-a", "belief-revision-a"),
                crate::domain::EvidenceRelationKind::Complicates,
                Some(unicode_note),
            );
            let value_link = evidence_link(
                "durable-value-link",
                "subject-a",
                crate::domain::EvidenceSource::Decision(
                    crate::domain::DecisionId::new("decision-a").unwrap(),
                ),
                value_evidence_target("value-a", "value-revision-a"),
                crate::domain::EvidenceRelationKind::Contradicts,
                Some("A deliberate choice was in tension with this priority."),
            );

            for (link, created_at_ms) in [
                (&duplicate_z, 50),
                (&duplicate_a, 50),
                (&belief_context, 60),
                (&value_link, 45),
            ] {
                repository_before_reopen
                    .create_evidence_link(link, created_at_ms)
                    .await
                    .unwrap();
            }
            drop(repository_before_reopen);

            let database = database.reopen().await;
            let reopened_repository = repository(&database);

            for expected in [&duplicate_a, &duplicate_z, &belief_context, &value_link] {
                assert_eq!(
                    reopened_repository
                        .load_evidence_link(expected.id())
                        .await
                        .unwrap(),
                    *expected
                );
            }
            assert_eq!(duplicate_a.user_note(), None);
            assert_eq!(
                reopened_repository
                    .load_evidence_link(belief_context.id())
                    .await
                    .unwrap()
                    .user_note(),
                Some(unicode_note)
            );

            let belief_links = reopened_repository
                .load_evidence_links_for_belief_revision(
                    &crate::domain::BeliefRevisionId::new("belief-revision-a").unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                belief_links
                    .iter()
                    .map(|link| link.id().as_str())
                    .collect::<Vec<_>>(),
                [
                    "durable-duplicate-a",
                    "durable-duplicate-z",
                    "durable-memory-m"
                ]
            );
            assert!(reopened_repository
                .load_evidence_links_for_belief_revision(belief_revision_two.id())
                .await
                .unwrap()
                .is_empty());
            assert_eq!(
                reopened_repository
                    .load_evidence_links_for_value_revision(
                        &crate::domain::ValueRevisionId::new("value-revision-a").unwrap(),
                    )
                    .await
                    .unwrap(),
                [value_link]
            );
            assert!(reopened_repository
                .load_evidence_links_for_value_revision(value_revision_two.id())
                .await
                .unwrap()
                .is_empty());

            drop(reopened_repository);
            database.close().await;
        });
    }

    #[test]
    fn third_migration_enforces_lived_experience_ownership_text_and_cardinality() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            sqlx::query(
                "INSERT INTO self_subjects (id, display_name, created_at_ms) VALUES \
                 ('subject-a', 'Subject A', 1), ('subject-b', 'Subject B', 2)",
            )
            .execute(&database.pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO situations (id, subject_id, description, created_at_ms) VALUES \
                 ('situation-a', 'subject-a', 'Context A', 3), \
                 ('situation-b', 'subject-b', 'Context B', 4)",
            )
            .execute(&database.pool)
            .await
            .unwrap();

            sqlx::query(
                "INSERT INTO memories \
                 (id, subject_id, situation_id, description, user_meaning, created_at_ms) \
                 VALUES ('memory-a', 'subject-a', 'situation-a', 'Remembered event', NULL, 5)",
            )
            .execute(&database.pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO memories \
                 (id, subject_id, situation_id, description, user_meaning, created_at_ms) \
                 VALUES ('memory-no-context', 'subject-a', NULL, 'No linked context', \
                 'User-authored meaning', 6)",
            )
            .execute(&database.pool)
            .await
            .unwrap();
            assert!(sqlx::query(
                "INSERT INTO memories \
                 (id, subject_id, situation_id, description, user_meaning, created_at_ms) \
                 VALUES ('memory-cross', 'subject-a', 'situation-b', 'Cross subject', NULL, 7)",
            )
            .execute(&database.pool)
            .await
            .is_err());
            assert!(sqlx::query(
                "INSERT INTO memories \
                 (id, subject_id, situation_id, description, user_meaning, created_at_ms) \
                 VALUES ('memory-missing', 'missing', NULL, 'Missing subject', NULL, 8)",
            )
            .execute(&database.pool)
            .await
            .is_err());
            assert!(sqlx::query(
                "INSERT INTO memories \
                 (id, subject_id, situation_id, description, user_meaning, created_at_ms) \
                 VALUES ('memory-blank', 'subject-a', NULL, ' ', NULL, 9)",
            )
            .execute(&database.pool)
            .await
            .is_err());
            assert!(sqlx::query(
                "INSERT INTO memories \
                 (id, subject_id, situation_id, description, user_meaning, created_at_ms) \
                 VALUES ('memory-empty', 'subject-a', NULL, '', NULL, 9)",
            )
            .execute(&database.pool)
            .await
            .is_err());
            assert!(sqlx::query(
                "INSERT INTO memories \
                 (id, subject_id, situation_id, description, user_meaning, created_at_ms) \
                 VALUES ('memory-meaning', 'subject-a', NULL, 'Valid description', ' ', 10)",
            )
            .execute(&database.pool)
            .await
            .is_err());
            assert!(sqlx::query(
                "INSERT INTO memories \
                 (id, subject_id, situation_id, description, user_meaning, created_at_ms) \
                 VALUES ('memory-empty-meaning', 'subject-a', NULL, 'Valid description', '', 10)",
            )
            .execute(&database.pool)
            .await
            .is_err());
            assert!(sqlx::query(
                "INSERT INTO memories \
                 (id, subject_id, situation_id, description, user_meaning, created_at_ms) \
                 VALUES (' ', 'subject-a', NULL, 'Blank id', NULL, 11)",
            )
            .execute(&database.pool)
            .await
            .is_err());

            sqlx::query(
                "INSERT INTO decisions \
                 (id, subject_id, situation_id, description, created_at_ms) \
                 VALUES ('decision-a', 'subject-a', 'situation-a', 'A decision', 12)",
            )
            .execute(&database.pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO decisions \
                 (id, subject_id, situation_id, description, created_at_ms) \
                 VALUES ('decision-no-context', 'subject-a', NULL, 'No linked context', 13)",
            )
            .execute(&database.pool)
            .await
            .unwrap();
            assert!(sqlx::query(
                "INSERT INTO decisions \
                 (id, subject_id, situation_id, description, created_at_ms) \
                 VALUES ('decision-cross', 'subject-a', 'situation-b', 'Cross subject', 14)",
            )
            .execute(&database.pool)
            .await
            .is_err());
            assert!(sqlx::query(
                "INSERT INTO decisions \
                 (id, subject_id, situation_id, description, created_at_ms) \
                 VALUES ('decision-blank', 'subject-a', NULL, '', 15)",
            )
            .execute(&database.pool)
            .await
            .is_err());
            assert!(sqlx::query(
                "INSERT INTO decisions \
                 (id, subject_id, situation_id, description, created_at_ms) \
                 VALUES ('decision-whitespace', 'subject-a', NULL, ' ', 15)",
            )
            .execute(&database.pool)
            .await
            .is_err());
            assert!(sqlx::query(
                "INSERT INTO decisions \
                 (id, subject_id, situation_id, description, created_at_ms) \
                 VALUES (' ', 'subject-a', NULL, 'Blank id', 16)",
            )
            .execute(&database.pool)
            .await
            .is_err());

            for (id, created_at_ms) in [("outcome-a", 17_i64), ("outcome-b", 18_i64)] {
                sqlx::query(
                    "INSERT INTO outcomes \
                     (id, subject_id, decision_id, description, created_at_ms) \
                     VALUES (?, 'subject-a', 'decision-a', 'Reported outcome', ?)",
                )
                .bind(id)
                .bind(created_at_ms)
                .execute(&database.pool)
                .await
                .unwrap();
            }
            assert!(sqlx::query(
                "INSERT INTO outcomes \
                 (id, subject_id, decision_id, description, created_at_ms) \
                 VALUES ('outcome-missing', 'subject-a', 'missing', 'Missing decision', 19)",
            )
            .execute(&database.pool)
            .await
            .is_err());
            assert!(sqlx::query(
                "INSERT INTO outcomes \
                 (id, subject_id, decision_id, description, created_at_ms) \
                 VALUES ('outcome-cross', 'subject-b', 'decision-a', 'Cross subject', 20)",
            )
            .execute(&database.pool)
            .await
            .is_err());
            assert!(sqlx::query(
                "INSERT INTO outcomes \
                 (id, subject_id, decision_id, description, created_at_ms) \
                 VALUES ('outcome-blank', 'subject-a', 'decision-a', ' ', 21)",
            )
            .execute(&database.pool)
            .await
            .is_err());
            assert!(sqlx::query(
                "INSERT INTO outcomes \
                 (id, subject_id, decision_id, description, created_at_ms) \
                 VALUES ('outcome-empty', 'subject-a', 'decision-a', '', 21)",
            )
            .execute(&database.pool)
            .await
            .is_err());
            assert!(sqlx::query(
                "INSERT INTO outcomes \
                 (id, subject_id, decision_id, description, created_at_ms) \
                 VALUES (' ', 'subject-a', 'decision-a', 'Blank id', 22)",
            )
            .execute(&database.pool)
            .await
            .is_err());

            let outcome_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM outcomes WHERE decision_id = 'decision-a'",
            )
            .fetch_one(&database.pool)
            .await
            .unwrap();
            assert_eq!(outcome_count, 2);
            assert!(sqlx::query("DELETE FROM decisions WHERE id = 'decision-a'")
                .execute(&database.pool)
                .await
                .is_err());
            assert!(
                sqlx::query("DELETE FROM situations WHERE id = 'situation-a'")
                    .execute(&database.pool)
                    .await
                    .is_err()
            );
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
    fn lived_experience_create_rejects_an_operation_connection_with_foreign_keys_disabled() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let mut first = database.pool.acquire().await.unwrap();
            let mut second = database.pool.acquire().await.unwrap();
            for connection in [&mut first, &mut second] {
                sqlx::query("PRAGMA foreign_keys = OFF")
                    .execute(&mut **connection)
                    .await
                    .unwrap();
            }
            drop(first);
            drop(second);

            let repository = repository(&database);
            let memory = crate::domain::Memory::new(
                crate::domain::MemoryId::new("fk-disabled-memory").unwrap(),
                crate::domain::SelfSubjectId::new("fk-disabled-subject").unwrap(),
                None,
                "The operation must fail before attempting this insert.",
                None,
            )
            .unwrap();
            assert!(matches!(
                repository.create_memory(&memory, 1).await,
                Err(PersistenceError::NotReady(_))
            ));

            drop(repository);
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
    fn lived_experience_records_round_trip_unicode_and_optional_fields() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let repository = repository(&database);
            let subject = create_subject_fixture(&repository, "lived-subject").await;
            let situation =
                create_situation_fixture(&repository, &subject, "lived-situation").await;

            let memories = [
                crate::domain::Memory::new(
                    crate::domain::MemoryId::new("memory-none").unwrap(),
                    subject.id().clone(),
                    None,
                    "I remembered the first day at school.",
                    None,
                )
                .unwrap(),
                crate::domain::Memory::new(
                    crate::domain::MemoryId::new("memory-situation").unwrap(),
                    subject.id().clone(),
                    Some(situation.id().clone()),
                    "那天我第一次独自回家。",
                    None,
                )
                .unwrap(),
                crate::domain::Memory::new(
                    crate::domain::MemoryId::new("memory-meaning").unwrap(),
                    subject.id().clone(),
                    None,
                    "A difficult conversation 对话",
                    Some("I learned 我可以 ask for help.".into()),
                )
                .unwrap(),
                crate::domain::Memory::new(
                    crate::domain::MemoryId::new("memory-both").unwrap(),
                    subject.id().clone(),
                    Some(situation.id().clone()),
                    "毕业那天 felt uncertain 🌱",
                    Some("Endings can also be beginnings.".into()),
                )
                .unwrap(),
            ];
            for (offset, memory) in memories.iter().enumerate() {
                repository
                    .create_memory(memory, 10 + i64::try_from(offset).unwrap())
                    .await
                    .unwrap();
                assert_eq!(repository.load_memory(memory.id()).await.unwrap(), *memory);
            }
            let stored_created_at_ms: i64 =
                sqlx::query_scalar("SELECT created_at_ms FROM memories WHERE id = 'memory-both'")
                    .fetch_one(&database.pool)
                    .await
                    .unwrap();
            assert_eq!(stored_created_at_ms, 13);

            let decisions = [
                crate::domain::Decision::new(
                    crate::domain::DecisionId::new("decision-none").unwrap(),
                    subject.id().clone(),
                    None,
                    "I chose to wait before replying.",
                )
                .unwrap(),
                crate::domain::Decision::new(
                    crate::domain::DecisionId::new("decision-situation").unwrap(),
                    subject.id().clone(),
                    Some(situation.id().clone()),
                    "我决定接受 the new role 🚀",
                )
                .unwrap(),
            ];
            for (offset, decision) in decisions.iter().enumerate() {
                repository
                    .create_decision(decision, 20 + i64::try_from(offset).unwrap())
                    .await
                    .unwrap();
                assert_eq!(
                    repository.load_decision(decision.id()).await.unwrap(),
                    *decision
                );
            }

            let outcome = crate::domain::Outcome::new(
                crate::domain::OutcomeId::new("outcome-unicode").unwrap(),
                subject.id().clone(),
                decisions[0].id().clone(),
                "后来我收到 a thoughtful reply 🌏",
            )
            .unwrap();
            repository.create_outcome(&outcome, 30).await.unwrap();
            assert_eq!(
                repository.load_outcome(outcome.id()).await.unwrap(),
                outcome
            );

            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn lived_experience_creates_translate_identity_and_scoped_ownership_constraints() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let repository = repository(&database);
            let subject_a = create_subject_fixture(&repository, "lived-a").await;
            let subject_b = create_subject_fixture(&repository, "lived-b").await;
            let situation_a =
                create_situation_fixture(&repository, &subject_a, "lived-situation-a").await;

            let memory = crate::domain::Memory::new(
                crate::domain::MemoryId::new("constraint-memory").unwrap(),
                subject_a.id().clone(),
                Some(situation_a.id().clone()),
                "A valid memory",
                None,
            )
            .unwrap();
            repository.create_memory(&memory, 10).await.unwrap();
            assert!(matches!(
                repository.create_memory(&memory, 11).await,
                Err(PersistenceError::ConstraintViolation {
                    operation: "create_memory",
                    ..
                })
            ));
            for invalid in [
                crate::domain::Memory::new(
                    crate::domain::MemoryId::new("memory-missing-subject").unwrap(),
                    crate::domain::SelfSubjectId::new("missing-subject").unwrap(),
                    None,
                    "Missing subject",
                    None,
                )
                .unwrap(),
                crate::domain::Memory::new(
                    crate::domain::MemoryId::new("memory-missing-situation").unwrap(),
                    subject_a.id().clone(),
                    Some(crate::domain::SituationId::new("missing-situation").unwrap()),
                    "Missing situation",
                    None,
                )
                .unwrap(),
                crate::domain::Memory::new(
                    crate::domain::MemoryId::new("memory-cross-situation").unwrap(),
                    subject_b.id().clone(),
                    Some(situation_a.id().clone()),
                    "Cross-subject situation",
                    None,
                )
                .unwrap(),
            ] {
                assert!(matches!(
                    repository.create_memory(&invalid, 12).await,
                    Err(PersistenceError::ConstraintViolation {
                        operation: "create_memory",
                        ..
                    })
                ));
            }

            let decision = crate::domain::Decision::new(
                crate::domain::DecisionId::new("constraint-decision").unwrap(),
                subject_a.id().clone(),
                Some(situation_a.id().clone()),
                "A valid decision",
            )
            .unwrap();
            repository.create_decision(&decision, 20).await.unwrap();
            assert!(matches!(
                repository.create_decision(&decision, 21).await,
                Err(PersistenceError::ConstraintViolation {
                    operation: "create_decision",
                    ..
                })
            ));
            for invalid in [
                crate::domain::Decision::new(
                    crate::domain::DecisionId::new("decision-missing-subject").unwrap(),
                    crate::domain::SelfSubjectId::new("missing-subject").unwrap(),
                    None,
                    "Missing subject",
                )
                .unwrap(),
                crate::domain::Decision::new(
                    crate::domain::DecisionId::new("decision-missing-situation").unwrap(),
                    subject_a.id().clone(),
                    Some(crate::domain::SituationId::new("missing-situation").unwrap()),
                    "Missing situation",
                )
                .unwrap(),
                crate::domain::Decision::new(
                    crate::domain::DecisionId::new("decision-cross-situation").unwrap(),
                    subject_b.id().clone(),
                    Some(situation_a.id().clone()),
                    "Cross-subject situation",
                )
                .unwrap(),
            ] {
                assert!(matches!(
                    repository.create_decision(&invalid, 22).await,
                    Err(PersistenceError::ConstraintViolation {
                        operation: "create_decision",
                        ..
                    })
                ));
            }

            let outcome = crate::domain::Outcome::new(
                crate::domain::OutcomeId::new("constraint-outcome").unwrap(),
                subject_a.id().clone(),
                decision.id().clone(),
                "A valid outcome",
            )
            .unwrap();
            repository.create_outcome(&outcome, 30).await.unwrap();
            assert!(matches!(
                repository.create_outcome(&outcome, 31).await,
                Err(PersistenceError::ConstraintViolation {
                    operation: "create_outcome",
                    ..
                })
            ));
            for invalid in [
                crate::domain::Outcome::new(
                    crate::domain::OutcomeId::new("outcome-missing-subject").unwrap(),
                    crate::domain::SelfSubjectId::new("missing-subject").unwrap(),
                    decision.id().clone(),
                    "Missing subject",
                )
                .unwrap(),
                crate::domain::Outcome::new(
                    crate::domain::OutcomeId::new("outcome-missing-decision").unwrap(),
                    subject_a.id().clone(),
                    crate::domain::DecisionId::new("missing-decision").unwrap(),
                    "Missing decision",
                )
                .unwrap(),
                crate::domain::Outcome::new(
                    crate::domain::OutcomeId::new("outcome-cross-decision").unwrap(),
                    subject_b.id().clone(),
                    decision.id().clone(),
                    "Cross-subject decision",
                )
                .unwrap(),
            ] {
                assert!(matches!(
                    repository.create_outcome(&invalid, 32).await,
                    Err(PersistenceError::ConstraintViolation {
                        operation: "create_outcome",
                        ..
                    })
                ));
            }

            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn outcome_collection_distinguishes_missing_empty_and_orders_storage_deterministically() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let repository = repository(&database);
            let subject = create_subject_fixture(&repository, "outcome-list-subject").await;
            let decision = crate::domain::Decision::new(
                crate::domain::DecisionId::new("outcome-list-decision").unwrap(),
                subject.id().clone(),
                None,
                "A decision that may have outcomes",
            )
            .unwrap();
            repository.create_decision(&decision, 5).await.unwrap();

            assert!(repository
                .load_outcomes_for_decision(decision.id())
                .await
                .unwrap()
                .is_empty());
            assert!(matches!(
                repository
                    .load_outcomes_for_decision(
                        &crate::domain::DecisionId::new("missing-decision").unwrap()
                    )
                    .await,
                Err(PersistenceError::NotFound {
                    entity: "Decision",
                    ..
                })
            ));

            let outcomes = [
                ("outcome-b", 10_i64),
                ("outcome-c", 20_i64),
                ("outcome-a", 10_i64),
            ]
            .map(|(id, created_at_ms)| {
                (
                    crate::domain::Outcome::new(
                        crate::domain::OutcomeId::new(id).unwrap(),
                        subject.id().clone(),
                        decision.id().clone(),
                        format!("Reported result {id}"),
                    )
                    .unwrap(),
                    created_at_ms,
                )
            });

            repository
                .create_outcome(&outcomes[0].0, outcomes[0].1)
                .await
                .unwrap();
            assert_eq!(
                repository
                    .load_outcomes_for_decision(decision.id())
                    .await
                    .unwrap(),
                vec![outcomes[0].0.clone()]
            );
            for (outcome, created_at_ms) in outcomes.iter().skip(1) {
                repository
                    .create_outcome(outcome, *created_at_ms)
                    .await
                    .unwrap();
            }

            let loaded = repository
                .load_outcomes_for_decision(decision.id())
                .await
                .unwrap();
            let loaded_ids = loaded
                .iter()
                .map(|outcome| outcome.id().as_str())
                .collect::<Vec<_>>();
            assert_eq!(loaded_ids, ["outcome-a", "outcome-b", "outcome-c"]);

            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn lived_experience_repository_loads_reject_corrupt_rows_without_repair() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let repository = repository(&database);
            let subject = create_subject_fixture(&repository, "corrupt-lived-subject").await;
            let decision = crate::domain::Decision::new(
                crate::domain::DecisionId::new("valid-parent-decision").unwrap(),
                subject.id().clone(),
                None,
                "A valid parent decision",
            )
            .unwrap();
            repository.create_decision(&decision, 1).await.unwrap();
            let valid_outcome = crate::domain::Outcome::new(
                crate::domain::OutcomeId::new("valid-outcome-before-corruption").unwrap(),
                subject.id().clone(),
                decision.id().clone(),
                "A valid row must not cause a later corrupt row to be skipped.",
            )
            .unwrap();
            repository.create_outcome(&valid_outcome, 2).await.unwrap();

            let mut connection = database.pool.acquire().await.unwrap();
            sqlx::query("PRAGMA ignore_check_constraints = ON")
                .execute(&mut *connection)
                .await
                .unwrap();
            sqlx::query(
                "INSERT INTO memories VALUES \
                 ('corrupt-memory', ?, NULL, 'Valid description', ' ', 3)",
            )
            .bind(subject.id().as_str())
            .execute(&mut *connection)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO decisions VALUES \
                 ('corrupt-decision', ?, NULL, ' ', 4)",
            )
            .bind(subject.id().as_str())
            .execute(&mut *connection)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO outcomes VALUES \
                 ('corrupt-outcome', ?, 'valid-parent-decision', ' ', 5)",
            )
            .bind(subject.id().as_str())
            .execute(&mut *connection)
            .await
            .unwrap();
            drop(connection);

            assert_domain_reconstruction(
                repository
                    .load_memory(&crate::domain::MemoryId::new("corrupt-memory").unwrap())
                    .await,
            );
            assert_domain_reconstruction(
                repository
                    .load_decision(&crate::domain::DecisionId::new("corrupt-decision").unwrap())
                    .await,
            );
            assert_domain_reconstruction(
                repository
                    .load_outcome(&crate::domain::OutcomeId::new("corrupt-outcome").unwrap())
                    .await,
            );
            assert_domain_reconstruction(
                repository.load_outcomes_for_decision(decision.id()).await,
            );

            drop(repository);
            database.close().await;
        });
    }

    #[test]
    fn lived_experience_rows_reject_every_invalid_domain_field() {
        let memory_rows = [
            MemoryRow {
                id: " ".into(),
                subject_id: "subject".into(),
                situation_id: None,
                description: "Description".into(),
                user_meaning: None,
                created_at_ms: 1,
            },
            MemoryRow {
                id: "memory".into(),
                subject_id: "\t".into(),
                situation_id: None,
                description: "Description".into(),
                user_meaning: None,
                created_at_ms: 1,
            },
            MemoryRow {
                id: "memory".into(),
                subject_id: "subject".into(),
                situation_id: Some(" ".into()),
                description: "Description".into(),
                user_meaning: None,
                created_at_ms: 1,
            },
            MemoryRow {
                id: "memory".into(),
                subject_id: "subject".into(),
                situation_id: None,
                description: " ".into(),
                user_meaning: None,
                created_at_ms: 1,
            },
            MemoryRow {
                id: "memory".into(),
                subject_id: "subject".into(),
                situation_id: None,
                description: "Description".into(),
                user_meaning: Some("\t".into()),
                created_at_ms: 1,
            },
        ];
        for row in memory_rows {
            assert_domain_reconstruction(crate::domain::Memory::try_from(row));
        }

        let decision_rows = [
            DecisionRow {
                id: " ".into(),
                subject_id: "subject".into(),
                situation_id: None,
                description: "Description".into(),
                created_at_ms: 1,
            },
            DecisionRow {
                id: "decision".into(),
                subject_id: "\n".into(),
                situation_id: None,
                description: "Description".into(),
                created_at_ms: 1,
            },
            DecisionRow {
                id: "decision".into(),
                subject_id: "subject".into(),
                situation_id: Some(" ".into()),
                description: "Description".into(),
                created_at_ms: 1,
            },
            DecisionRow {
                id: "decision".into(),
                subject_id: "subject".into(),
                situation_id: None,
                description: "\t".into(),
                created_at_ms: 1,
            },
        ];
        for row in decision_rows {
            assert_domain_reconstruction(crate::domain::Decision::try_from(row));
        }

        let outcome_rows = [
            OutcomeRow {
                id: " ".into(),
                subject_id: "subject".into(),
                decision_id: "decision".into(),
                description: "Description".into(),
                created_at_ms: 1,
            },
            OutcomeRow {
                id: "outcome".into(),
                subject_id: "\t".into(),
                decision_id: "decision".into(),
                description: "Description".into(),
                created_at_ms: 1,
            },
            OutcomeRow {
                id: "outcome".into(),
                subject_id: "subject".into(),
                decision_id: " ".into(),
                description: "Description".into(),
                created_at_ms: 1,
            },
            OutcomeRow {
                id: "outcome".into(),
                subject_id: "subject".into(),
                decision_id: "decision".into(),
                description: "\n".into(),
                created_at_ms: 1,
            },
        ];
        for row in outcome_rows {
            assert_domain_reconstruction(crate::domain::Outcome::try_from(row));
        }
    }

    #[test]
    fn lived_experience_records_and_storage_order_survive_close_and_reopen() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let repository = repository(&database);
            let subject = create_subject_fixture(&repository, "durable-lived-subject").await;
            let situation =
                create_situation_fixture(&repository, &subject, "durable-lived-situation").await;
            let memory_without_optional_fields = crate::domain::Memory::new(
                crate::domain::MemoryId::new("durable-memory-none").unwrap(),
                subject.id().clone(),
                None,
                "A remembered experience without linked context.",
                None,
            )
            .unwrap();
            let memory_with_optional_fields = crate::domain::Memory::new(
                crate::domain::MemoryId::new("durable-memory-context").unwrap(),
                subject.id().clone(),
                Some(situation.id().clone()),
                "这段记忆 remains important 🌱",
                Some("这是我自己写下的 meaning.".into()),
            )
            .unwrap();
            repository
                .create_memory(&memory_without_optional_fields, 10)
                .await
                .unwrap();
            repository
                .create_memory(&memory_with_optional_fields, 11)
                .await
                .unwrap();

            let decision = crate::domain::Decision::new(
                crate::domain::DecisionId::new("durable-lived-decision").unwrap(),
                subject.id().clone(),
                Some(situation.id().clone()),
                "我决定 continue with the plan.",
            )
            .unwrap();
            repository.create_decision(&decision, 12).await.unwrap();

            let outcomes = [
                ("durable-outcome-b", 20_i64),
                ("durable-outcome-c", 30_i64),
                ("durable-outcome-a", 20_i64),
            ]
            .map(|(id, created_at_ms)| {
                (
                    crate::domain::Outcome::new(
                        crate::domain::OutcomeId::new(id).unwrap(),
                        subject.id().clone(),
                        decision.id().clone(),
                        format!("结果 persisted for {id}"),
                    )
                    .unwrap(),
                    created_at_ms,
                )
            });
            for (outcome, created_at_ms) in &outcomes {
                repository
                    .create_outcome(outcome, *created_at_ms)
                    .await
                    .unwrap();
            }
            drop(repository);

            let database = database.reopen().await;
            let reopened_repository = SqliteSelfModelRepository::new(SharedSqlitePool {
                pool: database.pool.clone(),
            });
            assert_eq!(
                reopened_repository
                    .load_memory(memory_without_optional_fields.id())
                    .await
                    .unwrap(),
                memory_without_optional_fields
            );
            assert_eq!(
                reopened_repository
                    .load_memory(memory_with_optional_fields.id())
                    .await
                    .unwrap(),
                memory_with_optional_fields
            );
            assert_eq!(
                reopened_repository
                    .load_decision(decision.id())
                    .await
                    .unwrap(),
                decision
            );
            for (outcome, _) in &outcomes {
                assert_eq!(
                    reopened_repository
                        .load_outcome(outcome.id())
                        .await
                        .unwrap(),
                    *outcome
                );
            }
            let ordered_ids = reopened_repository
                .load_outcomes_for_decision(decision.id())
                .await
                .unwrap()
                .into_iter()
                .map(|outcome| outcome.id().as_str().to_owned())
                .collect::<Vec<_>>();
            assert_eq!(
                ordered_ids,
                [
                    "durable-outcome-a",
                    "durable-outcome-b",
                    "durable-outcome-c",
                ]
            );

            drop(reopened_repository);
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
            assert!(matches!(
                repository
                    .load_memory(&crate::domain::MemoryId::new("missing").unwrap())
                    .await,
                Err(PersistenceError::NotFound {
                    entity: "Memory",
                    ..
                })
            ));
            assert!(matches!(
                repository
                    .load_decision(&crate::domain::DecisionId::new("missing").unwrap())
                    .await,
                Err(PersistenceError::NotFound {
                    entity: "Decision",
                    ..
                })
            ));
            assert!(matches!(
                repository
                    .load_outcome(&crate::domain::OutcomeId::new("missing").unwrap())
                    .await,
                Err(PersistenceError::NotFound {
                    entity: "Outcome",
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

    #[test]
    fn structured_capture_rolls_back_bootstrap_and_earlier_rows_on_late_collision() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let repository = repository(&database);
            let subject = crate::domain::SelfSubject::new(
                crate::domain::SelfSubjectId::new("self").unwrap(),
                "Self",
            )
            .unwrap();
            let situation = crate::domain::Situation::new(
                crate::domain::SituationId::new("capture-situation").unwrap(),
                subject.id().clone(),
                "A confirmed context",
            )
            .unwrap();
            let observation = crate::domain::Observation::new(
                crate::domain::ObservationId::new("capture-observation").unwrap(),
                subject.id().clone(),
                Some(situation.id().clone()),
                "A confirmed observation",
            )
            .unwrap();
            let thoughts = [
                crate::domain::Thought::new(
                    crate::domain::ThoughtId::new("duplicate-thought").unwrap(),
                    subject.id().clone(),
                    Some(situation.id().clone()),
                    "First thought",
                    None,
                )
                .unwrap(),
                crate::domain::Thought::new(
                    crate::domain::ThoughtId::new("duplicate-thought").unwrap(),
                    subject.id().clone(),
                    Some(situation.id().clone()),
                    "Second thought",
                    None,
                )
                .unwrap(),
            ];

            assert!(matches!(
                repository
                    .create_structured_capture_atomic(
                        subject.id(),
                        Some(&subject),
                        Some(&situation),
                        &[observation],
                        &thoughts,
                        10,
                    )
                    .await,
                Err(PersistenceError::ConstraintViolation {
                    operation: "create_structured_capture_atomic",
                    ..
                })
            ));

            for table in ["self_subjects", "situations", "observations", "thoughts"] {
                let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
                    .fetch_one(&database.pool)
                    .await
                    .unwrap();
                assert_eq!(count, 0, "transaction left a row in {table}");
            }
            database.close().await;
        });
    }

    #[test]
    fn structured_capture_rejects_cross_subject_records_before_writing() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let repository = repository(&database);
            let subject_a = create_subject_fixture(&repository, "capture-a").await;
            let subject_b = create_subject_fixture(&repository, "capture-b").await;
            let thought = crate::domain::Thought::new(
                crate::domain::ThoughtId::new("cross-capture-thought").unwrap(),
                subject_a.id().clone(),
                None,
                "Must not cross ownership",
                None,
            )
            .unwrap();

            assert!(matches!(
                repository
                    .create_structured_capture_atomic(
                        subject_b.id(),
                        None,
                        None,
                        &[],
                        &[thought],
                        10,
                    )
                    .await,
                Err(PersistenceError::SubjectInvariant(_))
            ));
            let same_subject_thought = crate::domain::Thought::new(
                crate::domain::ThoughtId::new("multiple-subject-thought").unwrap(),
                subject_a.id().clone(),
                None,
                "Must not save while ownership is ambiguous",
                None,
            )
            .unwrap();
            assert!(matches!(
                repository
                    .create_structured_capture_atomic(
                        subject_a.id(),
                        None,
                        None,
                        &[],
                        &[same_subject_thought],
                        11,
                    )
                    .await,
                Err(PersistenceError::SubjectInvariant(_))
            ));
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM thoughts")
                .fetch_one(&database.pool)
                .await
                .unwrap();
            assert_eq!(count, 0);
            database.close().await;
        });
    }

    #[test]
    fn structured_capture_records_survive_close_and_reopen() {
        tauri::async_runtime::block_on(async {
            let database = migrated_database().await;
            let initial_repository = repository(&database);
            let subject = crate::domain::SelfSubject::new(
                crate::domain::SelfSubjectId::new("self").unwrap(),
                "Self",
            )
            .unwrap();
            let situation = crate::domain::Situation::new(
                crate::domain::SituationId::new("durable-capture-situation").unwrap(),
                subject.id().clone(),
                "Durable context",
            )
            .unwrap();
            let observation = crate::domain::Observation::new(
                crate::domain::ObservationId::new("durable-capture-observation").unwrap(),
                subject.id().clone(),
                Some(situation.id().clone()),
                "Durable observation",
            )
            .unwrap();
            let thought = crate::domain::Thought::new(
                crate::domain::ThoughtId::new("durable-capture-thought").unwrap(),
                subject.id().clone(),
                Some(situation.id().clone()),
                "Durable thought",
                None,
            )
            .unwrap();

            initial_repository
                .create_structured_capture_atomic(
                    subject.id(),
                    Some(&subject),
                    Some(&situation),
                    std::slice::from_ref(&observation),
                    std::slice::from_ref(&thought),
                    10,
                )
                .await
                .unwrap();
            drop(initial_repository);

            let database = database.reopen().await;
            let repository = repository(&database);
            assert_eq!(
                repository.load_situation(situation.id()).await.unwrap(),
                situation
            );
            assert_eq!(
                repository.load_observation(observation.id()).await.unwrap(),
                observation
            );
            let loaded_thought = repository.load_thought(thought.id()).await.unwrap();
            assert_eq!(loaded_thought, thought);
            assert_eq!(loaded_thought.confidence(), None);
            database.close().await;
        });
    }
}
