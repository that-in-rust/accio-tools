use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    create_recommended_action_copy, FindingGroup, OpenAiRuntimeConfig, PatchRequest, PieError,
    PromptPatchMode, OPENAI_MODEL_ID,
};

pub fn ideal_voice_prompt_structure_json() -> &'static str {
    include_str!("../prompts/ideal_voice_prompt_structure.json")
}

pub fn build_patch_request(
    config: &OpenAiRuntimeConfig,
    original_prompt_json: &str,
    groups: &[FindingGroup],
    selected_finding_ids: &[String],
) -> Result<PatchRequest, PieError> {
    if selected_finding_ids.is_empty() {
        return Err(PieError::EmptySelection);
    }

    let selected: HashSet<&str> = selected_finding_ids.iter().map(String::as_str).collect();
    let mut evidence_spans = Vec::new();
    let mut patch_instructions = Vec::new();

    for finding in groups.iter().flat_map(|group| group.findings.iter()) {
        if selected.contains(finding.finding_id.as_str()) {
            evidence_spans.push(finding.prompt_evidence.clone());
            let action = groups
                .iter()
                .find(|group| {
                    group
                        .findings
                        .iter()
                        .any(|item| item.finding_id.as_str() == finding.finding_id.as_str())
                })
                .map(|group| create_recommended_action_copy(&group.section, finding))
                .ok_or_else(|| PieError::InvalidRequest {
                    message: "selected finding did not have a parent group".to_string(),
                })?;
            patch_instructions.push(format!(
                "{}: {}",
                action.better_home, action.recommended_action
            ));
        }
    }

    if evidence_spans.len() != selected_finding_ids.len() {
        return Err(PieError::InvalidRequest {
            message: "one or more selected finding IDs were not returned by analysis".to_string(),
        });
    }

    let prompt_text = extract_general_prompt_text(original_prompt_json)?;
    let (patch_mode, patch_mode_reason) = select_prompt_patch_mode(&prompt_text, &evidence_spans);

    Ok(PatchRequest {
        api_key: config.api_key.clone(),
        model: OPENAI_MODEL_ID.to_string(),
        original_prompt_json: original_prompt_json.to_string(),
        selected_finding_ids: selected_finding_ids.to_vec(),
        evidence_spans,
        patch_instructions,
        patch_mode,
        patch_mode_reason,
        ideal_prompt_structure_json: ideal_voice_prompt_structure_json().to_string(),
    })
}

pub fn create_prompt_candidate_json(request: &PatchRequest) -> Result<String, PieError> {
    if request.patch_mode == PromptPatchMode::CleanScaffold {
        return create_scaffold_candidate_json(request);
    }

    let prompt_replacements = request
        .evidence_spans
        .iter()
        .zip(request.patch_instructions.iter())
        .map(|(evidence_span, instruction)| PromptReplacement {
            find_text: evidence_span.clone(),
            replacement_text: format_recommended_replacement_text(instruction),
        })
        .collect();

    create_prompt_candidate_from_patch(
        &request.original_prompt_json,
        &PromptPatchEnvelope {
            applied_finding_ids: request.selected_finding_ids.clone(),
            patch_summary: "Applied recommended prompt patch.".to_string(),
            prompt_replacements,
            prompt_appendix: Some(format_recommended_patch_block(request)),
            export: None,
        },
    )
}

pub fn create_prompt_candidate_from_patch(
    original_prompt_json: &str,
    patch: &PromptPatchEnvelope,
) -> Result<String, PieError> {
    let mut value: Value = serde_json::from_str(original_prompt_json)?;
    let mut prompt_text = value
        .get("general_prompt")
        .and_then(Value::as_str)
        .ok_or(PieError::MissingRequiredField {
            field: "general_prompt",
        })?
        .to_string();

    if let Some(export) = &patch.export {
        let rebuilt_prompt = export.general_prompt.trim();
        if rebuilt_prompt.is_empty() {
            return Err(PieError::InvalidRequest {
                message: "patch response included an empty rebuilt prompt export".to_string(),
            });
        }

        value["general_prompt"] = Value::String(rebuilt_prompt.to_string());
        return serde_json::to_string_pretty(&value).map_err(PieError::from);
    }

    let mut applied_replacements = 0_usize;
    for replacement in &patch.prompt_replacements {
        if replacement.find_text.trim().is_empty() {
            continue;
        }

        if prompt_text.contains(&replacement.find_text) {
            prompt_text =
                prompt_text.replacen(&replacement.find_text, &replacement.replacement_text, 1);
            applied_replacements += 1;
        }
    }

    if applied_replacements == 0 && patch.prompt_appendix.as_deref().unwrap_or("").is_empty() {
        return Err(PieError::InvalidRequest {
            message: "patch response did not include any applicable prompt replacements"
                .to_string(),
        });
    }

    if let Some(prompt_appendix) = patch.prompt_appendix.as_deref() {
        if !prompt_appendix.trim().is_empty() {
            prompt_text = format!("{}\n\n{}", prompt_text.trim_end(), prompt_appendix.trim());
        }
    }

    value["general_prompt"] = Value::String(prompt_text);
    serde_json::to_string_pretty(&value).map_err(PieError::from)
}

