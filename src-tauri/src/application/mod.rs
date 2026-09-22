mod structured_capture;
mod structured_correction;
mod structured_deletion;
mod structured_history;

pub(crate) use structured_capture::{
    save_structured_capture, ObservationCaptureInput, SaveStructuredCaptureInput,
    SavedStructuredCapture, SituationCaptureInput, StructuredCaptureError, ThoughtCaptureInput,
};
pub(crate) use structured_correction::{
    correct_structured_record, CorrectionInput, StructuredCorrectionError,
};
pub(crate) use structured_deletion::{
    delete_structured_record, DeletionInput, StructuredDeletionError,
};
pub(crate) use structured_history::{
    load_structured_history, StructuredHistory, StructuredHistoryError,
};
