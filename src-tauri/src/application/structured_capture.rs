use crate::{
    domain::{
        Observation, ObservationId, SelfSubject, SelfSubjectId, Situation, SituationId, Thought,
        ThoughtId, ValidationError,
    },
    persistence::{PersistenceError, SqliteSelfModelRepository},
};

const BOOTSTRAP_SUBJECT_ID: &str = "self";
const BOOTSTRAP_SUBJECT_DISPLAY_NAME: &str = "Self";

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SituationCaptureInput {
    pub(crate) id: String,
    pub(crate) description: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ObservationCaptureInput {
    pub(crate) id: String,
    pub(crate) content: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ThoughtCaptureInput {
    pub(crate) id: String,
    pub(crate) content: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SaveStructuredCaptureInput {
    pub(crate) situation: Option<SituationCaptureInput>,
    pub(crate) observations: Vec<ObservationCaptureInput>,
    pub(crate) thoughts: Vec<ThoughtCaptureInput>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SavedStructuredCapture {
    pub(crate) situation: Option<Situation>,
    pub(crate) observations: Vec<Observation>,
    pub(crate) thoughts: Vec<Thought>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum StructuredCaptureError {
    EmptyCapture,
    InvalidSituation(ValidationError),
    InvalidObservation {
        index: usize,
        source: ValidationError,
    },
    InvalidThought {
        index: usize,
        source: ValidationError,
    },
    Persistence(PersistenceError),
}

impl From<PersistenceError> for StructuredCaptureError {
    fn from(error: PersistenceError) -> Self {
        Self::Persistence(error)
    }
}

pub(crate) async fn save_structured_capture(
    repository: &SqliteSelfModelRepository,
    input: SaveStructuredCaptureInput,
    created_at_ms: i64,
) -> Result<SavedStructuredCapture, StructuredCaptureError> {
    if input.situation.is_none() && input.observations.is_empty() && input.thoughts.is_empty() {
        return Err(StructuredCaptureError::EmptyCapture);
    }

    let current_subject = repository.load_single_self_subject_for_capture().await?;
    let bootstrap_subject = if current_subject.is_none() {
        Some(
            SelfSubject::new(
                SelfSubjectId::new(BOOTSTRAP_SUBJECT_ID)
                    .expect("the fixed bootstrap subject ID is valid"),
                BOOTSTRAP_SUBJECT_DISPLAY_NAME,
            )
            .expect("the fixed bootstrap subject is valid"),
        )
    } else {
        None
    };
    let subject = current_subject
        .as_ref()
        .or(bootstrap_subject.as_ref())
        .expect("a current or bootstrap subject is always available");

    let situation = input
        .situation
        .map(|candidate| {
            Situation::new(
                SituationId::new(candidate.id).map_err(StructuredCaptureError::InvalidSituation)?,
                subject.id().clone(),
                candidate.description,
            )
            .map_err(StructuredCaptureError::InvalidSituation)
        })
        .transpose()?;
    let situation_id = situation.as_ref().map(|value| value.id().clone());

    let observations = input
        .observations
        .into_iter()
        .enumerate()
        .map(|(index, candidate)| {
            let id = ObservationId::new(candidate.id)
                .map_err(|source| StructuredCaptureError::InvalidObservation { index, source })?;
            Observation::new(
                id,
                subject.id().clone(),
                situation_id.clone(),
                candidate.content,
            )
            .map_err(|source| StructuredCaptureError::InvalidObservation { index, source })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let thoughts = input
        .thoughts
        .into_iter()
        .enumerate()
        .map(|(index, candidate)| {
            let id = ThoughtId::new(candidate.id)
                .map_err(|source| StructuredCaptureError::InvalidThought { index, source })?;
            Thought::new(
                id,
                subject.id().clone(),
                situation_id.clone(),
                candidate.content,
                None,
            )
            .map_err(|source| StructuredCaptureError::InvalidThought { index, source })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let saved = SavedStructuredCapture {
        situation,
        observations,
        thoughts,
    };

    repository
        .create_structured_capture_atomic(
            subject.id(),
            bootstrap_subject.as_ref(),
            saved.situation.as_ref(),
            &saved.observations,
            &saved.thoughts,
            created_at_ms,
        )
        .await?;

    Ok(saved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    use sqlx::{
        sqlite::{SqliteConnectOptions, SqlitePoolOptions},
        SqlitePool,
    };

    use crate::persistence::SharedSqlitePool;

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
            include_str!("../../migrations/0005_create_structured_corrections.sql"),
        ] {
            sqlx::raw_sql(migration).execute(&pool).await.unwrap();
        }
        let repository =
            SqliteSelfModelRepository::new(SharedSqlitePool::from_test_pool(pool.clone()));
        (pool, repository)
    }

    async fn count(pool: &SqlitePool, table: &str) -> i64 {
        let table = if table == "values" {
            "\"values\""
        } else {
            table
        };
        let query = format!("SELECT COUNT(*) FROM {table}");
        sqlx::query_scalar(&query).fetch_one(pool).await.unwrap()
    }

    fn input(
        situation: Option<(&str, &str)>,
        observations: &[(&str, &str)],
        thoughts: &[(&str, &str)],
    ) -> SaveStructuredCaptureInput {
        SaveStructuredCaptureInput {
            situation: situation.map(|(id, description)| SituationCaptureInput {
                id: id.into(),
                description: description.into(),
            }),
            observations: observations
                .iter()
                .map(|(id, content)| ObservationCaptureInput {
                    id: (*id).into(),
                    content: (*content).into(),
                })
                .collect(),
            thoughts: thoughts
                .iter()
                .map(|(id, content)| ThoughtCaptureInput {
                    id: (*id).into(),
                    content: (*content).into(),
                })
                .collect(),
        }
    }

    #[test]
    fn fixed_bootstrap_identity_is_valid_and_neutral() {
        let subject = SelfSubject::new(
            SelfSubjectId::new(BOOTSTRAP_SUBJECT_ID).unwrap(),
            BOOTSTRAP_SUBJECT_DISPLAY_NAME,
        )
        .unwrap();

        assert_eq!(subject.id().as_str(), "self");
        assert_eq!(subject.display_name(), "Self");
    }

    #[test]
    fn valid_variable_capture_shapes_persist_and_reuse_one_subject() {
        tauri::async_runtime::block_on(async {
            let (pool, repository) = test_repository().await;

            let thought_only = save_structured_capture(
                &repository,
                input(None, &[], &[("thought-only", "Maybe I should wait.")]),
                1,
            )
            .await
            .unwrap();
            assert_eq!(thought_only.thoughts.len(), 1);
            assert_eq!(thought_only.thoughts[0].confidence(), None);

            save_structured_capture(
                &repository,
                input(None, &[("observation-only", "The meeting ended.")], &[]),
                2,
            )
            .await
            .unwrap();
            save_structured_capture(
                &repository,
                input(Some(("situation-only", "A meeting today")), &[], &[]),
                3,
            )
            .await
            .unwrap();
            let mixed = save_structured_capture(
                &repository,
                input(
                    Some(("mixed-situation", "Working with my team")),
                    &[
                        ("mixed-observation-1", "They had completed the draft."),
                        ("mixed-observation-2", "I took over one section."),
                    ],
                    &[
                        ("mixed-thought-1", "They may not trust me."),
                        ("mixed-thought-2", "I should handle it myself."),
                    ],
                ),
                4,
            )
            .await
            .unwrap();

            assert_eq!(mixed.observations.len(), 2);
            assert_eq!(mixed.thoughts.len(), 2);
            assert!(mixed
                .observations
                .iter()
                .all(|value| value.situation_id() == mixed.situation.as_ref().map(Situation::id)));
            assert_eq!(count(&pool, "self_subjects").await, 1);
            assert_eq!(count(&pool, "situations").await, 2);
            assert_eq!(count(&pool, "observations").await, 3);
            assert_eq!(count(&pool, "thoughts").await, 3);
            pool.close().await;
        });
    }

    #[test]
    fn empty_and_invalid_capture_inputs_cause_zero_writes() {
        tauri::async_runtime::block_on(async {
            let (pool, repository) = test_repository().await;

            assert!(matches!(
                save_structured_capture(&repository, input(None, &[], &[]), 1).await,
                Err(StructuredCaptureError::EmptyCapture)
            ));
            assert!(matches!(
                save_structured_capture(
                    &repository,
                    input(Some(("invalid-situation", "  ")), &[], &[]),
                    2,
                )
                .await,
                Err(StructuredCaptureError::InvalidSituation(_))
            ));
            assert!(matches!(
                save_structured_capture(
                    &repository,
                    input(None, &[("invalid-observation", "\t")], &[]),
                    3,
                )
                .await,
                Err(StructuredCaptureError::InvalidObservation { index: 0, .. })
            ));
            assert!(matches!(
                save_structured_capture(
                    &repository,
                    input(
                        Some(("valid-situation-before-invalid-thought", "Valid context")),
                        &[("valid-observation-before-invalid-thought", "Valid detail")],
                        &[("invalid-thought", "\n")],
                    ),
                    4,
                )
                .await,
                Err(StructuredCaptureError::InvalidThought { index: 0, .. })
            ));

            for table in ["self_subjects", "situations", "observations", "thoughts"] {
                assert_eq!(count(&pool, table).await, 0, "unexpected row in {table}");
            }
            pool.close().await;
        });
    }

    #[test]
    fn existing_subject_is_reused_and_multiple_subjects_are_rejected() {
        tauri::async_runtime::block_on(async {
            let (pool, repository) = test_repository().await;
            let existing =
                SelfSubject::new(SelfSubjectId::new("existing").unwrap(), "Existing").unwrap();
            repository.create_self_subject(&existing, 1).await.unwrap();

            let saved = save_structured_capture(
                &repository,
                input(None, &[], &[("existing-thought", "A confirmed thought")]),
                2,
            )
            .await
            .unwrap();
            assert_eq!(saved.thoughts[0].subject_id(), existing.id());
            assert_eq!(count(&pool, "self_subjects").await, 1);

            let second = SelfSubject::new(SelfSubjectId::new("second").unwrap(), "Second").unwrap();
            repository.create_self_subject(&second, 3).await.unwrap();
            assert!(matches!(
                save_structured_capture(
                    &repository,
                    input(None, &[], &[("blocked-thought", "Must not be saved")]),
                    4,
                )
                .await,
                Err(StructuredCaptureError::Persistence(
                    PersistenceError::SubjectInvariant(_)
                ))
            ));
            assert_eq!(count(&pool, "thoughts").await, 1);
            pool.close().await;
        });
    }

    #[test]
    fn confirmed_third_party_interpretation_is_only_a_thought() {
        tauri::async_runtime::block_on(async {
            let (pool, repository) = test_repository().await;
            let saved = save_structured_capture(
                &repository,
                input(
                    None,
                    &[],
                    &[("third-party-thought", "He doesn't trust me.")],
                ),
                1,
            )
            .await
            .unwrap();

            assert_eq!(saved.thoughts[0].content(), "He doesn't trust me.");
            assert_eq!(saved.thoughts[0].confidence(), None);
            for table in [
                "person_references",
                "emotions",
                "beliefs",
                "values",
                "memories",
                "decisions",
                "outcomes",
                "evidence_links",
            ] {
                assert_eq!(count(&pool, table).await, 0, "unexpected row in {table}");
            }
            pool.close().await;
        });
    }
}