pub fn create_prompt_candidate_from_rewrite(
    original_prompt_json: &str,
    rewritten_prompt: &str,
) -> Result<String, PieError> {
    let mut value: Value = serde_json::from_str(original_prompt_json)?;
    value
        .get("general_prompt")
        .and_then(Value::as_str)
        .ok_or(PieError::MissingRequiredField {
            field: "general_prompt",
        })?;

    let rewritten = rewritten_prompt.trim();
    if rewritten.is_empty() {
        return Err(PieError::InvalidRequest {
            message: "empty rewritten prompt returned by patch engine".to_string(),
        });
    }

    value["general_prompt"] = Value::String(rewritten.to_string());
    serde_json::to_string_pretty(&value).map_err(PieError::from)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptPatchEnvelope {
    pub applied_finding_ids: Vec<String>,
    pub patch_summary: String,
    #[serde(default)]
    pub prompt_replacements: Vec<PromptReplacement>,
    #[serde(default)]
    pub prompt_appendix: Option<String>,
    #[serde(default)]
    pub export: Option<PromptExportCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptExportCandidate {
    #[serde(default)]
    pub agent_name: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    pub general_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptReplacement {
    pub find_text: String,
    pub replacement_text: String,
}

fn format_recommended_replacement_text(instruction: &str) -> String {
    format!(
        "PIE recommended correction: {instruction} Placeholder tools or configs are provider/system integration requirements, not live-tested capabilities."
    )
}

fn format_recommended_patch_block(request: &PatchRequest) -> String {
    let mut output = String::new();
    output.push_str("## PIE Recommended Patch\n\n");
    output.push_str("Applied finding IDs:\n");
    for finding_id in &request.selected_finding_ids {
        output.push_str("- ");
        output.push_str(finding_id);
        output.push('\n');
    }

    output.push_str("\nRecommended prompt/config/tool changes:\n");
    for instruction in &request.patch_instructions {
        output.push_str("- ");
        output.push_str(instruction);
        output.push('\n');
    }

    output.push_str(
        "\nPlaceholder tools or configs above are provider/system integration requirements, not live-tested capabilities.",
    );
    output
}

#[allow(dead_code)]
pub fn create_prompt_candidate_with_appendix(
    original_prompt_json: &str,
    prompt_appendix: &str,
) -> Result<String, PieError> {
    let value: Value = serde_json::from_str(original_prompt_json)?;
    value
        .get("general_prompt")
        .and_then(Value::as_str)
        .ok_or(PieError::MissingRequiredField {
            field: "general_prompt",
        })?;
    create_prompt_candidate_from_patch(
        original_prompt_json,
        &PromptPatchEnvelope {
            applied_finding_ids: Vec::new(),
            patch_summary: "Applied appendix patch.".to_string(),
            prompt_replacements: Vec::new(),
            prompt_appendix: Some(prompt_appendix.to_string()),
            export: None,
        },
    )
}

fn extract_general_prompt_text(original_prompt_json: &str) -> Result<String, PieError> {
    let value: Value = serde_json::from_str(original_prompt_json)?;
    value
        .get("general_prompt")
        .and_then(Value::as_str)
        .filter(|prompt| !prompt.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or(PieError::MissingRequiredField {
            field: "general_prompt",
        })
}

fn select_prompt_patch_mode(
    prompt_text: &str,
    evidence_spans: &[String],
) -> (PromptPatchMode, String) {
    let usable_evidence = evidence_spans
        .iter()
        .filter(|span| !span.trim().is_empty())
        .collect::<Vec<_>>();

    if usable_evidence.is_empty() {
        return (
            PromptPatchMode::CleanScaffold,
            "No usable selected evidence spans were provided; clean scaffold required.".to_string(),
        );
    }

    let matched_evidence_count = usable_evidence
        .iter()
        .filter(|span| prompt_text.contains(span.as_str()))
        .count();

    if matched_evidence_count == usable_evidence.len() {
        (
            PromptPatchMode::TargetedPatch,
            format!(
                "All {matched_evidence_count} selected evidence spans were found in general_prompt."
            ),
        )
    } else if matched_evidence_count > 0 {
        (
            PromptPatchMode::SectionRewrite,
            format!(
                "Only partial evidence anchoring succeeded: {matched_evidence_count}/{} selected spans were found.",
                usable_evidence.len()
            ),
        )
    } else {
        (
            PromptPatchMode::CleanScaffold,
            "No selected evidence spans were found in general_prompt; clean scaffold required."
                .to_string(),
        )
    }
}

fn create_scaffold_candidate_json(request: &PatchRequest) -> Result<String, PieError> {
    let mut value: Value = serde_json::from_str(&request.original_prompt_json)?;
    value
        .get("general_prompt")
        .and_then(Value::as_str)
        .ok_or(PieError::MissingRequiredField {
            field: "general_prompt",
        })?;

    value["general_prompt"] = Value::String(format_clean_scaffold_prompt(request));
    serde_json::to_string_pretty(&value).map_err(PieError::from)
}

fn format_clean_scaffold_prompt(request: &PatchRequest) -> String {
    let mut output = String::new();
    output.push_str("# PIE Clean Scaffold Prompt\n\n");
    output.push_str(
        "This prompt was rebuilt because the selected evidence could not be safely located in the old prompt. Treat clinic facts, tool gaps, and policy-sensitive details as human-review items until confirmed.\n\n",
    );

    write_scaffold_section_text(
        &mut output,
        "Agent Identity and Scope",
        &[
            "Act as the healthcare front-desk voice agent for the configured clinic.",
            "Handle scheduling, confirmation, cancellation, routing, and basic administrative questions only when supported by tools or confirmed policy.",
            "Do not promise unsupported capabilities or live actions unless an available tool performs that action.",
        ],
    );
    write_scaffold_section_text(
        &mut output,
        "Voice and Patient Experience",
        &[
            "Use short, spoken turns suitable for a phone call.",
            "Ask one question at a time and read back names, dates, appointment details, and state-changing actions.",
            "Repair confusion or frustration with a brief apology, a concise restatement, and a safe next step.",
        ],
    );
    write_scaffold_section_text(
        &mut output,
        "Runtime Context",
        &[
            "Use current date, caller metadata, and clinic context only when supplied by the runtime.",
            "Do not hardcode time-sensitive facts that should come from the app, config, or tools.",
        ],
    );
    write_scaffold_section_text(
        &mut output,
        "Workflow Sequences",
        &[
            "For scheduling, verify required patient and visit information before searching slots.",
            "For rescheduling, find and confirm the replacement slot before canceling the old appointment.",
            "For booking, canceling, or confirming, read back the action and wait for explicit caller confirmation before using a write tool.",
        ],
    );
    write_scaffold_section_text(
        &mut output,
        "Tool Contracts and State Changes",
        &[
            "Use lookup tools before making claims about patients, appointments, or availability.",
            "Use write tools only after the caller confirms the exact action.",
            "If a requested capability has no tool, explain the limitation and offer the configured handoff path.",
        ],
    );
    write_scaffold_section_text(
        &mut output,
        "PHI, Identity, and Privacy",
        &[
            "Verify the patient's identity before disclosing PHI-bearing appointment or health-related details.",
            "Use the minimum necessary information and do not disclose patient information to an unverified caller.",
            "If lookup fails or the caller may be the wrong person, ask safe clarifying questions or transfer.",
        ],
    );
    write_scaffold_section_text(
        &mut output,
        "Clinical Safety and Escalation",
        &[
            "Do not diagnose, recommend treatment, change medication guidance, or interpret lab results.",
            "Route medical advice, lab result, urgent, and emergency concerns according to confirmed clinic policy or emergency guidance.",
            "When policy is missing, transfer rather than improvising clinical guidance.",
        ],
    );
    write_scaffold_section_text(
        &mut output,
        "Repair and Fallback",
        &[
            "If the agent misunderstands, ask one clarifying question and continue from the safest confirmed state.",
            "If tools fail or information is missing after reasonable repair, offer transfer or a staff follow-up path if supported.",
        ],
    );

    output.push_str("## Selected Improvements To Preserve\n\n");
    for instruction in &request.patch_instructions {
        output.push_str("- ");
        output.push_str(instruction);
        output.push('\n');
    }

    output.push_str(
        "\nPlaceholder tools or configs above are implementation requirements, not live-tested capabilities.",
    );
    output
}

fn write_scaffold_section_text(output: &mut String, title: &str, bullets: &[&str]) {
    output.push_str("## ");
    output.push_str(title);
    output.push_str("\n\n");

    for bullet in bullets {
        output.push_str("- ");
        output.push_str(bullet);
        output.push('\n');
    }
    output.push('\n');
}
