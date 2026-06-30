use crate::{
    create_recommended_action_copy, DeterministicCheck, FindingGroup, JudgeSection, PieError,
    SemanticCheck, VerificationExport,
};

pub fn export_findings_markdown_text(
    groups: &[FindingGroup],
    model_label: String,
) -> Result<String, PieError> {
    let mut output = String::new();
    output.push_str("# PIE Findings Export\n\n");
    output.push_str(&model_label);
    output.push_str("\n\n");

    output.push_str("| Findings | Recommended Actions |\n");
    output.push_str("| --- | --- |\n");

    for group in groups
        .iter()
        .filter(|group| group.section != JudgeSection::Verification)
    {
        for finding in &group.findings {
            let action = create_recommended_action_copy(&group.section, finding);
            output.push_str("| ");
            output.push_str(&escape_markdown_table_cell(&format!(
                "**{}**<br>ID: {}<br>Section: {}<br>Severity: {}<br>Impact: {}",
                finding.title,
                finding.finding_id,
                group.section.as_label(),
                finding.severity.as_label(),
                finding.impact
            )));
            output.push_str(" | ");
            output.push_str(&escape_markdown_table_cell(&format!(
                "**{}**<br>{}",
                action.better_home, action.recommended_action
            )));
            output.push_str(" |\n");
        }
    }

    Ok(output)
}

fn escape_markdown_table_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', "<br>")
}

pub fn export_verification_report(
    updated_prompt_json: String,
    model_label: String,
    selected_finding_ids: &[String],
    deterministic_checks: Vec<DeterministicCheck>,
    semantic_checks: Vec<SemanticCheck>,
) -> VerificationExport {
    let mut markdown_report = String::new();
    markdown_report.push_str("# Before/After Verification\n\n");
    markdown_report.push_str(&model_label);
    markdown_report.push_str("\n\n");
    markdown_report.push_str("## Selected Fixes\n\n");
    for finding_id in selected_finding_ids {
        markdown_report.push_str("- ");
        markdown_report.push_str(finding_id);
        markdown_report.push('\n');
    }

    markdown_report.push_str("\n## Deterministic Checks\n\n");
    for check in &deterministic_checks {
        markdown_report.push_str("- ");
        markdown_report.push_str(&check.scenario_id);
        markdown_report.push_str(": ");
        markdown_report.push_str(if check.passed { "pass" } else { "fail" });
        markdown_report.push_str(" - ");
        markdown_report.push_str(&check.message);
        markdown_report.push('\n');
    }

    markdown_report.push_str("\n## LLM Semantic Checks\n\n");
    for check in &semantic_checks {
        markdown_report.push_str("- ");
        markdown_report.push_str(&check.scenario_id);
        markdown_report.push_str(": ");
        markdown_report.push_str(if check.passed { "pass" } else { "fail" });
        markdown_report.push_str(" - ");
        markdown_report.push_str(&check.message);
        markdown_report.push('\n');
    }

    VerificationExport {
        markdown_report,
        updated_prompt_json,
        deterministic_checks,
        semantic_checks,
    }
}
