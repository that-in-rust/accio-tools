use crate::{
    AssignmentPrompt, Finding, FindingGroup, FindingSeverity, FixMode, JudgeRequest, JudgeSection,
    NormalizedPrompt, OpenAiRuntimeConfig, PieError, OPENAI_MODEL_ID,
};

pub fn build_judge_request(
    config: &OpenAiRuntimeConfig,
    prompt: &AssignmentPrompt,
    normalized: &NormalizedPrompt,
) -> Result<JudgeRequest, PieError> {
    if normalized.sections.is_empty() {
        return Err(PieError::InvalidRequest {
            message: "normalized prompt has no sections".to_string(),
        });
    }

    Ok(JudgeRequest {
        api_key: config.api_key.clone(),
        model: OPENAI_MODEL_ID.to_string(),
        agent_name: prompt.agent_name.clone(),
        required_sections: required_judge_sections(),
        normalized_sections_count: normalized.sections.len(),
        normalized_prompt: normalized.clone(),
        tool_names: prompt
            .general_tools
            .iter()
            .map(|tool| tool.name.clone())
            .collect(),
    })
}

pub fn required_judge_sections() -> Vec<JudgeSection> {
    vec![
        JudgeSection::Structure,
        JudgeSection::ToolGaps,
        JudgeSection::WorkflowOrder,
        JudgeSection::SafetyPhi,
        JudgeSection::Verification,
    ]
}

pub fn deterministic_finding_groups(prompt: &AssignmentPrompt) -> Vec<FindingGroup> {
    vec![
        FindingGroup {
            section: JudgeSection::Structure,
            findings: vec![Finding {
                finding_id: "finding-structure-dynamic-provider-facts".to_string(),
                title: "Dynamic provider schedules are embedded in prompt prose".to_string(),
                severity: FindingSeverity::Medium,
                prompt_evidence: evidence_or_default(
                    &prompt.general_prompt,
                    "Dr. Priya Patel",
                    "provider schedules and leave windows are prompt text",
                ),
                impact: "Provider availability can change without the prompt being updated."
                    .to_string(),
                suggested_fix: "Flag schedules, leave dates, and provider facts as config or tool-retrieved data."
                    .to_string(),
                fix_mode: FixMode::HumanReviewOnly,
                verification_scenario: "dynamic_provider_facts_embedded".to_string(),
            }],
        },
        FindingGroup {
            section: JudgeSection::ToolGaps,
            findings: vec![Finding {
                finding_id: "finding-tool-waitlist".to_string(),
                title: "Waitlist promise has no durable tool".to_string(),
                severity: FindingSeverity::High,
                prompt_evidence: evidence_or_default(
                    &prompt.general_prompt,
                    "waitlist",
                    "offer to put them on a waitlist",
                ),
                impact: "The caller may believe a state change happened when no waitlist tool exists."
                    .to_string(),
                suggested_fix: "Use honest handoff language unless a waitlist tool exists."
                    .to_string(),
                fix_mode: FixMode::AutoFixable,
                verification_scenario: "waitlist_without_tool".to_string(),
            }],
        },
        FindingGroup {
            section: JudgeSection::WorkflowOrder,
            findings: vec![Finding {
                finding_id: "finding-workflow-reschedule-cancels-first".to_string(),
                title: "Reschedule flow cancels old appointment before replacement is secured"
                    .to_string(),
                severity: FindingSeverity::High,
                prompt_evidence: evidence_or_default(
                    &prompt.general_prompt,
                    "Cancel the old appointment first",
                    "Cancel the old appointment first using cancel_appointment",
                ),
                impact: "A patient can lose their current appointment before the new slot is booked."
                    .to_string(),
                suggested_fix: "Find and confirm the replacement slot before canceling the old appointment."
                    .to_string(),
                fix_mode: FixMode::AutoFixable,
                verification_scenario: "unsafe_reschedule".to_string(),
            }],
        },
        FindingGroup {
            section: JudgeSection::SafetyPhi,
            findings: vec![Finding {
                finding_id: "finding-safety-family-member-verification".to_string(),
                title: "Family-member booking needs explicit authority handling".to_string(),
                severity: FindingSeverity::Medium,
                prompt_evidence: evidence_or_default(
                    &prompt.general_prompt,
                    "family member",
                    "verify each patient separately",
                ),
                impact: "Proxy ambiguity can expose or change patient information incorrectly."
                    .to_string(),
                suggested_fix: "Require separate identity verification and authority handling for each patient."
                    .to_string(),
                fix_mode: FixMode::HumanReviewOnly,
                verification_scenario: "phi_before_verification".to_string(),
            }],
        },
        FindingGroup {
            section: JudgeSection::Verification,
            findings: vec![Finding {
                finding_id: "finding-verification-scenarios-missing".to_string(),
                title: "Prompt lacks before/after verification scenarios".to_string(),
                severity: FindingSeverity::Medium,
                prompt_evidence: "No explicit scenario pack is present in the prompt.".to_string(),
                impact: "The team cannot prove that selected prompt fixes improved behavior."
                    .to_string(),
                suggested_fix: "Add verification scenarios for waitlist, reschedule, PHI, emergency, date-format, and dynamic facts."
                    .to_string(),
                fix_mode: FixMode::Backlog,
                verification_scenario: "missing_before_after_evals".to_string(),
            }],
        },
    ]
}

fn evidence_or_default(prompt_text: &str, needle: &str, fallback: &str) -> String {
    prompt_text
        .split("\n\n")
        .find(|paragraph| {
            paragraph
                .to_ascii_lowercase()
                .contains(&needle.to_ascii_lowercase())
        })
        .map(str::trim)
        .filter(|paragraph| !paragraph.is_empty())
        .map(|paragraph| paragraph.chars().take(220).collect())
        .unwrap_or_else(|| fallback.to_string())
}
