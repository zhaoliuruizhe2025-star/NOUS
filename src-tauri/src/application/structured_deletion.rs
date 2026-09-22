use crate::{
    domain::{ObservationId, SituationId, ThoughtId},
    persistence::{PersistenceError, SqliteSelfModelRepository, StructuredDeletionInput},
};

use super::structured_history::{
    assemble_structured_history, StructuredHistory, StructuredHistoryError,
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DeletionInput {
    Situation {
        target_id: String,
        expected_state_token: String,
    },
    Observation {
        target_id: String,
        expected_state_token: String,
    },
    Thought {
        target_id: String,
        expected_state_token: String,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum StructuredDeletionError {
    InvalidTarget,
    Persistence(PersistenceError),
    History(StructuredHistoryError),
}

pub(crate) async fn delete_structured_record(
    repository: &SqliteSelfModelRepository,
    input: DeletionInput,
) -> Result<StructuredHistory, StructuredDeletionError> {
    let deletion = match input {
        DeletionInput::Situation {
            target_id,
            expected_state_token,
        } => StructuredDeletionInput::Situation {
            target_id: SituationId::new(target_id)
                .map_err(|_| StructuredDeletionError::InvalidTarget)?,
            expected_state_token,
        },
        DeletionInput::Observation {
            target_id,
            expected_state_token,
        } => StructuredDeletionInput::Observation {
            target_id: ObservationId::new(target_id)
                .map_err(|_| StructuredDeletionError::InvalidTarget)?,
            expected_state_token,
        },
        DeletionInput::Thought {
            target_id,
            expected_state_token,
        } => StructuredDeletionInput::Thought {
            target_id: ThoughtId::new(target_id)
                .map_err(|_| StructuredDeletionError::InvalidTarget)?,
            expected_state_token,
        },
    };
    let records = repository
        .delete_structured_record_atomic(deletion)
        .await
        .map_err(StructuredDeletionError::Persistence)?;
    assemble_structured_history(records).map_err(StructuredDeletionError::History)
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
        application::{correct_structured_record, load_structured_history, CorrectionInput},
        persistence::SharedSqlitePool,
    };

    static NEXT_DATABASE: AtomicU64 = AtomicU64::new(1);

    async fn migrated_pool(
        options: SqliteConnectOptions,
    ) -> (SqlitePool, SqliteSelfModelRepository) {
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
            include_str!("../../migrations/0005_create_structured_corrections.sql"),
        ] {
            sqlx::raw_sql(migration).execute(&pool).await.unwrap();
        }
        let repository =
            SqliteSelfModelRepository::new(SharedSqlitePool::from_test_pool(pool.clone()));
        (pool, repository)
    }

    async fn database() -> (SqlitePool, SqliteSelfModelRepository) {
        migrated_pool(
            SqliteConnectOptions::from_str("sqlite::memory:")
                .unwrap()
                .foreign_keys(true),
        )
        .await
    }

    async fn subject(pool: &SqlitePool) {
        sqlx::query("INSERT INTO self_subjects (id, display_name, created_at_ms) VALUES ('self', 'Self', 1)")
            .execute(pool).await.unwrap();
    }

    async fn situation(pool: &SqlitePool) {
        sqlx::query("INSERT INTO situations (id, subject_id, description, created_at_ms) VALUES ('s', 'self', 'Tuesday meeting', 2)")
            .execute(pool).await.unwrap();
    }

    async fn observation(pool: &SqlitePool, context: Option<&str>) {
        sqlx::query("INSERT INTO observations (id, subject_id, situation_id, content, created_at_ms) VALUES ('o', 'self', ?, 'I heard an answer', 3)")
            .bind(context).execute(pool).await.unwrap();
    }

    async fn thought(pool: &SqlitePool, context: Option<&str>) {
        sqlx::query("INSERT INTO thoughts (id, subject_id, situation_id, content, confidence, created_at_ms) VALUES ('t', 'self', ?, 'I may be mistaken', 40, 4)")
            .bind(context).execute(pool).await.unwrap();
    }

    async fn count(pool: &SqlitePool, table: &str) -> i64 {
        sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(pool)
            .await
            .unwrap()
    }

    fn request(kind: &str, token: String) -> DeletionInput {
        match kind {
            "situation" => DeletionInput::Situation {
                target_id: "s".into(),
                expected_state_token: token,
            },
            "observation" => DeletionInput::Observation {
                target_id: "o".into(),
                expected_state_token: token,
            },
            _ => DeletionInput::Thought {
                target_id: "t".into(),
                expected_state_token: token,
            },
        }
    }

    async fn token(repository: &SqliteSelfModelRepository, kind: &str) -> String {
        let history = load_structured_history(repository).await.unwrap();
        match kind {
            "situation" => history.contexts[0].state_token.clone(),
            "observation" => history.standalone_observations[0].state_token.clone(),
            _ => history.standalone_thoughts[0].state_token.clone(),
        }
    }

    #[test]
    fn deletes_each_standalone_type_without_tombstone_or_other_writes() {
        tauri::async_runtime::block_on(async {
            for (kind, table) in [
                ("situation", "situations"),
                ("observation", "observations"),
                ("thought", "thoughts"),
            ] {
                let (pool, repository) = database().await;
                subject(&pool).await;
                match kind {
                    "situation" => situation(&pool).await,
                    "observation" => observation(&pool, None).await,
                    _ => thought(&pool, None).await,
                }
                let before = token(&repository, kind).await;
                let after = delete_structured_record(&repository, request(kind, before))
                    .await
                    .unwrap();
                assert!(
                    after.contexts.is_empty()
                        && after.standalone_observations.is_empty()
                        && after.standalone_thoughts.is_empty()
                );
                assert_eq!(count(&pool, table).await, 0);
                let no_longer_loads = match kind {
                    "situation" => repository
                        .load_situation(&SituationId::new("s").unwrap())
                        .await
                        .is_err(),
                    "observation" => repository
                        .load_observation(&ObservationId::new("o").unwrap())
                        .await
                        .is_err(),
                    _ => repository
                        .load_thought(&ThoughtId::new("t").unwrap())
                        .await
                        .is_err(),
                };
                assert!(no_longer_loads);
                assert_eq!(count(&pool, "self_subjects").await, 1);
                for audit in [
                    "situation_corrections",
                    "observation_corrections",
                    "thought_corrections",
                ] {
                    assert_eq!(count(&pool, audit).await, 0);
                }
                pool.close().await;
            }
        });
    }

    #[test]
    fn deletes_corrected_targets_and_all_own_provenance() {
        tauri::async_runtime::block_on(async {
            for (kind, table, audit) in [
                ("situation", "situations", "situation_corrections"),
                ("observation", "observations", "observation_corrections"),
                ("thought", "thoughts", "thought_corrections"),
            ] {
                let (pool, repository) = database().await;
                subject(&pool).await;
                match kind {
                    "situation" => situation(&pool).await,
                    "observation" => observation(&pool, None).await,
                    _ => thought(&pool, None).await,
                }
                let original = token(&repository, kind).await;
                let correction = match kind {
                    "situation" => CorrectionInput::Situation {
                        target_id: "s".into(),
                        expected_state_token: original.clone(),
                        description: "Thursday meeting".into(),
                        note: None,
                    },
                    "observation" => CorrectionInput::Observation {
                        target_id: "o".into(),
                        expected_state_token: original.clone(),
                        content: "I heard another answer".into(),
                        situation_id: None,
                        note: None,
                    },
                    _ => CorrectionInput::Thought {
                        target_id: "t".into(),
                        expected_state_token: original.clone(),
                        content: "I might be mistaken".into(),
                        situation_id: None,
                        subjective_conviction: Some(60),
                        note: None,
                    },
                };
                correct_structured_record(&repository, correction, 5)
                    .await
                    .unwrap();
                if kind == "situation" {
                    let first_token = token(&repository, kind).await;
                    correct_structured_record(
                        &repository,
                        CorrectionInput::Situation {
                            target_id: "s".into(),
                            expected_state_token: first_token,
                            description: "Friday meeting".into(),
                            note: None,
                        },
                        6,
                    )
                    .await
                    .unwrap();
                }
                assert_eq!(
                    count(&pool, audit).await,
                    if kind == "situation" { 2 } else { 1 }
                );
                assert!(matches!(
                    delete_structured_record(&repository, request(kind, original)).await,
                    Err(StructuredDeletionError::Persistence(
                        PersistenceError::StaleDelete
                    ))
                ));
                let current = token(&repository, kind).await;
                delete_structured_record(&repository, request(kind, current))
                    .await
                    .unwrap();
                assert_eq!(count(&pool, table).await, 0);
                assert_eq!(count(&pool, audit).await, 0);
                pool.close().await;
            }
        });
    }

    #[test]
    fn subject_and_missing_target_boundaries_are_safe() {
        tauri::async_runtime::block_on(async {
            let (pool, repository) = database().await;
            assert!(matches!(
                delete_structured_record(
                    &repository,
                    request("situation", "initial:situation:s".into())
                )
                .await,
                Err(StructuredDeletionError::Persistence(
                    PersistenceError::NotFound { .. }
                ))
            ));
            assert_eq!(count(&pool, "self_subjects").await, 0);
            subject(&pool).await;
            situation(&pool).await;
            assert!(matches!(
                delete_structured_record(
                    &repository,
                    request("observation", "initial:observation:o".into())
                )
                .await,
                Err(StructuredDeletionError::Persistence(
                    PersistenceError::NotFound { .. }
                ))
            ));
            sqlx::query("INSERT INTO self_subjects (id, display_name, created_at_ms) VALUES ('other', 'Other', 2)").execute(&pool).await.unwrap();
            assert!(matches!(
                delete_structured_record(
                    &repository,
                    request("situation", "initial:situation:s".into())
                )
                .await,
                Err(StructuredDeletionError::Persistence(
                    PersistenceError::SubjectInvariant(_)
                ))
            ));
            assert_eq!(count(&pool, "situations").await, 1);
            pool.close().await;

            let (pool, repository) = database().await;
            subject(&pool).await;
            let mut connection = pool.acquire().await.unwrap();
            sqlx::query("PRAGMA foreign_keys = OFF")
                .execute(&mut *connection)
                .await
                .unwrap();
            sqlx::query("INSERT INTO situations (id, subject_id, description, created_at_ms) VALUES ('s', 'other', 'A context', 1)")
                .execute(&mut *connection).await.unwrap();
            sqlx::query("PRAGMA foreign_keys = ON")
                .execute(&mut *connection)
                .await
                .unwrap();
            drop(connection);
            assert!(matches!(
                delete_structured_record(
                    &repository,
                    request("situation", "initial:situation:s".into())
                )
                .await,
                Err(StructuredDeletionError::Persistence(
                    PersistenceError::DomainReconstruction { .. }
                ))
            ));
            assert_eq!(count(&pool, "situations").await, 1);
            pool.close().await;

            let (pool, repository) = database().await;
            let mut connection = pool.acquire().await.unwrap();
            sqlx::query("PRAGMA foreign_keys = OFF")
                .execute(&mut *connection)
                .await
                .unwrap();
            sqlx::query("INSERT INTO situations (id, subject_id, description, created_at_ms) VALUES ('s', 'orphan', 'A context', 1)")
                .execute(&mut *connection).await.unwrap();
            sqlx::query("PRAGMA foreign_keys = ON")
                .execute(&mut *connection)
                .await
                .unwrap();
            drop(connection);
            let result = delete_structured_record(
                &repository,
                request("situation", "initial:situation:s".into()),
            )
            .await;
            assert!(
                matches!(
                    result,
                    Err(StructuredDeletionError::Persistence(
                        PersistenceError::DomainReconstruction { .. }
                    ))
                ),
                "{result:?}"
            );
            assert_eq!(count(&pool, "situations").await, 1);
            pool.close().await;
        });
    }

    #[test]
    fn current_structured_dependents_block_situation_deletion_without_detach() {
        tauri::async_runtime::block_on(async {
            for table in [
                "observations",
                "thoughts",
                "emotions",
                "memories",
                "decisions",
            ] {
                let (pool, repository) = database().await;
                subject(&pool).await;
                situation(&pool).await;
                match table {
                    "observations" => observation(&pool, Some("s")).await,
                    "thoughts" => thought(&pool, Some("s")).await,
                    "emotions" => {
                        sqlx::query("INSERT INTO emotions (id, subject_id, situation_id, label, intensity, created_at_ms) VALUES ('e','self','s','Uneasy',50,5)").execute(&pool).await.unwrap();
                    }
                    "memories" => {
                        sqlx::query("INSERT INTO memories (id, subject_id, situation_id, description, created_at_ms) VALUES ('m','self','s','A meeting',5)").execute(&pool).await.unwrap();
                    }
                    _ => {
                        sqlx::query("INSERT INTO decisions (id, subject_id, situation_id, description, created_at_ms) VALUES ('d','self','s','I waited',5)").execute(&pool).await.unwrap();
                    }
                }
                let result = delete_structured_record(
                    &repository,
                    request("situation", token(&repository, "situation").await),
                )
                .await;
                assert!(
                    matches!(
                        result,
                        Err(StructuredDeletionError::Persistence(
                            PersistenceError::DependencyBlocked
                        ))
                    ),
                    "{table}: {result:?}"
                );
                assert_eq!(count(&pool, "situations").await, 1);
                assert_eq!(count(&pool, table).await, 1);
                pool.close().await;
            }
        });
    }

    #[test]
    fn prior_correction_context_references_block_situation_deletion() {
        tauri::async_runtime::block_on(async {
            for kind in ["observation", "thought"] {
                let (pool, repository) = database().await;
                subject(&pool).await;
                situation(&pool).await;
                if kind == "observation" {
                    observation(&pool, Some("s")).await;
                } else {
                    thought(&pool, Some("s")).await;
                }
                let history = load_structured_history(&repository).await.unwrap();
                let correction = if kind == "observation" {
                    CorrectionInput::Observation {
                        target_id: "o".into(),
                        expected_state_token: history.contexts[0].observations[0]
                            .state_token
                            .clone(),
                        content: "I heard an answer".into(),
                        situation_id: None,
                        note: None,
                    }
                } else {
                    CorrectionInput::Thought {
                        target_id: "t".into(),
                        expected_state_token: history.contexts[0].thoughts[0].state_token.clone(),
                        content: "I may be mistaken".into(),
                        situation_id: None,
                        subjective_conviction: Some(40),
                        note: None,
                    }
                };
                correct_structured_record(&repository, correction, 5)
                    .await
                    .unwrap();
                assert!(matches!(
                    delete_structured_record(
                        &repository,
                        request("situation", token(&repository, "situation").await)
                    )
                    .await,
                    Err(StructuredDeletionError::Persistence(
                        PersistenceError::DependencyBlocked
                    ))
                ));
                assert_eq!(count(&pool, "situations").await, 1);
                pool.close().await;
            }
        });
    }

    async fn evidence(pool: &SqlitePool, kind: &str, source_column: &str, source_id: &str) {
        sqlx::raw_sql("INSERT INTO beliefs (id, subject_id, created_at_ms) VALUES ('b','self',6);
            INSERT INTO belief_revisions (id, belief_id, revision_number, proposition, origin, created_at_ms)
            VALUES ('br','b',1,'A belief','InitialUserEntry',7);")
            .execute(pool).await.unwrap();
        let statement = format!(
            "INSERT INTO evidence_links
            (id, subject_id, relationship_kind, provenance, source_kind, {source_column},
             target_kind, target_belief_id, target_belief_revision_id, created_at_ms)
            VALUES ('el','self','Contextualizes','UserAuthored',?,?,'BeliefRevision','b','br',8)"
        );
        sqlx::query(&statement)
            .bind(kind)
            .bind(source_id)
            .execute(pool)
            .await
            .unwrap();
    }

    #[test]
    fn exact_evidence_sources_block_all_types_and_corruption_is_not_hidden() {
        tauri::async_runtime::block_on(async {
            for (kind, evidence_kind, source_column, source_id) in [
                ("situation", "Situation", "source_situation_id", "s"),
                ("observation", "Observation", "source_observation_id", "o"),
                ("thought", "Thought", "source_thought_id", "t"),
            ] {
                let (pool, repository) = database().await;
                subject(&pool).await;
                match kind {
                    "situation" => situation(&pool).await,
                    "observation" => observation(&pool, None).await,
                    _ => thought(&pool, None).await,
                }
                evidence(&pool, evidence_kind, source_column, source_id).await;
                let current = token(&repository, kind).await;
                assert!(matches!(
                    delete_structured_record(&repository, request(kind, current.clone())).await,
                    Err(StructuredDeletionError::Persistence(
                        PersistenceError::DependencyBlocked
                    ))
                ));
                assert_eq!(count(&pool, "evidence_links").await, 1);

                let mut connection = pool.acquire().await.unwrap();
                sqlx::query("PRAGMA foreign_keys = OFF")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                sqlx::query("UPDATE evidence_links SET subject_id = 'other' WHERE id = 'el'")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                sqlx::query("PRAGMA foreign_keys = ON")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                drop(connection);
                assert!(matches!(
                    delete_structured_record(&repository, request(kind, current.clone())).await,
                    Err(StructuredDeletionError::Persistence(
                        PersistenceError::DataInconsistent(_)
                    ))
                ));
                let mut connection = pool.acquire().await.unwrap();
                sqlx::query("PRAGMA foreign_keys = OFF")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                sqlx::query("UPDATE evidence_links SET subject_id = 'self' WHERE id = 'el'")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                sqlx::query("PRAGMA foreign_keys = ON")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                drop(connection);
                let mut connection = pool.acquire().await.unwrap();
                sqlx::query("PRAGMA foreign_keys = OFF")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                sqlx::query("UPDATE evidence_links SET target_belief_revision_id = 'missing' WHERE id = 'el'")
                    .execute(&mut *connection).await.unwrap();
                sqlx::query("PRAGMA foreign_keys = ON")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                drop(connection);
                assert!(matches!(
                    delete_structured_record(&repository, request(kind, current.clone())).await,
                    Err(StructuredDeletionError::Persistence(
                        PersistenceError::DataInconsistent(_)
                    ))
                ));
                let mut connection = pool.acquire().await.unwrap();
                sqlx::query("PRAGMA foreign_keys = OFF")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                sqlx::query(
                    "UPDATE evidence_links SET target_belief_revision_id = 'br' WHERE id = 'el'",
                )
                .execute(&mut *connection)
                .await
                .unwrap();
                sqlx::query("PRAGMA foreign_keys = ON")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                sqlx::query("PRAGMA ignore_check_constraints = ON")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                sqlx::query("UPDATE evidence_links SET source_kind = 'Outcome' WHERE id = 'el'")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                sqlx::query("PRAGMA ignore_check_constraints = OFF")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                drop(connection);
                assert!(matches!(
                    delete_structured_record(&repository, request(kind, current)).await,
                    Err(StructuredDeletionError::Persistence(
                        PersistenceError::DataInconsistent(_)
                    ))
                ));
                assert_eq!(count(&pool, "evidence_links").await, 1);
                pool.close().await;
            }
        });
    }

    #[test]
    fn exact_external_subject_corruption_and_broken_own_chain_fail_before_writes() {
        tauri::async_runtime::block_on(async {
            let (pool, repository) = database().await;
            subject(&pool).await;
            situation(&pool).await;
            sqlx::query("INSERT INTO emotions (id, subject_id, situation_id, label, intensity, created_at_ms) VALUES ('e','self','s','Uneasy',50,5)")
                .execute(&pool).await.unwrap();
            let current = token(&repository, "situation").await;
            let mut connection = pool.acquire().await.unwrap();
            sqlx::query("PRAGMA foreign_keys = OFF")
                .execute(&mut *connection)
                .await
                .unwrap();
            sqlx::query("UPDATE emotions SET subject_id = 'other' WHERE id = 'e'")
                .execute(&mut *connection)
                .await
                .unwrap();
            sqlx::query("PRAGMA foreign_keys = ON")
                .execute(&mut *connection)
                .await
                .unwrap();
            drop(connection);
            assert!(matches!(
                delete_structured_record(&repository, request("situation", current)).await,
                Err(StructuredDeletionError::Persistence(
                    PersistenceError::DataInconsistent(_)
                ))
            ));
            assert_eq!(count(&pool, "situations").await, 1);
            pool.close().await;

            let (pool, repository) = database().await;
            subject(&pool).await;
            situation(&pool).await;
            let current = token(&repository, "situation").await;
            correct_structured_record(
                &repository,
                CorrectionInput::Situation {
                    target_id: "s".into(),
                    expected_state_token: current,
                    description: "Thursday meeting".into(),
                    note: None,
                },
                5,
            )
            .await
            .unwrap();
            sqlx::query("UPDATE situation_corrections SET after_state_token = 'broken' WHERE situation_id = 's'")
                .execute(&pool).await.unwrap();
            assert!(matches!(
                delete_structured_record(&repository, request("situation", "broken".into())).await,
                Err(StructuredDeletionError::Persistence(
                    PersistenceError::DataInconsistent(_)
                )) | Err(StructuredDeletionError::Persistence(
                    PersistenceError::DomainReconstruction { .. }
                ))
            ));
            assert_eq!(count(&pool, "situations").await, 1);
            assert_eq!(count(&pool, "situation_corrections").await, 1);
            pool.close().await;
        });
    }

    #[test]
    fn failure_after_audit_removal_and_audit_count_mismatch_roll_back() {
        tauri::async_runtime::block_on(async {
            for trigger in [
                "CREATE TRIGGER stop_target BEFORE DELETE ON situations BEGIN SELECT RAISE(FAIL, 'stop target'); END",
                "CREATE TRIGGER ignore_audit BEFORE DELETE ON situation_corrections BEGIN SELECT RAISE(IGNORE); END",
            ] {
                let (pool, repository) = database().await;
                subject(&pool).await;
                situation(&pool).await;
                let current = token(&repository, "situation").await;
                correct_structured_record(&repository, CorrectionInput::Situation {
                    target_id: "s".into(), expected_state_token: current,
                    description: "Thursday meeting".into(), note: None,
                }, 5).await.unwrap();
                let current = token(&repository, "situation").await;
                sqlx::query(trigger).execute(&pool).await.unwrap();
                let result = delete_structured_record(&repository, request("situation", current)).await;
                if trigger.contains("ignore_audit") {
                    assert!(matches!(result, Err(StructuredDeletionError::Persistence(PersistenceError::DeleteConflict(_)))));
                } else {
                    assert!(result.is_err());
                }
                assert_eq!(count(&pool, "situations").await, 1);
                assert_eq!(count(&pool, "situation_corrections").await, 1);
                pool.close().await;
            }
        });
    }

    #[test]
    fn standalone_and_corrected_deletes_survive_close_and_reopen() {
        tauri::async_runtime::block_on(async {
            for corrected in [false, true] {
                let unique = NEXT_DATABASE.fetch_add(1, Ordering::Relaxed);
                let directory = std::env::temp_dir().join(format!(
                    "nous-task010-delete-{}-{unique}",
                    std::process::id()
                ));
                fs::create_dir_all(&directory).unwrap();
                let path = directory.join("nous.db");
                let url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
                let options = SqliteConnectOptions::from_str(&url)
                    .unwrap()
                    .create_if_missing(true)
                    .foreign_keys(true);
                let (pool, repository) = migrated_pool(options.clone()).await;
                subject(&pool).await;
                thought(&pool, None).await;
                if corrected {
                    let current = token(&repository, "thought").await;
                    correct_structured_record(
                        &repository,
                        CorrectionInput::Thought {
                            target_id: "t".into(),
                            expected_state_token: current,
                            content: "I might be mistaken".into(),
                            situation_id: None,
                            subjective_conviction: Some(60),
                            note: None,
                        },
                        5,
                    )
                    .await
                    .unwrap();
                }
                let current = token(&repository, "thought").await;
                delete_structured_record(&repository, request("thought", current))
                    .await
                    .unwrap();
                drop(repository);
                pool.close().await;
                drop(pool);
                let reopened = SqlitePoolOptions::new()
                    .max_connections(1)
                    .connect_with(options)
                    .await
                    .unwrap();
                let repository = SqliteSelfModelRepository::new(SharedSqlitePool::from_test_pool(
                    reopened.clone(),
                ));
                assert!(load_structured_history(&repository)
                    .await
                    .unwrap()
                    .standalone_thoughts
                    .is_empty());
                assert_eq!(count(&reopened, "thoughts").await, 0);
                assert_eq!(count(&reopened, "thought_corrections").await, 0);
                let version: String = sqlx::query_scalar(
                    "SELECT value FROM app_metadata WHERE key = 'schema_version'",
                )
                .fetch_one(&reopened)
                .await
                .unwrap();
                assert_eq!(version, "5");
                drop(repository);
                reopened.close().await;
                drop(reopened);
                fs::remove_dir_all(directory).unwrap();
            }
        });
    }
}
