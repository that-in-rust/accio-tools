use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    create_prompt_candidate_from_rewrite, create_prompt_candidate_json,
    deterministic_finding_groups, Finding, FindingSeverity, LlmVerifyRequest, PatchCandidate,
    PatchFailureMode, PatchRequest, PieError, ReverifyFindingStatus, ReverifyPromptRequest,
    ReverifyPromptResult, ReverifyStatus, SemanticCheck,
};
use crate::{FindingGroup, JudgeRequest, JudgeSection, OpenAiRuntimeConfig};

const PATCH_PROMPT_TOKEN_BUDGET: u16 = 12_000;

pub fn judge_prompt_instructions() -> &'static str {
    include_str!("../prompts/llm-judge.md")
}

pub fn fix_patch_instructions() -> &'static str {
    include_str!("../prompts/fix-patch.md")
}

pub fn reverify_prompt_instructions() -> &'static str {
    include_str!("../prompts/reverify-prompt.md")
}

#[async_trait]
pub trait OpenAiClient: Send + Sync {
    async fn validate_key_with_model(&self, config: &OpenAiRuntimeConfig) -> Result<(), PieError>;

    async fn judge_prompt_quality(
        &self,
        request: JudgeRequest,
    ) -> Result<Vec<FindingGroup>, PieError>;

    async fn patch_selected_prompt(
        &self,
        request: PatchRequest,
    ) -> Result<PatchCandidate, PieError>;

    async fn verify_semantic_update(
        &self,
        request: LlmVerifyRequest,
    ) -> Result<Vec<SemanticCheck>, PieError>;

    async fn reverify_prompt_history(
        &self,
        request: ReverifyPromptRequest,
    ) -> Result<ReverifyPromptResult, PieError>;
}

#[derive(Debug, Clone)]
pub struct OpenAiHttpClient {
    http: reqwest::Client,
    endpoint: String,
}

impl OpenAiHttpClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            endpoint: "https://api.openai.com/v1/responses".to_string(),
        }
    }

    pub fn with_endpoint(endpoint: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            endpoint: endpoint.into(),
        }
    }

    async fn create_openai_response_text(
        &self,
        config: &OpenAiRuntimeConfig,
        input: serde_json::Value,
    ) -> Result<String, PieError> {
        let response = self
            .http
            .post(&self.endpoint)
            .bearer_auth(&config.api_key)
            .json(&input)
            .send()
            .await
            .map_err(|_| PieError::NetworkUnavailable)?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|_| PieError::NetworkUnavailable)?;

        if !status.is_success() {
            return Err(map_openai_status_error(
                status.as_u16(),
                &response_text,
                &config.api_key,
            ));
        }

        extract_response_output_text(&response_text)
    }

    async fn create_structured_response_text(
        &self,
        config: &OpenAiRuntimeConfig,
        instructions: &str,
        user_content: String,
        max_output_tokens: u16,
    ) -> Result<String, PieError> {
        self.create_openai_response_text(
            config,
            json!({
                "model": config.model,
                "input": [
                    {
                        "role": "developer",
                        "content": instructions
                    },
                    {
                        "role": "user",
                        "content": user_content
                    }
                ],
                "max_output_tokens": max_output_tokens,
                "text": {
                    "format": {
                        "type": "json_object"
                    }
                }
            }),
        )
        .await
    }
}

impl Default for OpenAiHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OpenAiClient for OpenAiHttpClient {
    async fn validate_key_with_model(&self, config: &OpenAiRuntimeConfig) -> Result<(), PieError> {
        let input = json!({
            "model": config.model,
            "input": "Reply with OK.",
            "reasoning": {
                "effort": "none"
            },
            "text": {
                "verbosity": "low"
            },
            "max_output_tokens": 16
        });
        self.create_openai_response_text(config, input)
            .await
            .map(|_| ())
    }

    async fn judge_prompt_quality(
        &self,
        request: JudgeRequest,
    ) -> Result<Vec<FindingGroup>, PieError> {
        let config = OpenAiRuntimeConfig {
            api_key: request.api_key.clone(),
            model: request.model.clone(),
        };
        let payload = serde_json::to_string(&request).map_err(PieError::from)?;
        let text = self
            .create_structured_response_text(&config, judge_prompt_instructions(), payload, 4000)
            .await?;
        parse_judge_response_text(&text)
    }

