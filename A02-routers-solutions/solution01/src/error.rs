use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PieError {
    #[error("invalid assignment prompt JSON: {message}")]
    InvalidPromptJson { message: String },
    #[error("missing required field: {field}")]
    MissingRequiredField { field: &'static str },
    #[error("empty selected-fix submission")]
    EmptySelection,
    #[error("model gpt-5.4-mini is unavailable for the provided API key")]
    ModelUnavailable,
    #[error("OpenAI API key is invalid or unauthorized")]
    InvalidApiKey,
    #[error("OpenAI quota or rate limit blocked the request")]
    QuotaLimited,
    #[error("OpenAI request could not reach the API")]
    NetworkUnavailable,
    #[error("OpenAI response was incomplete: {reason}")]
    OpenAiIncomplete { reason: String },
    #[error("OpenAI key is missing")]
    MissingApiKey,
    #[error("patch failed for selected finding: {finding_id}")]
    PatchFailed { finding_id: String },
    #[error("SQLite error: {message}")]
    Sqlite { message: String },
    #[error("I/O error: {message}")]
    Io { message: String },
    #[error("serialization error: {message}")]
    Serialization { message: String },
    #[error("verification error: {message}")]
    Verification { message: String },
    #[error("invalid request: {message}")]
    InvalidRequest { message: String },
}

impl PieError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidPromptJson { .. } | Self::MissingRequiredField { .. } => {
                "invalid_prompt_json"
            }
            Self::EmptySelection => "empty_selection",
            Self::ModelUnavailable => "model_unavailable",
            Self::InvalidApiKey => "invalid_api_key",
            Self::QuotaLimited => "quota_limited",
            Self::NetworkUnavailable => "network_unavailable",
            Self::OpenAiIncomplete { .. } => "openai_incomplete",
            Self::MissingApiKey => "missing_api_key",
            Self::PatchFailed { .. } => "patch_failed",
            Self::Sqlite { .. } => "sqlite_error",
            Self::Io { .. } => "io_error",
            Self::Serialization { .. } => "serialization_error",
            Self::Verification { .. } => "verification_error",
            Self::InvalidRequest { .. } => "invalid_request",
        }
    }

    fn hint(&self) -> &'static str {
        match self {
            Self::InvalidPromptJson { .. } | Self::MissingRequiredField { .. } => {
                "Upload assignment-compatible JSON with agent_name, model, general_prompt, and general_tools."
            }
            Self::EmptySelection => "Select at least one checkbox before implementing fixes.",
            Self::ModelUnavailable => {
                "Use an API key that can access gpt-5.4-mini, then retry readiness."
            }
            Self::InvalidApiKey => "Replace the local API key and retry readiness.",
            Self::QuotaLimited => "Check quota, billing, or rate limits, then retry.",
            Self::NetworkUnavailable => "Check network access to OpenAI, then retry.",
            Self::OpenAiIncomplete { .. } => {
                "Retry with a smaller selection or a larger output allowance."
            }
            Self::MissingApiKey => "Enter an OpenAI API key locally before analysis.",
            Self::PatchFailed { .. } => "Retry the selected fix or export findings for manual review.",
            Self::Sqlite { .. } => "Check local storage readiness and retry.",
            Self::Io { .. } => "Check local file permissions and retry.",
            Self::Serialization { .. } => "Check the prompt JSON shape and retry.",
            Self::Verification { .. } => "Review the verification report and retry with a valid prompt.",
            Self::InvalidRequest { .. } => "Correct the request and retry.",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub hint: String,
}

impl From<PieError> for AppError {
    fn from(error: PieError) -> Self {
        Self {
            code: error.code().to_string(),
            message: error.to_string(),
            hint: error.hint().to_string(),
        }
    }
}

impl From<rusqlite::Error> for PieError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite {
            message: error.to_string(),
        }
    }
}

impl From<serde_json::Error> for PieError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization {
            message: error.to_string(),
        }
    }
}

impl From<std::io::Error> for PieError {
    fn from(error: std::io::Error) -> Self {
        Self::Io {
            message: error.to_string(),
        }
    }
}
