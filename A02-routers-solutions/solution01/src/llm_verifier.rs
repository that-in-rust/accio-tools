use crate::{LlmVerifyRequest, OPENAI_MODEL_ID};

pub fn build_llm_verify_request(
    api_key: String,
    original_prompt_json: String,
    updated_prompt_json: String,
    selected_finding_ids: Vec<String>,
) -> LlmVerifyRequest {
    LlmVerifyRequest {
        api_key,
        model: OPENAI_MODEL_ID.to_string(),
        original_prompt_json,
        updated_prompt_json,
        selected_finding_ids,
    }
}