    async fn patch_selected_prompt(
        &self,
        request: PatchRequest,
    ) -> Result<PatchCandidate, PieError> {
        let text = self
            .create_openai_response_text(
                &OpenAiRuntimeConfig {
                    api_key: request.api_key.clone(),
                    model: request.model.clone(),
                },
                json!({
                    "model": request.model.clone(),
                    "input": [
                        {
                            "role": "developer",
                            "content": fix_patch_instructions()
                        },
                        {
                            "role": "user",
                            "content": serde_json::to_string(&request).map_err(PieError::from)?
                        }
                    ],
                    "max_output_tokens": PATCH_PROMPT_TOKEN_BUDGET
                }),
            )
            .await?;
        let updated_prompt_json =
            create_prompt_candidate_from_rewrite(&request.original_prompt_json, &text)?;

        Ok(PatchCandidate {
            updated_prompt_json,
            applied_finding_ids: request.selected_finding_ids,
            patch_summary: "Applied selected prompt rewrite.".to_string(),
        })
    }

    async fn verify_semantic_update(
        &self,
        request: LlmVerifyRequest,
    ) -> Result<Vec<SemanticCheck>, PieError> {
        let text = self
            .create_structured_response_text(
                &OpenAiRuntimeConfig {
                    api_key: request.api_key.clone(),
                    model: request.model.clone(),
                },
                "You are PIE's semantic verifier. Return JSON with key semantic_checks. Only judge tone, clarity, patient experience, semantic fix quality, and confusing-language risk.",
                serde_json::to_string(&request).map_err(PieError::from)?,
                1600,
            )
            .await?;
        let parsed: SemanticResponseEnvelope =
            serde_json::from_str(&text).map_err(PieError::from)?;
        Ok(parsed.semantic_checks)
    }

    async fn reverify_prompt_history(
        &self,
        request: ReverifyPromptRequest,
    ) -> Result<ReverifyPromptResult, PieError> {
        let text = self
            .create_structured_response_text(
                &OpenAiRuntimeConfig {
                    api_key: request.api_key.clone(),
                    model: request.model(),
                },
                reverify_prompt_instructions(),
                serde_json::to_string(&request).map_err(PieError::from)?,
                2800,
            )
            .await?;
        parse_reverify_response_text(&text, &request)
    }
}

#[derive(Debug, Clone)]
pub struct FakeOpenAiClient {
    mode: FakeOpenAiMode,
}

#[derive(Debug, Clone)]
enum FakeOpenAiMode {
    Ready,
    ModelUnavailable,
    PatchFailure(PatchFailureMode),
}

impl FakeOpenAiClient {
    pub fn ready() -> Self {
        Self {
            mode: FakeOpenAiMode::Ready,
        }
    }

    pub fn model_unavailable() -> Self {
        Self {
            mode: FakeOpenAiMode::ModelUnavailable,
        }
    }

    pub fn patch_failure(mode: PatchFailureMode) -> Self {
        Self {
            mode: FakeOpenAiMode::PatchFailure(mode),
        }
    }
}

#[async_trait]
impl OpenAiClient for FakeOpenAiClient {
    async fn validate_key_with_model(&self, _config: &OpenAiRuntimeConfig) -> Result<(), PieError> {
        match &self.mode {
            FakeOpenAiMode::ModelUnavailable => Err(PieError::ModelUnavailable),
            FakeOpenAiMode::Ready | FakeOpenAiMode::PatchFailure(_) => Ok(()),
        }
    }

    async fn judge_prompt_quality(
        &self,
        request: JudgeRequest,
    ) -> Result<Vec<FindingGroup>, PieError> {
        if matches!(self.mode, FakeOpenAiMode::ModelUnavailable) {
            return Err(PieError::ModelUnavailable);
        }
        Ok(deterministic_finding_groups_from_request(&request))
    }

