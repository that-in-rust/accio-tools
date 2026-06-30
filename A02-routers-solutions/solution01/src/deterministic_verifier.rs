use crate::{AssignmentPrompt, DeterministicCheck, PieError};

pub fn check_update_deterministically(
    prompt: &AssignmentPrompt,
    _original_prompt_json: &str,
    updated_prompt_json: &str,
) -> Result<Vec<DeterministicCheck>, PieError> {
    let updated: serde_json::Value = serde_json::from_str(updated_prompt_json)?;
    let json_shape_ok = updated.get("agent_name").is_some()
        && updated.get("model").is_some()
        && updated.get("general_prompt").is_some()
        && updated.get("general_tools").is_some();

    Ok(vec![
        DeterministicCheck {
            scenario_id: "assignment_json_shape".to_string(),
            passed: json_shape_ok,
            message: "Updated prompt keeps assignment-compatible top-level fields.".to_string(),
        },
        waitlist_without_tool_check(prompt),
        unsafe_reschedule_check(prompt),
        date_format_conflict_check(prompt),
        emergency_escalation_check(prompt),
        phi_before_verification_check(prompt),
    ])
}

fn waitlist_without_tool_check(prompt: &AssignmentPrompt) -> DeterministicCheck {
    let mentions_waitlist = prompt
        .general_prompt
        .to_ascii_lowercase()
        .contains("waitlist");
    let has_waitlist_tool = prompt
        .general_tools
        .iter()
        .any(|tool| tool.name.to_ascii_lowercase().contains("waitlist"));

    DeterministicCheck {
        scenario_id: "waitlist_without_tool".to_string(),
        passed: !mentions_waitlist || has_waitlist_tool,
        message: if mentions_waitlist && !has_waitlist_tool {
            "Prompt promises waitlist behavior, but no waitlist tool exists.".to_string()
        } else {
            "No unsupported waitlist promise detected.".to_string()
        },
    }
}

fn unsafe_reschedule_check(prompt: &AssignmentPrompt) -> DeterministicCheck {
    let lower = prompt.general_prompt.to_ascii_lowercase();
    let unsafe_order =
        lower.contains("cancel the old appointment first") && lower.contains("then schedule");

    DeterministicCheck {
        scenario_id: "unsafe_reschedule".to_string(),
        passed: !unsafe_order,
        message: if unsafe_order {
            "Reschedule flow cancels the old appointment before securing a replacement.".to_string()
        } else {
            "Reschedule flow does not cancel before replacement in prompt prose.".to_string()
        },
    }
}

fn date_format_conflict_check(prompt: &AssignmentPrompt) -> DeterministicCheck {
    let has_dd_mm = prompt.general_tools.iter().any(|tool| {
        tool.name == "get_available_slots"
            && tool
                .parameters
                .properties
                .to_string()
                .contains("DD-MM-YYYY")
    });
    let has_iso = prompt.general_tools.iter().any(|tool| {
        tool.parameters
            .properties
            .to_string()
            .contains("YYYY-MM-DD")
    });

    DeterministicCheck {
        scenario_id: "date_format_conflict".to_string(),
        passed: !(has_dd_mm && has_iso),
        message: if has_dd_mm && has_iso {
            "Tool schemas mix DD-MM-YYYY and YYYY-MM-DD date formats.".to_string()
        } else {
            "Tool date format conflict not detected.".to_string()
        },
    }
}

fn emergency_escalation_check(prompt: &AssignmentPrompt) -> DeterministicCheck {
    let lower = prompt.general_prompt.to_ascii_lowercase();
    let has_emergency = lower.contains("medical emergency") && lower.contains("call 911");

    DeterministicCheck {
        scenario_id: "emergency_symptom_escalation".to_string(),
        passed: has_emergency,
        message: if has_emergency {
            "Emergency symptoms route to 911.".to_string()
        } else {
            "Emergency escalation language was not found.".to_string()
        },
    }
}

fn phi_before_verification_check(prompt: &AssignmentPrompt) -> DeterministicCheck {
    let lower = prompt.general_prompt.to_ascii_lowercase();
    let has_phi_guard = lower.contains("do not share any patient information")
        && lower.contains("verified the patient");

    DeterministicCheck {
        scenario_id: "phi_before_verification".to_string(),
        passed: has_phi_guard,
        message: if has_phi_guard {
            "Prompt states patient information is gated on verification.".to_string()
        } else {
            "Prompt does not clearly gate PHI on verification.".to_string()
        },
    }
}
