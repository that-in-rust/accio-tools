use crate::{
    AssignmentPrompt, JudgeSection, NormalizedPrompt, PieError, PromptParagraph, PromptSection,
};

struct SectionTemplate {
    section_id: &'static str,
    section_name: &'static str,
    better_home: &'static str,
    stability: &'static str,
    judge_section: JudgeSection,
    belongs_in_final_prompt: bool,
    keywords: &'static [&'static str],
}

pub fn normalize_prompt_sections_model(
    prompt: &AssignmentPrompt,
) -> Result<NormalizedPrompt, PieError> {
    let paragraphs = split_prompt_paragraphs(&prompt.general_prompt);
    if paragraphs.is_empty() {
        return Err(PieError::MissingRequiredField {
            field: "general_prompt",
        });
    }

    let sections = section_templates()
        .into_iter()
        .map(|template| build_prompt_section(template, &paragraphs))
        .collect();

    Ok(NormalizedPrompt {
        raw_text: prompt.general_prompt.clone(),
        sections,
    })
}

fn split_prompt_paragraphs(raw_text: &str) -> Vec<String> {
    raw_text
        .split("\n\n")
        .map(str::trim)
        .filter(|paragraph| !paragraph.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn build_prompt_section(template: SectionTemplate, paragraphs: &[String]) -> PromptSection {
    let paragraph_items = paragraphs
        .iter()
        .enumerate()
        .filter(|(_, paragraph)| paragraph_matches_keywords(paragraph, template.keywords))
        .take(4)
        .map(|(index, text)| PromptParagraph {
            paragraph_id: format!("{}_p{:03}", template.section_id, index + 1),
            source_order: index + 1,
            text: text.clone(),
        })
        .collect();

    PromptSection {
        section_id: template.section_id.to_string(),
        section_name: template.section_name.to_string(),
        better_home: template.better_home.to_string(),
        stability: template.stability.to_string(),
        judge_section: template.judge_section,
        belongs_in_final_prompt: template.belongs_in_final_prompt,
        paragraphs: paragraph_items,
    }
}

fn paragraph_matches_keywords(paragraph: &str, keywords: &[&str]) -> bool {
    let lower = paragraph.to_ascii_lowercase();
    keywords
        .iter()
        .any(|keyword| lower.contains(&keyword.to_ascii_lowercase()))
}

fn section_templates() -> Vec<SectionTemplate> {
    vec![
        SectionTemplate {
            section_id: "agent_identity_and_scope",
            section_name: "Agent Identity And Scope",
            better_home: "prompt",
            stability: "invariant",
            judge_section: JudgeSection::Structure,
            belongs_in_final_prompt: true,
            keywords: &["virtual assistant", "front desk", "scope"],
        },
        SectionTemplate {
            section_id: "voice_and_patient_experience",
            section_name: "Voice And Patient Experience",
            better_home: "prompt_and_eval",
            stability: "invariant",
            judge_section: JudgeSection::Verification,
            belongs_in_final_prompt: true,
            keywords: &[
                "thank you for calling",
                "frustrated",
                "apologize",
                "reading",
            ],
        },
        SectionTemplate {
            section_id: "runtime_context",
            section_name: "Runtime Context",
            better_home: "runtime_context",
            stability: "time_variant",
            judge_section: JudgeSection::Structure,
            belongs_in_final_prompt: false,
            keywords: &["current time", "caller", "user_number"],
        },
        SectionTemplate {
            section_id: "clinic_config_facts",
            section_name: "Clinic Config Facts",
            better_home: "config",
            stability: "provider_variant",
            judge_section: JudgeSection::Structure,
            belongs_in_final_prompt: false,
            keywords: &["locations", "office", "phone", "parking"],
        },
        SectionTemplate {
            section_id: "provider_directory_and_schedules",
            section_name: "Provider Directory And Schedules",
            better_home: "config_or_tool",
            stability: "provider_variant_or_time_variant",
            judge_section: JudgeSection::Structure,
            belongs_in_final_prompt: false,
            keywords: &["providers", "maternity leave", "sees patients", "available"],
        },
        SectionTemplate {
            section_id: "insurance_and_payment_policy",
            section_name: "Insurance And Payment Policy",
            better_home: "config_or_policy",
            stability: "provider_variant",
            judge_section: JudgeSection::Structure,
            belongs_in_final_prompt: false,
            keywords: &["insurance", "self-pay", "billing", "copays"],
        },
        SectionTemplate {
            section_id: "appointment_catalog_and_visit_rules",
            section_name: "Appointment Catalog And Visit Rules",
            better_home: "config_or_policy",
            stability: "provider_variant",
            judge_section: JudgeSection::Structure,
            belongs_in_final_prompt: false,
            keywords: &["appointment types", "visit", "telehealth", "urgent"],
        },
        SectionTemplate {
            section_id: "workflow_sequences",
            section_name: "Workflow Sequences",
            better_home: "prompt_or_policy",
            stability: "invariant_with_provider_variants",
            judge_section: JudgeSection::WorkflowOrder,
            belongs_in_final_prompt: true,
            keywords: &[
                "for scheduling",
                "for canceling",
                "for rescheduling",
                "confirming",
            ],
        },
        SectionTemplate {
            section_id: "tool_contracts_and_state_changes",
            section_name: "Tool Contracts And State Changes",
            better_home: "tool_schema_and_eval",
            stability: "capability_variant",
            judge_section: JudgeSection::ToolGaps,
            belongs_in_final_prompt: false,
            keywords: &[
                "tools available",
                "use get_available_slots",
                "use cancel_appointment",
            ],
        },
        SectionTemplate {
            section_id: "phi_identity_and_privacy",
            section_name: "PHI Identity And Privacy",
            better_home: "prompt_policy_and_eval",
            stability: "invariant_with_runtime_data",
            judge_section: JudgeSection::SafetyPhi,
            belongs_in_final_prompt: true,
            keywords: &["verify", "identity", "patient information", "date of birth"],
        },
        SectionTemplate {
            section_id: "clinical_safety_and_escalation",
            section_name: "Clinical Safety And Escalation",
            better_home: "prompt_policy_and_eval",
            stability: "invariant",
            judge_section: JudgeSection::SafetyPhi,
            belongs_in_final_prompt: true,
            keywords: &["medical", "emergency", "911", "clinical"],
        },
        SectionTemplate {
            section_id: "transfer_and_routing_rules",
            section_name: "Transfer And Routing Rules",
            better_home: "routing_config_and_tool",
            stability: "provider_variant",
            judge_section: JudgeSection::ToolGaps,
            belongs_in_final_prompt: false,
            keywords: &["transfer", "billing", "front_desk", "nurse_line"],
        },
        SectionTemplate {
            section_id: "language_accessibility_and_interpreter",
            section_name: "Language Accessibility And Interpreter",
            better_home: "prompt_config_or_tool",
            stability: "mixed",
            judge_section: JudgeSection::ToolGaps,
            belongs_in_final_prompt: true,
            keywords: &["Spanish", "interpreter", "language"],
        },
        SectionTemplate {
            section_id: "repair_and_fallback",
            section_name: "Repair And Fallback",
            better_home: "prompt_and_eval",
            stability: "invariant",
            judge_section: JudgeSection::Verification,
            belongs_in_final_prompt: true,
            keywords: &["no match", "three tries", "mistake", "confusion"],
        },
        SectionTemplate {
            section_id: "verification_and_eval_targets",
            section_name: "Verification And Eval Targets",
            better_home: "eval",
            stability: "invariant",
            judge_section: JudgeSection::Verification,
            belongs_in_final_prompt: false,
            keywords: &["scenario", "test", "verify", "before"],
        },
        SectionTemplate {
            section_id: "known_risks_and_human_review",
            section_name: "Known Risks And Human Review",
            better_home: "human_review_or_backlog",
            stability: "mixed",
            judge_section: JudgeSection::Verification,
            belongs_in_final_prompt: false,
            keywords: &["waitlist", "cannot check", "make a note", "arrange"],
        },
    ]
}
