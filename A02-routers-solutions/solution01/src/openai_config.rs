use crate::model::OpenAiRuntimeConfig;

pub const OPENAI_MODEL_ID: &str = "gpt-5.4-mini";

pub fn model_label_text() -> String {
    format!("Model: {OPENAI_MODEL_ID}")
}

impl OpenAiRuntimeConfig {
    pub fn from_api_key(api_key: String) -> Self {
        Self {
            api_key,
            model: OPENAI_MODEL_ID.to_string(),
        }
    }
}
