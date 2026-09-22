use crate::{
    domain::{ObservationId, SituationId, ThoughtConfidence, ThoughtId},
    persistence::{PersistenceError, SqliteSelfModelRepository, StructuredCorrectionInput},
};

use super::structured_history::{
    assemble_structured_history, StructuredHistory, StructuredHistoryError,
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CorrectionInput {
    Situation {
        target_id: String,
        expected_state_token: String,
        description: String,
        note: Option<String>,
    },
    Observation {
        target_id: String,
        expected_state_token: String,
        content: String,
        situation_id: Option<String>,
        note: Option<String>,
    },
    Thought {
        target_id: String,
        expected_state_token: String,
        content: String,
        situation_id: Option<String>,
        subjective_conviction: Option<u8>,
        note: Option<String>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum StructuredCorrectionError {
    InvalidCorrection,
    Persistence(PersistenceError),
    History(StructuredHistoryError),
}

fn normalize_note(value: Option<String>) -> Option<String> {
    value.filter(|text| !text.trim().is_empty())
}

pub(crate) async fn correct_structured_record(
    repository: &SqliteSelfModelRepository,
    input: CorrectionInput,
    recorded_at_ms: i64,
) -> Result<StructuredHistory, StructuredCorrectionError> {
    let correction = match input {
        CorrectionInput::Situation {
            target_id,
            expected_state_token,
            description,
            note,
        } => StructuredCorrectionInput::Situation {
            target_id: SituationId::new(target_id)
                .map_err(|_| StructuredCorrectionError::InvalidCorrection)?,
            expected_state_token,
            description,
            user_note: normalize_note(note),
        },
        CorrectionInput::Observation {
            target_id,
            expected_state_token,
            content,
            situation_id,
            note,
        } => StructuredCorrectionInput::Observation {
            target_id: ObservationId::new(target_id)
                .map_err(|_| StructuredCorrectionError::InvalidCorrection)?,
            expected_state_token,
            content,
            situation_id: situation_id
                .map(SituationId::new)
                .transpose()
                .map_err(|_| StructuredCorrectionError::InvalidCorrection)?,
            user_note: normalize_note(note),
        },
        CorrectionInput::Thought {
            target_id,
            expected_state_token,
            content,
            situation_id,
            subjective_conviction,
            note,
        } => StructuredCorrectionInput::Thought {
            target_id: ThoughtId::new(target_id)
                .map_err(|_| StructuredCorrectionError::InvalidCorrection)?,
            expected_state_token,
            content,
            situation_id: situation_id
                .map(SituationId::new)
                .transpose()
                .map_err(|_| StructuredCorrectionError::InvalidCorrection)?,
            confidence: subjective_conviction
                .map(ThoughtConfidence::new)
                .transpose()
                .map_err(|_| StructuredCorrectionError::InvalidCorrection)?,
            user_note: normalize_note(note),
        },
    };
    let records = repository
        .correct_structured_record_atomic(correction, recorded_at_ms)
        .await
        .map_err(StructuredCorrectionError::Persistence)?;
    assemble_structured_history(records).map_err(StructuredCorrectionError::History)
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
        application::load_structured_history,
        domain::{Observation, SelfSubject, SelfSubjectId, Situation, Thought},
        persistence::SharedSqlitePool,
    };

    static NEXT_DATABASE: AtomicU64 = AtomicU64::new(1);

    async fn database() -> (SqlitePool, SqliteSelfModelRepository) {
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
            include_str!("../../migrations/0005_create_structured_corrections.sql"),
        ] {
            sqlx::raw_sql(migration).execute(&pool).await.unwrap();
        }
        let repository =
            SqliteSelfModelRepository::new(SharedSqlitePool::from_test_pool(pool.clone()));
        (pool, repository)
    }

    async fn seed(repository: &SqliteSelfModelRepository) {
        let subject = SelfSubject::new(SelfSubjectId::new("self").unwrap(), "Self").unwrap();
        repository.create_self_subject(&subject, 1).await.unwrap();
        for (id, description) in [("s1", "Tuesday meeting"), ("s2", "Other meeting")] {
            repository
                .create_situation(
                    &Situation::new(
                        SituationId::new(id).unwrap(),
                        subject.id().clone(),
                        description,
                    )
                    .unwrap(),
                    2,
                )
                .await
                .unwrap();
        }
        repository
            .create_observation(
                &Observation::new(
                    ObservationId::new("o1").unwrap(),
                    subject.id().clone(),
                    Some(SituationId::new("s1").unwrap()),
                    "My teammate refused.",
                )
                .unwrap(),
                3,
            )
            .await
            .unwrap();
        repository
            .create_thought(
                &Thought::new(
                    ThoughtId::new("t1").unwrap(),
                    subject.id().clone(),
                    None,
                    "They definitely distrust me.",
                    Some(ThoughtConfidence::new(90).unwrap()),
                )
                .unwrap(),
                4,
            )
            .await
            .unwrap();
    }

    async fn count(pool: &SqlitePool, table: &str) -> i64 {
        sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(pool)
            .await
            .unwrap()
    }

    fn situation(token: String, description: &str) -> CorrectionInput {
        CorrectionInput::Situation {
            target_id: "s1".into(),
            expected_state_token: token,
            description: description.into(),
            note: None,
        }
    }

    #[test]
    fn corrects_all_three_types_with_stable_identity_relationship_and_conviction() {
        tauri::async_runtime::block_on(async {
            let (pool, repository) = database().await;
            seed(&repository).await;
            let initial = load_structured_history(&repository).await.unwrap();
            let first = correct_structured_record(
                &repository,
                CorrectionInput::Situation {
                    target_id: "s1".into(),
                    expected_state_token: initial.contexts[1].state_token.clone(),
                    description: "Thursday meeting".into(),
                    note: Some("  ".into()),
                },
                10,
            )
            .await
            .unwrap();
            assert_eq!(first.contexts[1].id, "s1");
            assert_eq!(first.contexts[1].description, "Thursday meeting");
            assert_eq!(first.contexts[1].corrections[0].note, None);
            assert_eq!(
                first.contexts[1].corrections[0].before_description,
                "Tuesday meeting"
            );
            let second = correct_structured_record(
                &repository,
                CorrectionInput::Observation {
                    target_id: "o1".into(),
                    expected_state_token: first.contexts[1].observations[0].state_token.clone(),
                    content: "They said they were too busy.".into(),
                    situation_id: Some("s2".into()),
                    note: Some("My wording was too strong.".into()),
                },
                11,
            )
            .await
            .unwrap();
            assert_eq!(second.contexts[0].observations[0].id, "o1");
            assert_eq!(
                second.contexts[0].observations[0].content,
                "They said they were too busy."
            );
            assert_eq!(
                second.contexts[0].observations[0].corrections[0]
                    .before_context
                    .as_ref()
                    .unwrap()
                    .id,
                "s1"
            );
            let third = correct_structured_record(
                &repository,
                CorrectionInput::Thought {
                    target_id: "t1".into(),
                    expected_state_token: second.standalone_thoughts[0].state_token.clone(),
                    content: "I worried they might distrust me.".into(),
                    situation_id: Some("s2".into()),
                    subjective_conviction: Some(60),
                    note: None,
                },
                12,
            )
            .await
            .unwrap();
            assert_eq!(third.contexts[0].thoughts[0].id, "t1");
            assert_eq!(
                third.contexts[0].thoughts[0].subjective_conviction,
                Some(60)
            );
            assert_eq!(
                third.contexts[0].thoughts[0].corrections[0].before_subjective_conviction,
                Some(90)
            );
            assert_eq!(count(&pool, "situation_corrections").await, 1);
            assert_eq!(count(&pool, "observation_corrections").await, 1);
            assert_eq!(count(&pool, "thought_corrections").await, 1);
            assert_eq!(count(&pool, "beliefs").await, 0);
            assert_eq!(count(&pool, "evidence_links").await, 0);
            pool.close().await;
        });
    }

    #[test]
    fn repeated_corrections_advance_tokens_and_reject_stale_and_noop_requests() {
        tauri::async_runtime::block_on(async {
            let (pool, repository) = database().await;
            seed(&repository).await;
            let original = load_structured_history(&repository).await.unwrap().contexts[1]
                .state_token
                .clone();
            let first = correct_structured_record(
                &repository,
                situation(original.clone(), "Thursday meeting"),
                10,
            )
            .await
            .unwrap();
            let first_token = first.contexts[1].state_token.clone();
            assert!(matches!(
                correct_structured_record(
                    &repository,
                    situation(original.clone(), "Friday meeting"),
                    11
                )
                .await,
                Err(StructuredCorrectionError::Persistence(
                    PersistenceError::StaleCorrection
                ))
            ));
            assert!(matches!(
                correct_structured_record(
                    &repository,
                    situation(first_token.clone(), "Thursday meeting"),
                    11
                )
                .await,
                Err(StructuredCorrectionError::Persistence(
                    PersistenceError::NoChanges
                ))
            ));
            let second = correct_structured_record(
                &repository,
                situation(first_token, "Tuesday meeting"),
                12,
            )
            .await
            .unwrap();
            assert_eq!(
                second.contexts[1]
                    .corrections
                    .iter()
                    .map(|item| item.sequence)
                    .collect::<Vec<_>>(),
                [1, 2]
            );
            assert_ne!(second.contexts[1].state_token, original);
            assert!(matches!(
                correct_structured_record(&repository, situation(original, "Monday meeting"), 13)
                    .await,
                Err(StructuredCorrectionError::Persistence(
                    PersistenceError::StaleCorrection
                ))
            ));
            assert_eq!(count(&pool, "situation_corrections").await, 2);
            pool.close().await;
        });
    }

    #[test]
    fn evidence_reference_blocks_real_change_but_not_noop_and_detects_bad_ownership() {
        tauri::async_runtime::block_on(async {
            for (kind, column, target) in [
                ("Situation", "source_situation_id", "s1"),
                ("Observation", "source_observation_id", "o1"),
                ("Thought", "source_thought_id", "t1"),
            ] {
                let (pool, repository) = database().await;
                seed(&repository).await;
                sqlx::raw_sql("INSERT INTO beliefs (id, subject_id, created_at_ms) VALUES ('b', 'self', 5);
                    INSERT INTO belief_revisions (id, belief_id, revision_number, proposition, origin, created_at_ms)
                    VALUES ('br', 'b', 1, 'A belief', 'InitialUserEntry', 6);").execute(&pool).await.unwrap();
                let statement = format!("INSERT INTO evidence_links (id, subject_id, relationship_kind, provenance, source_kind, {column}, target_kind, target_belief_id, target_belief_revision_id, created_at_ms) VALUES ('e', 'self', 'Contextualizes', 'UserAuthored', ?, ?, 'BeliefRevision', 'b', 'br', 7)");
                sqlx::query(&statement)
                    .bind(kind)
                    .bind(target)
                    .execute(&pool)
                    .await
                    .unwrap();
                let history = load_structured_history(&repository).await.unwrap();
                let (token, noop, changed) = match kind {
                    "Situation" => {
                        let token = history.contexts[1].state_token.clone();
                        (
                            token.clone(),
                            situation(token.clone(), "Tuesday meeting"),
                            situation(token, "Thursday meeting"),
                        )
                    }
                    "Observation" => {
                        let token = history.contexts[1].observations[0].state_token.clone();
                        (
                            token.clone(),
                            CorrectionInput::Observation {
                                target_id: target.into(),
                                expected_state_token: token.clone(),
                                content: "My teammate refused.".into(),
                                situation_id: Some("s1".into()),
                                note: None,
                            },
                            CorrectionInput::Observation {
                                target_id: target.into(),
                                expected_state_token: token,
                                content: "They were busy.".into(),
                                situation_id: Some("s1".into()),
                                note: None,
                            },
                        )
                    }
                    _ => {
                        let token = history.standalone_thoughts[0].state_token.clone();
                        (
                            token.clone(),
                            CorrectionInput::Thought {
                                target_id: target.into(),
                                expected_state_token: token.clone(),
                                content: "They definitely distrust me.".into(),
                                situation_id: None,
                                subjective_conviction: Some(90),
                                note: None,
                            },
                            CorrectionInput::Thought {
                                target_id: target.into(),
                                expected_state_token: token,
                                content: "I worried.".into(),
                                situation_id: None,
                                subjective_conviction: Some(60),
                                note: None,
                            },
                        )
                    }
                };
                assert!(!token.is_empty());
                assert!(matches!(
                    correct_structured_record(&repository, noop, 8).await,
                    Err(StructuredCorrectionError::Persistence(
                        PersistenceError::NoChanges
                    ))
                ));
                assert!(matches!(
                    correct_structured_record(&repository, changed, 8).await,
                    Err(StructuredCorrectionError::Persistence(
                        PersistenceError::EvidenceReferenceBlocked
                    ))
                ));
                assert_eq!(count(&pool, "evidence_links").await, 1);
                let mut connection = pool.acquire().await.unwrap();
                sqlx::query("PRAGMA foreign_keys = OFF")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                sqlx::query("UPDATE evidence_links SET subject_id = 'other' WHERE id = 'e'")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                sqlx::query("PRAGMA foreign_keys = ON")
                    .execute(&mut *connection)
                    .await
                    .unwrap();
                drop(connection);
                let changed = match kind {
                    "Situation" => situation(token, "Thursday meeting"),
                    "Observation" => CorrectionInput::Observation {
                        target_id: target.into(),
                        expected_state_token: token,
                        content: "They were busy.".into(),
                        situation_id: Some("s1".into()),
                        note: None,
                    },
                    _ => CorrectionInput::Thought {
                        target_id: target.into(),
                        expected_state_token: token,
                        content: "I worried.".into(),
                        situation_id: None,
                        subjective_conviction: Some(60),
                        note: None,
                    },
                };
                assert!(matches!(
                    correct_structured_record(&repository, changed, 9).await,
                    Err(StructuredCorrectionError::Persistence(
                        PersistenceError::DataInconsistent(_)
                    ))
                ));
                assert_eq!(
                    count(&pool, "situation_corrections").await
                        + count(&pool, "observation_corrections").await
                        + count(&pool, "thought_corrections").await,
                    0
                );
                pool.close().await;
            }
        });
    }

    #[test]
    fn migration_from_v4_and_close_reopen_preserve_correction() {
        tauri::async_runtime::block_on(async {
            let unique = NEXT_DATABASE.fetch_add(1, Ordering::Relaxed);
            let directory = std::env::temp_dir().join(format!(
                "nous-task009-correction-{}-{unique}",
                std::process::id()
            ));
            fs::create_dir_all(&directory).unwrap();
            let path = directory.join("nous.db");
            let url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));
            let options = SqliteConnectOptions::from_str(&url)
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
            seed(&repository).await;
            let version: String =
                sqlx::query_scalar("SELECT value FROM app_metadata WHERE key = 'schema_version'")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(version, "4");
            sqlx::raw_sql(include_str!(
                "../../migrations/0005_create_structured_corrections.sql"
            ))
            .execute(&pool)
            .await
            .unwrap();
            let version: String =
                sqlx::query_scalar("SELECT value FROM app_metadata WHERE key = 'schema_version'")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(version, "5");
            let token = load_structured_history(&repository).await.unwrap().contexts[1]
                .state_token
                .clone();
            correct_structured_record(&repository, situation(token, "Thursday meeting"), 10)
                .await
                .unwrap();
            drop(repository);
            pool.close().await;
            let reopened = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await
                .unwrap();
            let repository =
                SqliteSelfModelRepository::new(SharedSqlitePool::from_test_pool(reopened.clone()));
            let history = load_structured_history(&repository).await.unwrap();
            assert_eq!(history.contexts[1].description, "Thursday meeting");
            assert_eq!(history.contexts[1].corrections.len(), 1);
            drop(repository);
            reopened.close().await;
            drop(reopened);
            let _ = fs::remove_dir_all(directory);
        });
    }

    #[test]
    fn broken_audit_chains_and_orphans_make_history_inconsistent() {
        tauri::async_runtime::block_on(async {
            for defect in [
                "orphan",
                "foreign",
                "sequence",
                "before",
                "latest",
                "malformed",
            ] {
                let (pool, repository) = database().await;
                seed(&repository).await;
                if defect == "orphan" || defect == "foreign" {
                    let mut connection = pool.acquire().await.unwrap();
                    sqlx::query("PRAGMA foreign_keys = OFF")
                        .execute(&mut *connection)
                        .await
                        .unwrap();
                    let target = if defect == "orphan" { "missing" } else { "s1" };
                    let owner = if defect == "orphan" { "self" } else { "other" };
                    sqlx::query("INSERT INTO situation_corrections (situation_id, subject_id, correction_sequence, before_description, after_description, before_state_token, after_state_token, recorded_at_ms) VALUES (?, ?, 1, 'Old', 'New', 'initial:situation:s1', 'corrected:situation:0123456789abcdef0123456789abcdef', 10)")
                        .bind(target).bind(owner).execute(&mut *connection).await.unwrap();
                    sqlx::query("PRAGMA foreign_keys = ON")
                        .execute(&mut *connection)
                        .await
                        .unwrap();
                } else {
                    let token = load_structured_history(&repository).await.unwrap().contexts[1]
                        .state_token
                        .clone();
                    correct_structured_record(
                        &repository,
                        situation(token, "Thursday meeting"),
                        10,
                    )
                    .await
                    .unwrap();
                    let mut connection = pool.acquire().await.unwrap();
                    match defect {
                        "sequence" => {
                            sqlx::query("UPDATE situation_corrections SET correction_sequence = 2 WHERE situation_id = 's1'").execute(&mut *connection).await.unwrap();
                        }
                        "before" => {
                            sqlx::query("UPDATE situation_corrections SET before_description = 'Wrong earlier content' WHERE situation_id = 's1'").execute(&mut *connection).await.unwrap();
                            sqlx::query("INSERT INTO situation_corrections (situation_id, subject_id, correction_sequence, before_description, after_description, before_state_token, after_state_token, recorded_at_ms) VALUES ('s1', 'self', 2, 'Not the prior after', 'Another value', 'corrected:situation:0123456789abcdef0123456789abcdef', 'corrected:situation:fedcba9876543210fedcba9876543210', 11)").execute(&mut *connection).await.unwrap();
                        }
                        "latest" => {
                            sqlx::query("UPDATE situations SET description = 'Untracked change' WHERE id = 's1'").execute(&mut *connection).await.unwrap();
                        }
                        "malformed" => {
                            sqlx::query("PRAGMA ignore_check_constraints = ON")
                                .execute(&mut *connection)
                                .await
                                .unwrap();
                            sqlx::query("UPDATE situation_corrections SET after_state_token = 'bad' WHERE situation_id = 's1'").execute(&mut *connection).await.unwrap();
                            sqlx::query("PRAGMA ignore_check_constraints = OFF")
                                .execute(&mut *connection)
                                .await
                                .unwrap();
                        }
                        _ => unreachable!(),
                    }
                }
                assert!(
                    load_structured_history(&repository).await.is_err(),
                    "{defect} was silently accepted"
                );
                pool.close().await;
            }
        });
    }

    #[test]
    fn late_update_failure_rolls_back_audit_and_current_value() {
        tauri::async_runtime::block_on(async {
            let (pool, repository) = database().await;
            seed(&repository).await;
            let token = load_structured_history(&repository).await.unwrap().contexts[1]
                .state_token
                .clone();
            sqlx::query("CREATE TRIGGER reject_situation_correction BEFORE UPDATE ON situations BEGIN SELECT RAISE(ABORT, 'injected failure'); END")
                .execute(&pool).await.unwrap();
            assert!(correct_structured_record(
                &repository,
                situation(token, "Thursday meeting"),
                10
            )
            .await
            .is_err());
            assert_eq!(count(&pool, "situation_corrections").await, 0);
            let description: String =
                sqlx::query_scalar("SELECT description FROM situations WHERE id = 's1'")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(description, "Tuesday meeting");
            pool.close().await;
        });
    }

    #[test]
    fn subject_and_target_boundaries_fail_without_writes() {
        tauri::async_runtime::block_on(async {
            let (pool, repository) = database().await;
            assert!(matches!(
                correct_structured_record(
                    &repository,
                    situation("initial:situation:s1".into(), "Thursday meeting"),
                    10
                )
                .await,
                Err(StructuredCorrectionError::Persistence(
                    PersistenceError::NotFound { .. }
                ))
            ));
            seed(&repository).await;
            let missing = CorrectionInput::Situation {
                target_id: "missing".into(),
                expected_state_token: "initial:situation:missing".into(),
                description: "A context".into(),
                note: None,
            };
            assert!(matches!(
                correct_structured_record(&repository, missing, 10).await,
                Err(StructuredCorrectionError::Persistence(
                    PersistenceError::NotFound { .. }
                ))
            ));
            let token = load_structured_history(&repository).await.unwrap().contexts[1]
                .observations[0]
                .state_token
                .clone();
            let invalid_relation = CorrectionInput::Observation {
                target_id: "o1".into(),
                expected_state_token: token,
                content: "Corrected".into(),
                situation_id: Some("missing".into()),
                note: None,
            };
            assert!(matches!(
                correct_structured_record(&repository, invalid_relation, 10).await,
                Err(StructuredCorrectionError::Persistence(
                    PersistenceError::InvalidCorrection(_)
                ))
            ));
            repository
                .create_self_subject(
                    &SelfSubject::new(SelfSubjectId::new("other").unwrap(), "Other").unwrap(),
                    10,
                )
                .await
                .unwrap();
            assert!(matches!(
                correct_structured_record(
                    &repository,
                    situation("initial:situation:s1".into(), "Thursday meeting"),
                    11
                )
                .await,
                Err(StructuredCorrectionError::Persistence(
                    PersistenceError::SubjectInvariant(_)
                ))
            ));
            assert_eq!(
                count(&pool, "situation_corrections").await
                    + count(&pool, "observation_corrections").await,
                0
            );
            pool.close().await;
        });
    }
}