    async fn patch_selected_prompt(
        &self,
        request: PatchRequest,
    ) -> Result<PatchCandidate, PieError> {
        match &self.mode {
            FakeOpenAiMode::ModelUnavailable => Err(PieError::ModelUnavailable),
            FakeOpenAiMode::PatchFailure(PatchFailureMode::SelectedFindingFailed {
                finding_id,
            }) => Err(PieError::PatchFailed {
                finding_id: finding_id.clone(),
            }),
            FakeOpenAiMode::Ready => {
                let updated_prompt_json = create_prompt_candidate_json(&request)?;
                Ok(PatchCandidate {
                    updated_prompt_json,
                    applied_finding_ids: request.selected_finding_ids,
                    patch_summary: "Applied selected prompt fixes only.".to_string(),
                })
            }
        }
    }

    async fn verify_semantic_update(
        &self,
        request: LlmVerifyRequest,
    ) -> Result<Vec<SemanticCheck>, PieError> {
        if matches!(self.mode, FakeOpenAiMode::ModelUnavailable) {
            return Err(PieError::ModelUnavailable);
        }
        Ok(vec![SemanticCheck {
            scenario_id: "semantic_patient_experience".to_string(),
            passed: !request.selected_finding_ids.is_empty(),
            message: "Semantic verification is limited to tone, clarity, and patient experience."
                .to_string(),
        }])
    }

    async fn reverify_prompt_history(
        &self,
        request: ReverifyPromptRequest,
    ) -> Result<ReverifyPromptResult, PieError> {
        if matches!(self.mode, FakeOpenAiMode::ModelUnavailable) {
            return Err(PieError::ModelUnavailable);
        }

        Ok(create_deterministic_reverify_result(&request))
    }
}

fn deterministic_finding_groups_from_request(request: &JudgeRequest) -> Vec<FindingGroup> {
    let tools = request
        .tool_names
        .iter()
        .map(|name| crate::ToolDefinition {
            tool_type: "custom".to_string(),
            name: name.clone(),
            description: String::new(),
            method: "POST".to_string(),
            url: String::new(),
            headers: serde_json::Value::Object(serde_json::Map::new()),
            parameters: crate::model::ToolParameters {
                required: Vec::new(),
                properties: serde_json::Value::Object(serde_json::Map::new()),
            },
        })
        .collect();

    deterministic_finding_groups(&crate::AssignmentPrompt {
        agent_name: request.agent_name.clone(),
        model: request.model.clone(),
        general_prompt:
            "waitlist\nCancel the old appointment first using cancel_appointment\nfamily member"
                .to_string(),
        general_tools: tools,
    })
}

pub fn map_openai_status_error(status_code: u16, body: &str, api_key: &str) -> PieError {
    let sanitized = if api_key.is_empty() {
        body.to_string()
    } else {
        body.replace(api_key, "[redacted]")
    };
    let provider_message = extract_openai_error_message(&sanitized)
        .unwrap_or_else(|| fallback_provider_message(&sanitized));
    let body_lower = sanitized.to_ascii_lowercase();
    match status_code {
        401 | 403 => PieError::InvalidApiKey,
        400 | 404
            if body_lower.contains("model")
                || body_lower.contains("gpt-5.4-mini")
                || body_lower.contains("does not exist") =>
        {
            PieError::ModelUnavailable
        }
        429 => PieError::QuotaLimited,
        408 | 500..=599 => PieError::NetworkUnavailable,
        _ => PieError::InvalidRequest {
            message: format!("OpenAI returned HTTP {status_code}: {provider_message}"),
        },
    }
}

fn extract_openai_error_message(body: &str) -> Option<String> {
    serde_json::from_str::<OpenAiErrorEnvelope>(body)
        .ok()
        .and_then(|envelope| envelope.error)
        .map(|error| error.message)
        .filter(|message| !message.trim().is_empty())
}

fn fallback_provider_message(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return "OpenAI rejected the request.".to_string();
    }

    trimmed.chars().take(500).collect()
}

