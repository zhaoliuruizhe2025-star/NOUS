use crate::persistence::{
    HistoryObservationRecord, HistorySituationRecord, HistoryThoughtRecord, PersistenceError,
    SqliteSelfModelRepository,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StructuredHistoryObservation {
    pub(crate) id: String,
    pub(crate) content: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StructuredHistoryThought {
    pub(crate) id: String,
    pub(crate) content: String,
    pub(crate) subjective_conviction: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StructuredHistoryContext {
    pub(crate) id: String,
    pub(crate) description: String,
    pub(crate) observations: Vec<StructuredHistoryObservation>,
    pub(crate) thoughts: Vec<StructuredHistoryThought>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StructuredHistory {
    pub(crate) contexts: Vec<StructuredHistoryContext>,
    pub(crate) standalone_observations: Vec<StructuredHistoryObservation>,
    pub(crate) standalone_thoughts: Vec<StructuredHistoryThought>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum StructuredHistoryError {
    DataInconsistent,
    Persistence(PersistenceError),
}

impl From<PersistenceError> for StructuredHistoryError {
    fn from(error: PersistenceError) -> Self {
        Self::Persistence(error)
    }
}

pub(crate) async fn load_structured_history(
    repository: &SqliteSelfModelRepository,
) -> Result<StructuredHistory, StructuredHistoryError> {
    let records = repository.load_structured_history().await?;

    let mut contexts = records
        .situations
        .into_iter()
        .map(
            |HistorySituationRecord {
                 value,
                 created_at_ms,
             }| {
                let _storage_order = created_at_ms;
                StructuredHistoryContext {
                    id: value.id().as_str().to_owned(),
                    description: value.description().to_owned(),
                    observations: Vec::new(),
                    thoughts: Vec::new(),
                }
            },
        )
        .collect::<Vec<_>>();
    let mut standalone_observations = Vec::new();
    let mut standalone_thoughts = Vec::new();

    for HistoryObservationRecord {
        value,
        created_at_ms,
    } in records.observations
    {
        let _storage_order = created_at_ms;
        let observation = StructuredHistoryObservation {
            id: value.id().as_str().to_owned(),
            content: value.content().to_owned(),
        };
        if let Some(situation_id) = value.situation_id() {
            let context = contexts
                .iter_mut()
                .find(|context| context.id == situation_id.as_str())
                .ok_or(StructuredHistoryError::DataInconsistent)?;
            context.observations.push(observation);
        } else {
            standalone_observations.push(observation);
        }
    }

    for HistoryThoughtRecord {
        value,
        created_at_ms,
    } in records.thoughts
    {
        let _storage_order = created_at_ms;
        let thought = StructuredHistoryThought {
            id: value.id().as_str().to_owned(),
            content: value.content().to_owned(),
            subjective_conviction: value.confidence().map(|confidence| confidence.value()),
        };
        if let Some(situation_id) = value.situation_id() {
            let context = contexts
                .iter_mut()
                .find(|context| context.id == situation_id.as_str())
                .ok_or(StructuredHistoryError::DataInconsistent)?;
            context.thoughts.push(thought);
        } else {
            standalone_thoughts.push(thought);
        }
    }

    Ok(StructuredHistory {
        contexts,
        standalone_observations,
        standalone_thoughts,
    })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        str::FromStr,
        sync::atomic::{AtomicU64, Ordering},
    };

    use sqlx::{
        sqlite::{SqliteConnectOptions, SqlitePoolOptions},
        SqlitePool,
    };

    use super::*;
    use crate::{
        domain::{
            Observation, ObservationId, SelfSubject, SelfSubjectId, Situation, SituationId,
            Thought, ThoughtConfidence, ThoughtId,
        },
        persistence::SharedSqlitePool,
    };

    static NEXT_DATABASE: AtomicU64 = AtomicU64::new(1);

    async fn test_repository() -> (SqlitePool, SqliteSelfModelRepository) {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        for migration in [
            include_str!("../../migrations/0001_initialize.sql"),
            include_str!("../../migrations/0002_create_self_model.sql"),
            include_str!("../../migrations/0003_create_lived_experience_records.sql"),
            include_str!("../../migrations/0004_create_evidence_links.sql"),
        ] {
            sqlx::raw_sql(migration).execute(&pool).await.unwrap();
        }
        let repository =
            SqliteSelfModelRepository::new(SharedSqlitePool::from_test_pool(pool.clone()));
        (pool, repository)
    }

    async fn count(pool: &SqlitePool, table: &str) -> i64 {
        sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(pool)
            .await
            .unwrap()
    }

    async fn subject(repository: &SqliteSelfModelRepository, id: &str) -> SelfSubject {
        let subject = SelfSubject::new(SelfSubjectId::new(id).unwrap(), "Self").unwrap();
        repository.create_self_subject(&subject, 1).await.unwrap();
        subject
    }

    #[test]
    fn empty_inspection_is_read_only_and_does_not_bootstrap() {
        tauri::async_runtime::block_on(async {
            let (pool, repository) = test_repository().await;

            let history = load_structured_history(&repository).await.unwrap();

            assert!(history.contexts.is_empty());
            assert!(history.standalone_observations.is_empty());
            assert!(history.standalone_thoughts.is_empty());
            for table in ["self_subjects", "situations", "observations", "thoughts"] {
                assert_eq!(count(&pool, table).await, 0);
            }
            pool.close().await;
        });
    }

    #[test]
    fn assembles_only_explicit_relationships_and_preserves_storage_order() {
        tauri::async_runtime::block_on(async {
            let (pool, repository) = test_repository().await;
            let owner = subject(&repository, "self").await;
            let context_a = Situation::new(
                SituationId::new("context-a").unwrap(),
                owner.id().clone(),
                "Context A",
            )
            .unwrap();
            let context_b = Situation::new(
                SituationId::new("context-b").unwrap(),
                owner.id().clone(),
                "Context B",
            )
            .unwrap();
            repository.create_situation(&context_a, 10).await.unwrap();
            repository.create_situation(&context_b, 10).await.unwrap();

            for (id, situation_id, content, created_at_ms) in [
                ("linked-a", Some(context_a.id().clone()), "Linked A", 20),
                ("linked-b", Some(context_a.id().clone()), "Linked B", 20),
                ("standalone", None, "Standalone report", 30),
            ] {
                repository
                    .create_observation(
                        &Observation::new(
                            ObservationId::new(id).unwrap(),
                            owner.id().clone(),
                            situation_id,
                            content,
                        )
                        .unwrap(),
                        created_at_ms,
                    )
                    .await
                    .unwrap();
            }
            repository
                .create_thought(
                    &Thought::new(
                        ThoughtId::new("linked-thought").unwrap(),
                        owner.id().clone(),
                        Some(context_b.id().clone()),
                        "Maybe they do not trust me.",
                        Some(ThoughtConfidence::new(80).unwrap()),
                    )
                    .unwrap(),
                    40,
                )
                .await
                .unwrap();
            repository
                .create_thought(
                    &Thought::new(
                        ThoughtId::new("standalone-thought").unwrap(),
                        owner.id().clone(),
                        None,
                        "I may be avoiding this problem.",
                        None,
                    )
                    .unwrap(),
                    50,
                )
                .await
                .unwrap();

            let history = load_structured_history(&repository).await.unwrap();

            assert_eq!(
                history
                    .contexts
                    .iter()
                    .map(|context| context.id.as_str())
                    .collect::<Vec<_>>(),
                ["context-b", "context-a"]
            );
            assert_eq!(
                history.contexts[0].thoughts[0].subjective_conviction,
                Some(80)
            );
            assert_eq!(history.contexts[1].observations[0].id, "linked-b");
            assert_eq!(history.contexts[1].observations[1].id, "linked-a");
            assert_eq!(history.standalone_observations[0].id, "standalone");
            assert_eq!(history.standalone_thoughts[0].subjective_conviction, None);
            assert_eq!(count(&pool, "self_subjects").await, 1);
            assert_eq!(count(&pool, "situations").await, 2);
            assert_eq!(count(&pool, "observations").await, 3);
            assert_eq!(count(&pool, "thoughts").await, 2);
            pool.close().await;
        });
    }

    #[test]
    fn multiple_subjects_fail_without_returning_mixed_history() {
        tauri::async_runtime::block_on(async {
            let (pool, repository) = test_repository().await;
            subject(&repository, "first").await;
            subject(&repository, "second").await;

            assert!(matches!(
                load_structured_history(&repository).await,
                Err(StructuredHistoryError::Persistence(
                    PersistenceError::SubjectInvariant(_)
                ))
            ));
            pool.close().await;
        });
    }

    #[test]
    fn orphan_ownership_and_invalid_relationships_are_not_silently_omitted() {
        tauri::async_runtime::block_on(async {
            for setup in ["no-subject", "other-subject", "missing-situation"] {
                let (pool, repository) = test_repository().await;
                if setup != "no-subject" {
                    subject(&repository, "self").await;
                }
                let mut connection = pool.acquire().await.unwrap();
                sqlx::query("PRAGMA foreign_keys = OFF")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                match setup {
                    "no-subject" => {
                        sqlx::query("INSERT INTO situations (id, subject_id, description, created_at_ms) VALUES ('orphan', 'missing', 'Orphan context', 1)")
                            .execute(&mut *connection)
                            .await
                            .unwrap();
                    }
                    "other-subject" => {
                        sqlx::query("INSERT INTO observations (id, subject_id, situation_id, content, created_at_ms) VALUES ('foreign', 'other', NULL, 'Foreign detail', 1)")
                            .execute(&mut *connection)
                            .await
                            .unwrap();
                    }
                    "missing-situation" => {
                        sqlx::query("INSERT INTO thoughts (id, subject_id, situation_id, content, confidence, created_at_ms) VALUES ('orphan-thought', 'self', 'missing', 'A thought', NULL, 1)")
                            .execute(&mut *connection)
                            .await
                            .unwrap();
                    }
                    _ => unreachable!(),
                }
                sqlx::query("PRAGMA foreign_keys = ON")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                drop(connection);

                assert!(matches!(
                    load_structured_history(&repository).await,
                    Err(StructuredHistoryError::Persistence(
                        PersistenceError::DomainReconstruction { .. }
                    ))
                ));
                pool.close().await;
            }
        });
    }

    #[test]
    fn corrupt_domain_rows_fail_the_entire_inspection() {
        tauri::async_runtime::block_on(async {
            let (pool, repository) = test_repository().await;
            subject(&repository, "self").await;
            let mut connection = pool.acquire().await.unwrap();
            sqlx::query("PRAGMA ignore_check_constraints = ON")
                .execute(&mut *connection)
                .await
                .unwrap();
            sqlx::query("INSERT INTO thoughts (id, subject_id, situation_id, content, confidence, created_at_ms) VALUES ('corrupt', 'self', NULL, ' ', NULL, 1)")
                .execute(&mut *connection)
                .await
                .unwrap();
            sqlx::query("PRAGMA ignore_check_constraints = OFF")
                .execute(&mut *connection)
                .await
                .unwrap();
            drop(connection);

            assert!(matches!(
                load_structured_history(&repository).await,
                Err(StructuredHistoryError::Persistence(
                    PersistenceError::DomainReconstruction { .. }
                ))
            ));
            pool.close().await;
        });
    }

    #[test]
    fn valid_history_survives_database_close_and_reopen() {
        tauri::async_runtime::block_on(async {
            let unique = NEXT_DATABASE.fetch_add(1, Ordering::Relaxed);
            let directory = std::env::temp_dir().join(format!(
                "nous-task008-history-{}-{unique}",
                std::process::id()
            ));
            fs::create_dir_all(&directory).unwrap();
            let database_path = directory.join("nous.db");
            let database_url = format!(
                "sqlite://{}",
                database_path.to_string_lossy().replace('\\', "/")
            );
            let options = SqliteConnectOptions::from_str(&database_url)
                .unwrap()
                .create_if_missing(true)
                .foreign_keys(true);
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options.clone())
                .await
                .unwrap();
            for migration in [
                include_str!("../../migrations/0001_initialize.sql"),
                include_str!("../../migrations/0002_create_self_model.sql"),
                include_str!("../../migrations/0003_create_lived_experience_records.sql"),
                include_str!("../../migrations/0004_create_evidence_links.sql"),
            ] {
                sqlx::raw_sql(migration).execute(&pool).await.unwrap();
            }
            let repository =
                SqliteSelfModelRepository::new(SharedSqlitePool::from_test_pool(pool.clone()));
            let owner = subject(&repository, "self").await;
            repository
                .create_thought(
                    &Thought::new(
                        ThoughtId::new("durable-thought").unwrap(),
                        owner.id().clone(),
                        None,
                        "This should still be inspectable.",
                        None,
                    )
                    .unwrap(),
                    2,
                )
                .await
                .unwrap();
            pool.close().await;

            let reopened = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .unwrap();
            let repository =
                SqliteSelfModelRepository::new(SharedSqlitePool::from_test_pool(reopened.clone()));
            let history = load_structured_history(&repository).await.unwrap();
            assert_eq!(history.standalone_thoughts[0].id, "durable-thought");

            reopened.close().await;
            fs::remove_dir_all(directory).unwrap();
        });
    }
}
