use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::State;
use tauri_plugin_dialog::DialogExt;

use crate::{
    application::{
        correct_structured_record as correct_structured_record_workflow,
        create_database_backup_to_path,
        delete_structured_record as delete_structured_record_workflow, export_user_data_to_path,
        load_structured_history as load_structured_history_workflow,
        save_structured_capture as save_structured_capture_workflow, ArtifactError, BackupError,
        CorrectionInput, DeletionInput, ObservationCaptureInput, PortableExportError,
        SaveStructuredCaptureInput, SavedStructuredCapture, SituationCaptureInput,
        StructuredCaptureError, StructuredCorrectionError, StructuredDeletionError,
        StructuredHistory, StructuredHistoryError, ThoughtCaptureInput,
    },
    persistence::{PersistenceError, SharedSqlitePool, SqliteSelfModelRepository},
};

const EXPECTED_SCHEMA_VERSION: &str = "5";
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
    state_token: String,
    corrected: bool,
    corrections: Vec<ObservationCorrectionResponse>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContextReferenceResponse {
    id: String,
    description: String,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SituationCorrectionResponse {
    sequence: u32,
    before_description: String,
    after_description: String,
    note: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObservationCorrectionResponse {
    sequence: u32,
    before_content: String,
    after_content: String,
    before_context: Option<ContextReferenceResponse>,
    after_context: Option<ContextReferenceResponse>,
    note: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ThoughtCorrectionResponse {
    sequence: u32,
    before_content: String,
    after_content: String,
    before_context: Option<ContextReferenceResponse>,
    after_context: Option<ContextReferenceResponse>,
    before_subjective_conviction: Option<u8>,
    after_subjective_conviction: Option<u8>,
    note: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StructuredHistoryThoughtResponse {
    id: String,
    content: String,
    subjective_conviction: Option<u8>,
    state_token: String,
    corrected: bool,
    corrections: Vec<ThoughtCorrectionResponse>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StructuredHistoryContextResponse {
    id: String,
    description: String,
    state_token: String,
    corrected: bool,
    corrections: Vec<SituationCorrectionResponse>,
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

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum CorrectionContextRequest {
    None,
    Existing { situation_id: String },
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum CorrectionConvictionRequest {
    NotReported,
    Reported { value: u8 },
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(
    tag = "recordType",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum CorrectStructuredRecordRequest {
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
        context: CorrectionContextRequest,
        note: Option<String>,
    },
    Thought {
        target_id: String,
        expected_state_token: String,
        content: String,
        context: CorrectionContextRequest,
        subjective_conviction: CorrectionConvictionRequest,
        note: Option<String>,
    },
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StructuredCorrectionCommandError {
    code: &'static str,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(
    tag = "recordType",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub(crate) enum DeleteStructuredRecordRequest {
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

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StructuredDeletionCommandError {
    code: &'static str,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub(crate) struct PortabilityResult {
    status: &'static str,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub(crate) struct PortabilityCommandError {
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

#[tauri::command]
pub(crate) async fn correct_structured_record(
    database: State<'_, SharedSqlitePool>,
    request: CorrectStructuredRecordRequest,
) -> Result<StructuredHistoryResponse, StructuredCorrectionCommandError> {
    let recorded_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_millis()).ok())
        .ok_or(StructuredCorrectionCommandError {
            code: "clockUnavailable",
        })?;
    let repository = SqliteSelfModelRepository::new(database.inner().clone());
    let result = correct_structured_record_workflow(&repository, request.into(), recorded_at_ms)
        .await
        .map_err(map_correction_error)?;
    Ok(result.into())
}

#[tauri::command]
pub(crate) async fn delete_structured_record(
    database: State<'_, SharedSqlitePool>,
    request: DeleteStructuredRecordRequest,
) -> Result<StructuredHistoryResponse, StructuredDeletionCommandError> {
    let repository = SqliteSelfModelRepository::new(database.inner().clone());
    let result = delete_structured_record_workflow(&repository, request.into())
        .await
        .map_err(map_deletion_error)?;
    Ok(result.into())
}

async fn choose_save_destination(
    app: tauri::AppHandle,
    kind: &'static str,
    extension: &'static str,
) -> Result<Option<std::path::PathBuf>, PortabilityCommandError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| PortabilityCommandError {
            code: "destinationUnavailable",
        })?
        .as_millis();
    let filename = format!("nous-{kind}-{now}.{extension}");
    let selected = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .set_file_name(filename)
            .add_filter(extension.to_uppercase(), &[extension])
            .blocking_save_file()
    })
    .await
    .map_err(|_| PortabilityCommandError {
        code: "destinationUnavailable",
    })?;
    selected
        .map(|path| {
            path.into_path().map_err(|_| PortabilityCommandError {
                code: "destinationUnavailable",
            })
        })
        .transpose()
}

#[tauri::command]
pub(crate) async fn export_user_data(
    app: tauri::AppHandle,
    database: State<'_, SharedSqlitePool>,
) -> Result<PortabilityResult, PortabilityCommandError> {
    let Some(destination) = choose_save_destination(app, "export", "json").await? else {
        return Ok(PortabilityResult {
            status: "cancelled",
        });
    };
    let exported_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_millis()).ok())
        .ok_or(PortabilityCommandError {
            code: "exportFailed",
        })?;
    let repository = SqliteSelfModelRepository::new(database.inner().clone());
    export_user_data_to_path(&repository, &destination, exported_at_ms)
        .await
        .map_err(map_export_error)?;
    Ok(PortabilityResult {
        status: "completed",
    })
}

#[tauri::command]
pub(crate) async fn create_database_backup(
    app: tauri::AppHandle,
    database: State<'_, SharedSqlitePool>,
) -> Result<PortabilityResult, PortabilityCommandError> {
    let Some(destination) = choose_save_destination(app, "backup", "sqlite").await? else {
        return Ok(PortabilityResult {
            status: "cancelled",
        });
    };
    let repository = SqliteSelfModelRepository::new(database.inner().clone());
    create_database_backup_to_path(&repository, &destination)
        .await
        .map_err(map_backup_error)?;
    Ok(PortabilityResult {
        status: "completed",
    })
}

fn map_artifact_error(error: ArtifactError) -> &'static str {
    match error {
        ArtifactError::DestinationExists => "destinationExists",
        ArtifactError::DestinationUnavailable => "destinationUnavailable",
        ArtifactError::WriteFailed => "destinationUnavailable",
    }
}

fn map_export_error(error: PortableExportError) -> PortabilityCommandError {
    let code = match error {
        PortableExportError::Persistence(PersistenceError::SubjectInvariant(_)) => {
            "subjectInvariant"
        }
        PortableExportError::Persistence(
            PersistenceError::DataInconsistent(_) | PersistenceError::DomainReconstruction { .. },
        ) => "dataInconsistent",
        PortableExportError::Persistence(PersistenceError::NotReady(_)) => "storageUnavailable",
        PortableExportError::Persistence(_) => "exportFailed",
        PortableExportError::Serialization => "serializationFailed",
        PortableExportError::Artifact(error) => map_artifact_error(error),
    };
    PortabilityCommandError { code }
}

fn map_backup_error(error: BackupError) -> PortabilityCommandError {
    let code = match error {
        BackupError::Storage(PersistenceError::NotReady(_)) => "storageUnavailable",
        BackupError::Storage(_) | BackupError::Validation => "backupFailed",
        BackupError::Artifact(error) => map_artifact_error(error),
    };
    PortabilityCommandError { code }
}

fn map_deletion_error(error: StructuredDeletionError) -> StructuredDeletionCommandError {
    let code = match error {
        StructuredDeletionError::InvalidTarget
        | StructuredDeletionError::Persistence(PersistenceError::NotFound { .. }) => {
            "targetNotFound"
        }
        StructuredDeletionError::Persistence(PersistenceError::StaleDelete) => "staleDelete",
        StructuredDeletionError::Persistence(PersistenceError::DependencyBlocked) => {
            "dependencyBlocked"
        }
        StructuredDeletionError::Persistence(PersistenceError::SubjectInvariant(_)) => {
            "subjectInvariant"
        }
        StructuredDeletionError::Persistence(PersistenceError::DataInconsistent(_))
        | StructuredDeletionError::Persistence(PersistenceError::DomainReconstruction { .. })
        | StructuredDeletionError::History(_) => "dataInconsistent",
        StructuredDeletionError::Persistence(PersistenceError::NotReady(_)) => "storageUnavailable",
        StructuredDeletionError::Persistence(PersistenceError::DeleteConflict(_))
        | StructuredDeletionError::Persistence(PersistenceError::ConstraintViolation { .. }) => {
            "deleteConflict"
        }
        StructuredDeletionError::Persistence(_) => "deleteFailed",
    };
    StructuredDeletionCommandError { code }
}

impl From<DeleteStructuredRecordRequest> for DeletionInput {
    fn from(request: DeleteStructuredRecordRequest) -> Self {
        match request {
            DeleteStructuredRecordRequest::Situation {
                target_id,
                expected_state_token,
            } => Self::Situation {
                target_id,
                expected_state_token,
            },
            DeleteStructuredRecordRequest::Observation {
                target_id,
                expected_state_token,
            } => Self::Observation {
                target_id,
                expected_state_token,
            },
            DeleteStructuredRecordRequest::Thought {
                target_id,
                expected_state_token,
            } => Self::Thought {
                target_id,
                expected_state_token,
            },
        }
    }
}

fn map_correction_error(error: StructuredCorrectionError) -> StructuredCorrectionCommandError {
    let code = match error {
        StructuredCorrectionError::InvalidCorrection
        | StructuredCorrectionError::Persistence(PersistenceError::InvalidCorrection(_)) => {
            "invalidCorrection"
        }
        StructuredCorrectionError::Persistence(PersistenceError::NotFound { .. }) => {
            "targetNotFound"
        }
        StructuredCorrectionError::Persistence(PersistenceError::NoChanges) => "noChanges",
        StructuredCorrectionError::Persistence(PersistenceError::StaleCorrection) => {
            "staleCorrection"
        }
        StructuredCorrectionError::Persistence(PersistenceError::EvidenceReferenceBlocked) => {
            "evidenceReferenceBlocked"
        }
        StructuredCorrectionError::Persistence(PersistenceError::SubjectInvariant(_)) => {
            "subjectInvariant"
        }
        StructuredCorrectionError::Persistence(PersistenceError::DataInconsistent(_))
        | StructuredCorrectionError::Persistence(PersistenceError::DomainReconstruction {
            ..
        })
        | StructuredCorrectionError::History(_) => "dataInconsistent",
        StructuredCorrectionError::Persistence(PersistenceError::NotReady(_)) => {
            "storageUnavailable"
        }
        StructuredCorrectionError::Persistence(PersistenceError::CorrectionConflict(_))
        | StructuredCorrectionError::Persistence(PersistenceError::ConstraintViolation {
            ..
        }) => "correctionConflict",
        StructuredCorrectionError::Persistence(_) => "saveFailed",
    };
    StructuredCorrectionCommandError { code }
}

fn context_id(context: CorrectionContextRequest) -> Option<String> {
    match context {
        CorrectionContextRequest::None => None,
        CorrectionContextRequest::Existing { situation_id } => Some(situation_id),
    }
}

impl From<CorrectStructuredRecordRequest> for CorrectionInput {
    fn from(request: CorrectStructuredRecordRequest) -> Self {
        match request {
            CorrectStructuredRecordRequest::Situation {
                target_id,
                expected_state_token,
                description,
                note,
            } => Self::Situation {
                target_id,
                expected_state_token,
                description,
                note,
            },
            CorrectStructuredRecordRequest::Observation {
                target_id,
                expected_state_token,
                content,
                context,
                note,
            } => Self::Observation {
                target_id,
                expected_state_token,
                content,
                situation_id: context_id(context),
                note,
            },
            CorrectStructuredRecordRequest::Thought {
                target_id,
                expected_state_token,
                content,
                context,
                subjective_conviction,
                note,
            } => Self::Thought {
                target_id,
                expected_state_token,
                content,
                situation_id: context_id(context),
                subjective_conviction: match subjective_conviction {
                    CorrectionConvictionRequest::NotReported => None,
                    CorrectionConvictionRequest::Reported { value } => Some(value),
                },
                note,
            },
        }
    }
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
        | StructuredHistoryError::Persistence(PersistenceError::DataInconsistent(_))
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
                    state_token: context.state_token,
                    corrected: context.corrected,
                    corrections: context
                        .corrections
                        .into_iter()
                        .map(|item| SituationCorrectionResponse {
                            sequence: item.sequence,
                            before_description: item.before_description,
                            after_description: item.after_description,
                            note: item.note,
                        })
                        .collect(),
                    observations: context
                        .observations
                        .into_iter()
                        .map(|observation| StructuredHistoryObservationResponse {
                            id: observation.id,
                            content: observation.content,
                            state_token: observation.state_token,
                            corrected: observation.corrected,
                            corrections: observation
                                .corrections
                                .into_iter()
                                .map(|item| ObservationCorrectionResponse {
                                    sequence: item.sequence,
                                    before_content: item.before_content,
                                    after_content: item.after_content,
                                    before_context: item.before_context.map(|context| {
                                        ContextReferenceResponse {
                                            id: context.id,
                                            description: context.description,
                                        }
                                    }),
                                    after_context: item.after_context.map(|context| {
                                        ContextReferenceResponse {
                                            id: context.id,
                                            description: context.description,
                                        }
                                    }),
                                    note: item.note,
                                })
                                .collect(),
                        })
                        .collect(),
                    thoughts: context
                        .thoughts
                        .into_iter()
                        .map(|thought| StructuredHistoryThoughtResponse {
                            id: thought.id,
                            content: thought.content,
                            subjective_conviction: thought.subjective_conviction,
                            state_token: thought.state_token,
                            corrected: thought.corrected,
                            corrections: thought
                                .corrections
                                .into_iter()
                                .map(|item| ThoughtCorrectionResponse {
                                    sequence: item.sequence,
                                    before_content: item.before_content,
                                    after_content: item.after_content,
                                    before_context: item.before_context.map(|context| {
                                        ContextReferenceResponse {
                                            id: context.id,
                                            description: context.description,
                                        }
                                    }),
                                    after_context: item.after_context.map(|context| {
                                        ContextReferenceResponse {
                                            id: context.id,
                                            description: context.description,
                                        }
                                    }),
                                    before_subjective_conviction: item.before_subjective_conviction,
                                    after_subjective_conviction: item.after_subjective_conviction,
                                    note: item.note,
                                })
                                .collect(),
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
                    state_token: observation.state_token,
                    corrected: observation.corrected,
                    corrections: observation
                        .corrections
                        .into_iter()
                        .map(|item| ObservationCorrectionResponse {
                            sequence: item.sequence,
                            before_content: item.before_content,
                            after_content: item.after_content,
                            before_context: item.before_context.map(|context| {
                                ContextReferenceResponse {
                                    id: context.id,
                                    description: context.description,
                                }
                            }),
                            after_context: item.after_context.map(|context| {
                                ContextReferenceResponse {
                                    id: context.id,
                                    description: context.description,
                                }
                            }),
                            note: item.note,
                        })
                        .collect(),
                })
                .collect(),
            standalone_thoughts: history
                .standalone_thoughts
                .into_iter()
                .map(|thought| StructuredHistoryThoughtResponse {
                    id: thought.id,
                    content: thought.content,
                    subjective_conviction: thought.subjective_conviction,
                    state_token: thought.state_token,
                    corrected: thought.corrected,
                    corrections: thought
                        .corrections
                        .into_iter()
                        .map(|item| ThoughtCorrectionResponse {
                            sequence: item.sequence,
                            before_content: item.before_content,
                            after_content: item.after_content,
                            before_context: item.before_context.map(|context| {
                                ContextReferenceResponse {
                                    id: context.id,
                                    description: context.description,
                                }
                            }),
                            after_context: item.after_context.map(|context| {
                                ContextReferenceResponse {
                                    id: context.id,
                                    description: context.description,
                                }
                            }),
                            before_subjective_conviction: item.before_subjective_conviction,
                            after_subjective_conviction: item.after_subjective_conviction,
                            note: item.note,
                        })
                        .collect(),
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

    Ok(DatabaseStatus { schema_version: 5 })
}

#[cfg(test)]
mod tests {
    use super::{
        database_status_for_pool, map_capture_error, map_correction_error, map_deletion_error,
        map_history_error, status_for_schema_version, CorrectStructuredRecordRequest,
        DatabaseStatus, DeleteStructuredRecordRequest, SaveStructuredCaptureRequest,
        StructuredCaptureError, StructuredCorrectionError, StructuredDeletionError,
        StructuredHistoryCommandError, StructuredHistoryError, READINESS_ERROR,
    };
    use super::{
        map_backup_error, map_export_error, ArtifactError, BackupError, PortableExportError,
    };
    use crate::persistence::{PersistenceError, SharedSqlitePool};
    use sqlx::sqlite::SqlitePoolOptions;

    #[test]
    fn portability_errors_expose_only_safe_categories() {
        let export = map_export_error(PortableExportError::Persistence(
            PersistenceError::DataInconsistent("private row and SQL path".into()),
        ));
        let backup = map_backup_error(BackupError::Storage(PersistenceError::Storage(
            "C:\\private\\database.sqlite".into(),
        )));
        let exists = map_export_error(PortableExportError::Artifact(
            ArtifactError::DestinationExists,
        ));
        assert_eq!(export.code, "dataInconsistent");
        assert_eq!(backup.code, "backupFailed");
        assert_eq!(exists.code, "destinationExists");
        for value in [
            serde_json::to_string(&export).unwrap(),
            serde_json::to_string(&backup).unwrap(),
        ] {
            assert!(!value.contains("private"));
            assert!(!value.contains("SQL"));
            assert!(!value.contains("sqlite"));
        }
    }

    #[test]
    fn reports_ready_only_for_the_expected_schema_version() {
        assert_eq!(
            status_for_schema_version(Some("5")),
            Ok(DatabaseStatus { schema_version: 5 })
        );
        assert_eq!(status_for_schema_version(Some("4")), Err(READINESS_ERROR));
        assert_eq!(status_for_schema_version(Some("3")), Err(READINESS_ERROR));
        assert_eq!(status_for_schema_version(Some("2")), Err(READINESS_ERROR));
        assert_eq!(status_for_schema_version(Some("1")), Err(READINESS_ERROR));
        assert_eq!(status_for_schema_version(None), Err(READINESS_ERROR));
    }

    #[test]
    fn serializes_the_small_frontend_status_shape() {
        assert_eq!(
            serde_json::to_value(DatabaseStatus { schema_version: 5 })
                .expect("database status should serialize"),
            serde_json::json!({ "schemaVersion": 5 })
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
    fn structured_correction_request_is_typed_and_rejects_caller_authority() {
        for valid in [
            serde_json::json!({"recordType":"situation","targetId":"s","expectedStateToken":"initial:situation:s","description":"Thursday","note":null}),
            serde_json::json!({"recordType":"observation","targetId":"o","expectedStateToken":"initial:observation:o","content":"They said they were busy","context":{"kind":"existing","situationId":"s"},"note":null}),
            serde_json::json!({"recordType":"thought","targetId":"t","expectedStateToken":"initial:thought:t","content":"I worried","context":{"kind":"none"},"subjectiveConviction":{"kind":"reported","value":60},"note":null}),
        ] {
            assert!(
                serde_json::from_value::<CorrectStructuredRecordRequest>(valid.clone()).is_ok(),
                "{valid}"
            );
            for field in [
                "subjectId",
                "correctionSequence",
                "auditId",
                "createdAtMs",
                "newStateToken",
                "evidenceLinkId",
                "persistenceIntent",
                "sql",
            ] {
                let mut invalid = valid.clone();
                invalid
                    .as_object_mut()
                    .unwrap()
                    .insert(field.into(), serde_json::json!("forbidden"));
                assert!(
                    serde_json::from_value::<CorrectStructuredRecordRequest>(invalid).is_err(),
                    "accepted {field}"
                );
            }
        }
    }

    #[test]
    fn correction_errors_return_safe_codes_without_storage_details() {
        for (error, code) in [
            (
                StructuredCorrectionError::Persistence(PersistenceError::StaleCorrection),
                "staleCorrection",
            ),
            (
                StructuredCorrectionError::Persistence(PersistenceError::NoChanges),
                "noChanges",
            ),
            (
                StructuredCorrectionError::Persistence(PersistenceError::EvidenceReferenceBlocked),
                "evidenceReferenceBlocked",
            ),
            (
                StructuredCorrectionError::Persistence(PersistenceError::DataInconsistent(
                    "raw row".into(),
                )),
                "dataInconsistent",
            ),
            (
                StructuredCorrectionError::Persistence(PersistenceError::Storage(
                    "SQL path".into(),
                )),
                "saveFailed",
            ),
        ] {
            let mapped = map_correction_error(error);
            assert_eq!(mapped.code, code);
            let serialized = serde_json::to_string(&mapped).unwrap();
            assert!(!serialized.contains("SQL"));
            assert!(!serialized.contains("raw row"));
            assert!(!serialized.contains("path"));
        }
    }

    #[test]
    fn deletion_request_is_typed_and_rejects_caller_authority() {
        for kind in ["situation", "observation", "thought"] {
            let valid = serde_json::json!({"recordType":kind,"targetId":"target","expectedStateToken":"initial:situation:target"});
            assert!(serde_json::from_value::<DeleteStructuredRecordRequest>(valid.clone()).is_ok());
            for field in [
                "subjectId",
                "correctionId",
                "cascade",
                "detach",
                "createdAtMs",
                "sql",
                "deleteScope",
                "situationId",
            ] {
                let mut invalid = valid.clone();
                invalid
                    .as_object_mut()
                    .unwrap()
                    .insert(field.into(), serde_json::json!("forbidden"));
                assert!(
                    serde_json::from_value::<DeleteStructuredRecordRequest>(invalid).is_err(),
                    "accepted {field}"
                );
            }
        }
    }

    #[test]
    fn deletion_errors_expose_only_safe_codes() {
        for (error, expected) in [
            (PersistenceError::StaleDelete, "staleDelete"),
            (PersistenceError::DependencyBlocked, "dependencyBlocked"),
            (
                PersistenceError::SubjectInvariant("private".into()),
                "subjectInvariant",
            ),
            (
                PersistenceError::DataInconsistent("raw row".into()),
                "dataInconsistent",
            ),
            (
                PersistenceError::DeleteConflict("SQL constraint".into()),
                "deleteConflict",
            ),
            (
                PersistenceError::Storage("database path".into()),
                "deleteFailed",
            ),
        ] {
            let mapped = map_deletion_error(StructuredDeletionError::Persistence(error));
            assert_eq!(mapped.code, expected);
            let serialized = serde_json::to_string(&mapped).unwrap();
            assert!(!serialized.contains("SQL"));
            assert!(!serialized.contains("raw row"));
            assert!(!serialized.contains("path"));
        }
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
                Err(READINESS_ERROR)
            );
            sqlx::query("UPDATE app_metadata SET value = '5' WHERE key = 'schema_version'")
                .execute(&pool)
                .await
                .unwrap();
            assert_eq!(
                database_status_for_pool(&database).await,
                Ok(DatabaseStatus { schema_version: 5 })
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