fn extract_response_output_text(response_text: &str) -> Result<String, PieError> {
    let parsed: OpenAiResponseBody = serde_json::from_str(response_text).map_err(PieError::from)?;
    if parsed.status.as_deref() == Some("incomplete") {
        let reason = parsed
            .incomplete_details
            .and_then(|details| details.reason)
            .unwrap_or_else(|| "unknown".to_string());
        return Err(PieError::OpenAiIncomplete { reason });
    }

    parsed
        .output
        .into_iter()
        .flat_map(|item| item.content)
        .find_map(|content| match content {
            OpenAiResponseContent::OutputText { text } => Some(text),
            OpenAiResponseContent::Other => None,
        })
        .ok_or_else(|| PieError::InvalidRequest {
            message: "OpenAI response did not contain output text.".to_string(),
        })
}

#[derive(Debug, Deserialize)]
struct OpenAiResponseBody {
    status: Option<String>,
    incomplete_details: Option<OpenAiIncompleteDetails>,
    #[serde(default)]
    output: Vec<OpenAiResponseOutput>,
}

#[derive(Debug, Deserialize)]
struct OpenAiIncompleteDetails {
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponseOutput {
    #[serde(default)]
    content: Vec<OpenAiResponseContent>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum OpenAiResponseContent {
    #[serde(rename = "output_text")]
    OutputText { text: String },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
struct SemanticResponseEnvelope {
    semantic_checks: Vec<SemanticCheck>,
}

#[derive(Debug, Deserialize)]
struct ReverifyResponseEnvelope {
    finding_statuses: Vec<ReverifyFindingStatus>,
}

#[derive(Debug, Deserialize)]
struct OpenAiErrorEnvelope {
    error: Option<OpenAiErrorMessage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiErrorMessage {
    message: String,
}

pub fn parse_judge_response_text(text: &str) -> Result<Vec<FindingGroup>, PieError> {
    let parsed: Value = serde_json::from_str(text).map_err(PieError::from)?;
    let groups = parsed
        .get("finding_groups")
        .and_then(Value::as_array)
        .ok_or_else(|| PieError::Serialization {
            message: "judge response missing finding_groups array".to_string(),
        })?;

    let mut canonical_groups = required_judge_groups_empty();
    for group in groups {
        let Some(group_object) = group.as_object() else {
            continue;
        };
        let section = group_object
            .get("section")
            .and_then(value_to_string)
            .map(parse_judge_section_value)
            .transpose()?
            .unwrap_or(JudgeSection::Structure);
        let findings = group_object
            .get("findings")
            .and_then(Value::as_array)
            .map(|values| parse_finding_values(values))
            .transpose()?
            .unwrap_or_default();

        let inferred_section = infer_section_from_findings(section, &findings);

        if let Some(canonical_group) = canonical_groups
            .iter_mut()
            .find(|candidate| candidate.section == inferred_section)
        {
            canonical_group.findings.extend(findings);
        }
    }

    Ok(canonical_groups)
}

fn required_judge_groups_empty() -> Vec<FindingGroup> {
    vec![
        FindingGroup {
            section: JudgeSection::Structure,
            findings: Vec::new(),
        },
        FindingGroup {
            section: JudgeSection::ToolGaps,
            findings: Vec::new(),
        },
        FindingGroup {
            section: JudgeSection::WorkflowOrder,
            findings: Vec::new(),
        },
        FindingGroup {
            section: JudgeSection::SafetyPhi,
            findings: Vec::new(),
        },
        FindingGroup {
            section: JudgeSection::Verification,
            findings: Vec::new(),
        },
    ]
}

fn parse_finding_values(values: &[Value]) -> Result<Vec<Finding>, PieError> {
    values
        .iter()
        .filter_map(Value::as_object)
        .map(|object| {
            Ok(Finding {
                finding_id: required_string_field(object, "finding_id")?,
                title: required_string_field(object, "title")?,
                severity: parse_finding_severity_value(&required_string_field(
                    object, "severity",
                )?)?,
                prompt_evidence: required_string_field(object, "prompt_evidence")?,
                impact: required_string_field(object, "impact")?,
                suggested_fix: required_string_field(object, "suggested_fix")?,
                fix_mode: parse_fix_mode_value(&required_string_field(object, "fix_mode")?)?,
                verification_scenario: required_string_field(object, "verification_scenario")?,
            })
        })
        .collect()
}

fn infer_section_from_findings(section: JudgeSection, findings: &[Finding]) -> JudgeSection {
    let joined = findings
        .iter()
        .map(|finding| {
            format!(
                "{} {} {}",
                finding.finding_id, finding.title, finding.verification_scenario
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();

    if joined.contains("tool")
        || joined.contains("waitlist")
        || joined.contains("callback")
        || joined.contains("interpreter")
    {
        JudgeSection::ToolGaps
    } else if joined.contains("workflow")
        || joined.contains("reschedule")
        || joined.contains("cancel")
    {
        JudgeSection::WorkflowOrder
    } else if joined.contains("safety")
        || joined.contains("phi")
        || joined.contains("hipaa")
        || joined.contains("emergency")
    {
        JudgeSection::SafetyPhi
    } else if joined.contains("verification")
        || joined.contains("scenario")
        || joined.contains("eval")
    {
        JudgeSection::Verification
    } else {
        section
    }
}

fn required_string_field(
    object: &serde_json::Map<String, Value>,
    field_name: &'static str,
) -> Result<String, PieError> {
    object
        .get(field_name)
        .and_then(value_to_string)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| PieError::Serialization {
            message: format!("judge finding missing required field {field_name}"),
        })
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(boolean) => Some(boolean.to_string()),
        _ => None,
    }
}

fn parse_judge_section_value(value: String) -> Result<JudgeSection, PieError> {
    serde_json::from_value(Value::String(value)).map_err(PieError::from)
}

fn parse_finding_severity_value(value: &str) -> Result<FindingSeverity, PieError> {
    serde_json::from_value(Value::String(value.to_string())).map_err(PieError::from)
}

fn parse_fix_mode_value(value: &str) -> Result<crate::FixMode, PieError> {
    serde_json::from_value(Value::String(value.to_string())).map_err(PieError::from)
}

pub fn parse_reverify_response_text(
    text: &str,
    request: &ReverifyPromptRequest,
) -> Result<ReverifyPromptResult, PieError> {
    let parsed: ReverifyResponseEnvelope = serde_json::from_str(text).map_err(PieError::from)?;
    Ok(ReverifyPromptResult {
        prompt_version_name: request.updated_version_name.clone(),
        model_label: crate::model_label_text(),
        finding_statuses: parsed
            .finding_statuses
            .into_iter()
            .map(|status| ReverifyFindingStatus {
                status_label: status.status.as_label().to_string(),
                ..status
            })
            .collect(),
        action_log: vec![format!(
            "Compared updated prompt {} against previous findings.",
            request.updated_version_name
        )],
    })
}

fn create_deterministic_reverify_result(request: &ReverifyPromptRequest) -> ReverifyPromptResult {
    let updated_prompt_lower = request.updated_prompt_json.to_ascii_lowercase();
    let finding_statuses = request
        .previous_finding_groups
        .iter()
        .flat_map(|group| group.findings.iter())
        .filter(|finding| finding.fix_mode != crate::FixMode::Backlog)
        .map(|finding| {
            let evidence = finding.prompt_evidence.to_ascii_lowercase();
            let status = if !evidence.trim().is_empty() && !updated_prompt_lower.contains(&evidence)
            {
                ReverifyStatus::Fixed
            } else {
                ReverifyStatus::StillFailing
            };
            ReverifyFindingStatus {
                finding_id: finding.finding_id.clone(),
                status_label: status.as_label().to_string(),
                rationale: match status {
                    ReverifyStatus::Fixed => {
                        "The previous evidence span no longer appears in the updated prompt."
                            .to_string()
                    }
                    ReverifyStatus::StillFailing => {
                        "The previous evidence span still appears in the updated prompt."
                            .to_string()
                    }
                    ReverifyStatus::Unknown => {
                        "The reverify pass could not determine this finding status.".to_string()
                    }
                },
                status,
            }
        })
        .collect();

    ReverifyPromptResult {
        prompt_version_name: request.updated_version_name.clone(),
        model_label: crate::model_label_text(),
        finding_statuses,
        action_log: vec![format!(
            "Compared updated prompt {} against previous findings.",
            request.updated_version_name
        )],
    }
}

impl ReverifyPromptRequest {
    fn model(&self) -> String {
        crate::OPENAI_MODEL_ID.to_string()
    }
}
