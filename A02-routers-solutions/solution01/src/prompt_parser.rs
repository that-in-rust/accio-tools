use crate::{AssignmentPrompt, PieError};

pub fn parse_assignment_prompt_json(input: &str) -> Result<AssignmentPrompt, PieError> {
    let prompt: AssignmentPrompt =
        serde_json::from_str(input).map_err(|error| PieError::InvalidPromptJson {
            message: error.to_string(),
        })?;

    if prompt.agent_name.trim().is_empty() {
        return Err(PieError::MissingRequiredField {
            field: "agent_name",
        });
    }
    if prompt.model.trim().is_empty() {
        return Err(PieError::MissingRequiredField { field: "model" });
    }
    if prompt.general_prompt.trim().is_empty() {
        return Err(PieError::MissingRequiredField {
            field: "general_prompt",
        });
    }
    if prompt.general_tools.is_empty() {
        return Err(PieError::MissingRequiredField {
            field: "general_tools",
        });
    }

    Ok(prompt)
}
