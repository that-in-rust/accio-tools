use serde::{Deserialize, Serialize};

use crate::{Finding, FindingGroup, FixMode, JudgeSection};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecommendedAction {
    pub better_home: String,
    pub recommended_action: String,
}

pub fn collect_recommended_patch_ids(groups: &[FindingGroup]) -> Vec<String> {
    groups
        .iter()
        .filter(|group| group.section != JudgeSection::Verification)
        .flat_map(|group| group.findings.iter())
        .filter(|finding| finding.fix_mode != FixMode::Backlog)
        .map(|finding| finding.finding_id.clone())
        .collect()
}

pub fn create_recommended_action_copy(
    section: &JudgeSection,
    finding: &Finding,
) -> RecommendedAction {
    let text = format!(
        "{} {} {}",
        finding.finding_id, finding.title, finding.verification_scenario
    )
    .to_ascii_lowercase();

    if matches!(section, JudgeSection::SafetyPhi) {
        return RecommendedAction {
            better_home: "policy".to_string(),
            recommended_action:
                "Patch the safety/PHI policy language directly: verify identity before disclosing PHI, avoid medical advice, and escalate emergency symptoms instead of continuing normal scheduling."
                    .to_string(),
        };
    }

    if text.contains("waitlist") {
        return RecommendedAction {
            better_home: "tool/API".to_string(),
            recommended_action:
                "Do not promise a waitlist state change unless the system has a real tool. Add placeholder capability `create_waitlist_entry(patient_id, visit_type, preferred_location, preferred_date_range)` or change the prompt to honest handoff language."
                    .to_string(),
        };
    }

    if text.contains("interpreter") || text.contains("language") {
        return RecommendedAction {
            better_home: "tool/API".to_string(),
            recommended_action:
                "Add placeholder capability `request_interpreter(patient_id, language, appointment_id)` or instruct the agent to route the caller to staff without claiming interpreter support was arranged."
                    .to_string(),
        };
    }

    if text.contains("callback")
        || text.contains("followup")
        || text.contains("follow-up")
        || text.contains("note-taking")
        || text.contains("note taking")
        || text.contains("internal-note")
    {
        return RecommendedAction {
            better_home: "tool/API".to_string(),
            recommended_action:
                "Add placeholder capability `create_staff_followup_task(patient_id, reason, priority)` or change the prompt to say staff will be notified only through an actual transfer/handoff."
                    .to_string(),
        };
    }

    if text.contains("reschedule") || text.contains("cancel") {
        return RecommendedAction {
            better_home: "prompt".to_string(),
            recommended_action:
                "Rewrite the workflow order: call `get_available_slots`, read back the replacement slot, get caller confirmation, call `book_appointment`, and only then call `cancel_appointment` for the old appointment."
                    .to_string(),
        };
    }

    if text.contains("schedule")
        || text.contains("provider")
        || text.contains("availability")
        || text.contains("office hours")
        || text.contains("insurance")
        || text.contains("appointment duration")
    {
        return RecommendedAction {
            better_home: "tool/API".to_string(),
            recommended_action:
                "Move time-sensitive provider facts out of static prompt prose. Add a placeholder provider/config lookup such as `get_provider_availability(provider_id, location_id, visit_type, date_range)` or `get_clinic_config(config_key)`. This belongs in a tool/API, not static prompt prose."
                    .to_string(),
        };
    }

    RecommendedAction {
        better_home: match section {
            JudgeSection::ToolGaps => "tool/API",
            JudgeSection::WorkflowOrder => "prompt",
            JudgeSection::SafetyPhi => "policy",
            JudgeSection::Verification => "backlog",
            JudgeSection::Structure => "config",
        }
        .to_string(),
        recommended_action: finding.suggested_fix.clone(),
    }
}
