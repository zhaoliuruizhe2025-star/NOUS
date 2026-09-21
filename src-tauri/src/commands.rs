use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{
    application::{
        load_structured_history as load_structured_history_workflow,
        save_structured_capture as save_structured_capture_workflow, ObservationCaptureInput,
        SaveStructuredCaptureInput, SavedStructuredCapture, SituationCaptureInput,
        StructuredCaptureError, StructuredHistory, StructuredHistoryError, ThoughtCaptureInput,
    },
    persistence::{PersistenceError, SharedSqlitePool, SqliteSelfModelRepository},
};

const EXPECTED_SCHEMA_VERSION: &str = "4";
const READINESS_ERROR: &str = "Local database readiness verification failed.";

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatabaseStatus {
    schema_version: u32,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SituationCaptureRequest {
    id: String,
    description: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ObservationCaptureRequest {
    id: String,
    content: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ThoughtCaptureRequest {
    id: String,
    content: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SaveStructuredCaptureRequest {
    situation: Option<SituationCaptureRequest>,
    observations: Vec<ObservationCaptureRequest>,
    thoughts: Vec<ThoughtCaptureRequest>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SituationCaptureResponse {
    id: String,
    description: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObservationCaptureResponse {
    id: String,
    situation_id: Option<String>,
    content: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ThoughtCaptureResponse {
    id: String,
    situation_id: Option<String>,
    content: String,
    confidence: Option<u8>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SavedStructuredCaptureResponse {
    situation: Option<SituationCaptureResponse>,
    observations: Vec<ObservationCaptureResponse>,
    thoughts: Vec<ThoughtCaptureResponse>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StructuredCaptureCommandError {
    code: &'static str,
    item_index: Option<usize>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StructuredHistoryObservationResponse {
    id: String,
    content: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StructuredHistoryThoughtResponse {
    id: String,
    content: String,
    subjective_conviction: Option<u8>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StructuredHistoryContextResponse {
    id: String,
    description: String,
    observations: Vec<StructuredHistoryObservationResponse>,
    thoughts: Vec<StructuredHistoryThoughtResponse>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StructuredHistoryResponse {
    contexts: Vec<StructuredHistoryContextResponse>,
    standalone_observations: Vec<StructuredHistoryObservationResponse>,
    standalone_thoughts: Vec<StructuredHistoryThoughtResponse>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StructuredHistoryCommandError {
    code: &'static str,
}

/// Reports readiness for the one database managed and migrated by the SQL plugin.
///
/// The command deliberately accepts no caller-selected SQL, table, path, or operation.
#[tauri::command]
pub(crate) async fn database_status(
    database: State<'_, SharedSqlitePool>,
) -> Result<DatabaseStatus, &'static str> {
    database_status_for_pool(&database).await
}

#[tauri::command]
pub(crate) async fn save_structured_capture(
    database: State<'_, SharedSqlitePool>,
    request: SaveStructuredCaptureRequest,
) -> Result<SavedStructuredCaptureResponse, StructuredCaptureCommandError> {
    let created_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_millis()).ok())
        .ok_or_else(|| command_error("clockUnavailable", None))?;

    save_structured_capture_for_pool(&database, request, created_at_ms).await
}

/// Loads the fixed read-only Task 008 inspect scope without caller-selected authority or filters.
#[tauri::command]
pub(crate) async fn load_structured_history(
    database: State<'_, SharedSqlitePool>,
) -> Result<StructuredHistoryResponse, StructuredHistoryCommandError> {
    load_structured_history_for_pool(&database).await
}

async fn save_structured_capture_for_pool(
    database: &SharedSqlitePool,
    request: SaveStructuredCaptureRequest,
    created_at_ms: i64,
) -> Result<SavedStructuredCaptureResponse, StructuredCaptureCommandError> {
    let repository = SqliteSelfModelRepository::new(database.clone());
    let saved = save_structured_capture_workflow(&repository, request.into(), created_at_ms)
        .await
        .map_err(map_capture_error)?;
    Ok(saved.into())
}

async fn load_structured_history_for_pool(
    database: &SharedSqlitePool,
) -> Result<StructuredHistoryResponse, StructuredHistoryCommandError> {
    let repository = SqliteSelfModelRepository::new(database.clone());
    let history = load_structured_history_workflow(&repository)
        .await
        .map_err(map_history_error)?;
    Ok(history.into())
}

fn command_error(code: &'static str, item_index: Option<usize>) -> StructuredCaptureCommandError {
    StructuredCaptureCommandError { code, item_index }
}

fn map_capture_error(error: StructuredCaptureError) -> StructuredCaptureCommandError {
    match error {
        StructuredCaptureError::EmptyCapture => command_error("emptyCapture", None),
        StructuredCaptureError::InvalidSituation(_) => command_error("invalidSituation", None),
        StructuredCaptureError::InvalidObservation { index, .. } => {
            command_error("invalidObservation", Some(index))
        }
        StructuredCaptureError::InvalidThought { index, .. } => {
            command_error("invalidThought", Some(index))
        }
        StructuredCaptureError::Persistence(PersistenceError::SubjectInvariant(_)) => {
            command_error("subjectInvariant", None)
        }
        StructuredCaptureError::Persistence(PersistenceError::ConstraintViolation { .. }) => {
            command_error("captureConflict", None)
        }
        StructuredCaptureError::Persistence(PersistenceError::NotReady(_)) => {
            command_error("storageUnavailable", None)
        }
        StructuredCaptureError::Persistence(_) => command_error("saveFailed", None),
    }
}

fn map_history_error(error: StructuredHistoryError) -> StructuredHistoryCommandError {
    let code = match error {
        StructuredHistoryError::DataInconsistent
        | StructuredHistoryError::Persistence(PersistenceError::DomainReconstruction { .. }) => {
            "dataInconsistent"
        }
        StructuredHistoryError::Persistence(PersistenceError::SubjectInvariant(_)) => {
            "subjectInvariant"
        }
        StructuredHistoryError::Persistence(PersistenceError::NotReady(_)) => "storageUnavailable",
        StructuredHistoryError::Persistence(_) => "loadFailed",
    };
    StructuredHistoryCommandError { code }
}

impl From<SaveStructuredCaptureRequest> for SaveStructuredCaptureInput {
    fn from(request: SaveStructuredCaptureRequest) -> Self {
        Self {
            situation: request.situation.map(|situation| SituationCaptureInput {
                id: situation.id,
                description: situation.description,
            }),
            observations: request
                .observations
                .into_iter()
                .map(|observation| ObservationCaptureInput {
                    id: observation.id,
                    content: observation.content,
                })
                .collect(),
            thoughts: request
                .thoughts
                .into_iter()
                .map(|thought| ThoughtCaptureInput {
                    id: thought.id,
                    content: thought.content,
                })
                .collect(),
        }
    }
}

impl From<SavedStructuredCapture> for SavedStructuredCaptureResponse {
    fn from(saved: SavedStructuredCapture) -> Self {
        Self {
            situation: saved.situation.map(|situation| SituationCaptureResponse {
                id: situation.id().as_str().to_owned(),
                description: situation.description().to_owned(),
            }),
            observations: saved
                .observations
                .into_iter()
                .map(|observation| ObservationCaptureResponse {
                    id: observation.id().as_str().to_owned(),
                    situation_id: observation.situation_id().map(|id| id.as_str().to_owned()),
                    content: observation.content().to_owned(),
                })
                .collect(),
            thoughts: saved
                .thoughts
                .into_iter()
                .map(|thought| ThoughtCaptureResponse {
                    id: thought.id().as_str().to_owned(),
                    situation_id: thought.situation_id().map(|id| id.as_str().to_owned()),
                    content: thought.content().to_owned(),
                    confidence: thought.confidence().map(|value| value.value()),
                })
                .collect(),
        }
    }
}

impl From<StructuredHistory> for StructuredHistoryResponse {
    fn from(history: StructuredHistory) -> Self {
        Self {
            contexts: history
                .contexts
                .into_iter()
                .map(|context| StructuredHistoryContextResponse {
                    id: context.id,
                    description: context.description,
                    observations: context
                        .observations
                        .into_iter()
                        .map(|observation| StructuredHistoryObservationResponse {
                            id: observation.id,
                            content: observation.content,
                        })
                        .collect(),
                    thoughts: context
                        .thoughts
                        .into_iter()
                        .map(|thought| StructuredHistoryThoughtResponse {
                            id: thought.id,
                            content: thought.content,
                            subjective_conviction: thought.subjective_conviction,
                        })
                        .collect(),
                })
                .collect(),
            standalone_observations: history
                .standalone_observations
                .into_iter()
                .map(|observation| StructuredHistoryObservationResponse {
                    id: observation.id,
                    content: observation.content,
                })
                .collect(),
            standalone_thoughts: history
                .standalone_thoughts
                .into_iter()
                .map(|thought| StructuredHistoryThoughtResponse {
                    id: thought.id,
                    content: thought.content,
                    subjective_conviction: thought.subjective_conviction,
                })
                .collect(),
        }
    }
}

async fn database_status_for_pool(
    database: &SharedSqlitePool,
) -> Result<DatabaseStatus, &'static str> {
    let mut connection = database
        .acquire_verified_connection()
        .await
        .map_err(|_| READINESS_ERROR)?;
    let schema_version = sqlx::query_scalar::<_, String>(
        "SELECT value FROM app_metadata WHERE key = 'schema_version' LIMIT 1",
    )
    .fetch_optional(&mut **connection.connection())
    .await
    .map_err(|_| READINESS_ERROR)?;

    status_for_schema_version(schema_version.as_deref())
}

fn status_for_schema_version(schema_version: Option<&str>) -> Result<DatabaseStatus, &'static str> {
    if schema_version != Some(EXPECTED_SCHEMA_VERSION) {
        return Err(READINESS_ERROR);
    }

    Ok(DatabaseStatus { schema_version: 4 })
}

#[cfg(test)]
mod tests {
    use super::{
        database_status_for_pool, map_capture_error, map_history_error, status_for_schema_version,
        DatabaseStatus, SaveStructuredCaptureRequest, StructuredCaptureError,
        StructuredHistoryCommandError, StructuredHistoryError, READINESS_ERROR,
    };
    use crate::persistence::{PersistenceError, SharedSqlitePool};
    use sqlx::sqlite::SqlitePoolOptions;

    #[test]
    fn reports_ready_only_for_the_expected_schema_version() {
        assert_eq!(
            status_for_schema_version(Some("4")),
            Ok(DatabaseStatus { schema_version: 4 })
        );
        assert_eq!(status_for_schema_version(Some("3")), Err(READINESS_ERROR));
        assert_eq!(status_for_schema_version(Some("2")), Err(READINESS_ERROR));
        assert_eq!(status_for_schema_version(Some("1")), Err(READINESS_ERROR));
        assert_eq!(status_for_schema_version(None), Err(READINESS_ERROR));
    }

    #[test]
    fn serializes_the_small_frontend_status_shape() {
        assert_eq!(
            serde_json::to_value(DatabaseStatus { schema_version: 4 })
                .expect("database status should serialize"),
            serde_json::json!({ "schemaVersion": 4 })
        );
    }

    #[test]
    fn structured_capture_request_is_strict_and_has_no_inference_or_authority_fields() {
        let valid = serde_json::json!({
            "situation": { "id": "situation-1", "description": "A meeting" },
            "observations": [{ "id": "observation-1", "content": "The meeting ended." }],
            "thoughts": [{ "id": "thought-1", "content": "They may not trust me." }]
        });
        assert!(serde_json::from_value::<SaveStructuredCaptureRequest>(valid.clone()).is_ok());

        for (field, value) in [
            ("subjectId", serde_json::json!("other-subject")),
            ("situationId", serde_json::json!("older-situation")),
            ("rawInput", serde_json::json!("temporary conversation text")),
            ("persistenceIntent", serde_json::json!("explicitSave")),
            ("confidence", serde_json::json!(90)),
        ] {
            let mut invalid = valid.clone();
            invalid.as_object_mut().unwrap().insert(field.into(), value);
            assert!(
                serde_json::from_value::<SaveStructuredCaptureRequest>(invalid).is_err(),
                "unexpectedly accepted {field}"
            );
        }

        let mut thought_confidence = valid;
        thought_confidence["thoughts"][0]["confidence"] = serde_json::json!(80);
        assert!(
            serde_json::from_value::<SaveStructuredCaptureRequest>(thought_confidence).is_err()
        );
    }

    #[test]
    fn structured_capture_errors_expose_only_safe_codes() {
        let error = map_capture_error(StructuredCaptureError::Persistence(
            PersistenceError::Storage("SQL INSERT failed with secret details".into()),
        ));
        let serialized = serde_json::to_string(&error).unwrap();

        assert_eq!(serialized, r#"{"code":"saveFailed","itemIndex":null}"#);
        assert!(!serialized.contains("SQL"));
        assert!(!serialized.contains("secret"));
    }

    #[test]
    fn structured_history_errors_expose_only_safe_codes() {
        for (error, expected) in [
            (
                StructuredHistoryError::Persistence(PersistenceError::SubjectInvariant(
                    "private detail".into(),
                )),
                "subjectInvariant",
            ),
            (
                StructuredHistoryError::Persistence(PersistenceError::DomainReconstruction {
                    entity: "Thought",
                    field: "content",
                    detail: "raw row detail".into(),
                }),
                "dataInconsistent",
            ),
            (
                StructuredHistoryError::Persistence(PersistenceError::NotReady(
                    "database path".into(),
                )),
                "storageUnavailable",
            ),
            (
                StructuredHistoryError::Persistence(PersistenceError::Storage(
                    "SQL SELECT failed".into(),
                )),
                "loadFailed",
            ),
        ] {
            let mapped = map_history_error(error);
            assert_eq!(mapped, StructuredHistoryCommandError { code: expected });
            let serialized = serde_json::to_string(&mapped).unwrap();
            assert!(!serialized.contains("SQL"));
            assert!(!serialized.contains("Thought"));
            assert!(!serialized.contains("path"));
            assert!(!serialized.contains("detail"));
        }
    }

    #[test]
    fn database_failures_and_unready_connections_are_sanitized() {
        tauri::async_runtime::block_on(async {
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect("sqlite::memory:")
                .await
                .expect("readiness test database should open");
            let database = SharedSqlitePool::from_test_pool(pool.clone());

            assert_eq!(
                database_status_for_pool(&database).await,
                Err(READINESS_ERROR)
            );

            sqlx::query(
                "CREATE TABLE app_metadata (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL)",
            )
            .execute(&pool)
            .await
            .unwrap();
            assert_eq!(
                database_status_for_pool(&database).await,
                Err(READINESS_ERROR)
            );

            sqlx::query("INSERT INTO app_metadata (key, value) VALUES ('schema_version', '1')")
                .execute(&pool)
                .await
                .unwrap();
            assert_eq!(
                database_status_for_pool(&database).await,
                Err(READINESS_ERROR)
            );

            sqlx::query("UPDATE app_metadata SET value = '2' WHERE key = 'schema_version'")
                .execute(&pool)
                .await
                .unwrap();
            assert_eq!(
                database_status_for_pool(&database).await,
                Err(READINESS_ERROR)
            );

            sqlx::query("UPDATE app_metadata SET value = '3' WHERE key = 'schema_version'")
                .execute(&pool)
                .await
                .unwrap();
            assert_eq!(
                database_status_for_pool(&database).await,
                Err(READINESS_ERROR)
            );

            sqlx::query("UPDATE app_metadata SET value = '4' WHERE key = 'schema_version'")
                .execute(&pool)
                .await
                .unwrap();
            assert_eq!(
                database_status_for_pool(&database).await,
                Ok(DatabaseStatus { schema_version: 4 })
            );

            let mut connection = pool.acquire().await.unwrap();
            sqlx::query("PRAGMA foreign_keys = OFF")
                .execute(&mut *connection)
                .await
                .unwrap();
            drop(connection);
            assert_eq!(
                database_status_for_pool(&database).await,
                Err(READINESS_ERROR)
            );

            pool.close().await;
        });
    }
}
