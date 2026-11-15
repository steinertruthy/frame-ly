use serde::{ser::SerializeStruct, Serialize, Serializer};

/// Every fallible operation in the core returns this error. It serializes to
/// `{ code, message }` so the UI can translate by `code` and log `message`.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid settings: {0}")]
    InvalidSettings(String),
    #[error("Window or runtime error: {0}")]
    Runtime(String),
    #[error("Permission required: {0}")]
    PermissionDenied(&'static str),
    #[error("System API error: {0}")]
    Platform(String),
    #[error("Nothing was recorded")]
    EmptyRecording,
    #[error("Item not found")]
    NotFound,
    #[error("Invalid file name: {0}")]
    InvalidName(String),
    #[error("Blocked by local-only mode")]
    LocalOnly,
    #[error("Consent required to send content to {0}")]
    ConsentRequired(String),
    #[error("Model not installed: {0}")]
    ModelMissing(String),
    #[error("Cancelled")]
    Cancelled,
    #[error("The file has no audio")]
    NoAudio,
    #[error("Not supported: {0}")]
    Unsupported(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Not enough disk space")]
    DiskFull,
    #[error("Integrity check failed: {0}")]
    Integrity(String),
    #[error("Missing API key")]
    MissingKey,
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Io(_) => "io",
            AppError::Json(_) => "json",
            AppError::InvalidSettings(_) => "invalid_settings",
            AppError::Runtime(_) => "runtime",
            AppError::PermissionDenied(_) => "permission_denied",
            AppError::Platform(_) => "platform",
            AppError::EmptyRecording => "empty_recording",
            AppError::NotFound => "not_found",
            AppError::InvalidName(_) => "invalid_name",
            AppError::LocalOnly => "local_only",
            AppError::ConsentRequired(_) => "consent_required",
            AppError::ModelMissing(_) => "model_missing",
            AppError::Cancelled => "cancelled",
            AppError::NoAudio => "no_audio",
            AppError::Unsupported(_) => "unsupported",
            AppError::Network(_) => "network",
            AppError::Provider(_) => "provider",
            AppError::DiskFull => "disk_full",
            AppError::Integrity(_) => "integrity",
            AppError::MissingKey => "missing_key",
        }
    }
}

impl From<tauri::Error> for AppError {
    fn from(e: tauri::Error) -> Self {
        AppError::Runtime(e.to_string())
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut st = s.serialize_struct("AppError", 2)?;
        st.serialize_field("code", self.code())?;
        st.serialize_field("message", &self.to_string())?;
        st.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_code_and_message() {
        let e = AppError::InvalidSettings("bad".into());
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["code"], "invalid_settings");
        assert_eq!(v["message"], "Invalid settings: bad");
    }
}
