use pie_tauri_core::{
    analyze_prompt, apply_selected_fixes, build_judge_request, build_patch_request,
    check_update_deterministically, collect_recommended_patch_ids,
    create_prompt_candidate_from_patch, create_prompt_candidate_from_rewrite,
    create_prompt_candidate_json, create_recommended_action_copy, export_findings_markdown_text,
    fix_patch_instructions, get_app_readiness, ideal_voice_prompt_structure_json,
    judge_prompt_instructions, map_openai_status_error, model_label_text,
    normalize_prompt_sections_model, parse_assignment_prompt_json, parse_judge_response_text,
    reverify_prompt, reverify_prompt_instructions, verify_and_export_update, AnalyzePromptRequest,
    AppReadinessRequest, DiagnosticLogger, FakeOpenAiClient, Finding, FindingGroup,
    FindingSeverity, FixMode, JudgeSection, OpenAiClient, OpenAiHttpClient, OpenAiRuntimeConfig,
    PatchFailureMode, PromptExportCandidate, PromptPatchEnvelope, PromptPatchMode,
    ReverifyPromptRequest, ReverifyStatus, SelectedFixesRequest, SqliteVersionStore, VersionKind,
    OPENAI_MODEL_ID,
};

const ASSIGNMENT_PROMPT: &str = include_str!("../../A00-raw-ref/assignment-agent-prompt.json");

#[test]
fn test_req_tauri_005_parser_accepts_assignment_fixture() {
    let prompt = parse_assignment_prompt_json(ASSIGNMENT_PROMPT).expect("fixture parses");

    assert_eq!(
        prompt.agent_name,
        "Greenfield Medical Group - Front Desk Agent"
    );
    assert_eq!(prompt.model, "gpt-4.1");
    assert_eq!(prompt.general_tools.len(), 9);
    assert_eq!(prompt.general_prompt.chars().count(), 26_889);
}

#[test]
fn test_req_tauri_005_parser_rejects_missing_prompt() {
    let error = parse_assignment_prompt_json(r#"{"agent_name":"x","model":"gpt-4.1"}"#)
        .expect_err("missing prompt must fail");

    assert_eq!(error.code(), "invalid_prompt_json");
}

#[test]
fn test_req_tauri_021_model_config_uses_fixed_model() {
    let config = OpenAiRuntimeConfig::from_api_key("user-provided-key".to_string());
    let prompt = parse_assignment_prompt_json(ASSIGNMENT_PROMPT).expect("fixture parses");
    let normalized = normalize_prompt_sections_model(&prompt).expect("normalizes");
    let request = build_judge_request(&config, &prompt, &normalized).expect("request builds");

    assert_eq!(OPENAI_MODEL_ID, "gpt-5.4-mini");
    assert_eq!(config.model, OPENAI_MODEL_ID);
    assert_eq!(request.model, OPENAI_MODEL_ID);
    assert_eq!(model_label_text(), "Model: gpt-5.4-mini");
}

#[test]
fn test_req_tauri_007_normalizer_emits_prd_sections() {
    let prompt = parse_assignment_prompt_json(ASSIGNMENT_PROMPT).expect("fixture parses");
    let normalized = normalize_prompt_sections_model(&prompt).expect("normalizes");

    assert_eq!(normalized.sections.len(), 16);
    assert!(normalized
        .sections
        .iter()
        .any(|section| section.section_id == "tool_contracts_and_state_changes"));
    assert!(normalized
        .sections
        .iter()
        .any(|section| section.section_id == "known_risks_and_human_review"));
}

#[test]
fn test_req_tauri_006_sqlite_store_names_timestamped_versions() {
    let store = SqliteVersionStore::open_in_memory().expect("store opens");
    let version = store
        .store_prompt_version(
            "assignment-agent-prompt.json",
            VersionKind::Original,
            ASSIGNMENT_PROMPT,
            "analysis-run-001",
            model_label_text(),
        )
        .expect("version stored");

    assert!(version
        .version_name
        .starts_with("assignment-agent-prompt_v"));
    assert!(store
        .load_version_payload(&version.version_name)
        .expect("payload loads")
        .contains("Greenfield Medical Group"));
}

#[test]
fn test_req_tauri_006_file_backed_store_persists_versions() {
    let directory = tempfile::tempdir().expect("tempdir opens");
    let database_path = directory.path().join("pie-versions.sqlite");
    let version_name = {
        let store = SqliteVersionStore::open_at_path(&database_path).expect("store opens");
        let version = store
            .store_prompt_version(
                "assignment-agent-prompt.json",
                VersionKind::Original,
                ASSIGNMENT_PROMPT,
                "analysis-run-file-backed",
                model_label_text(),
            )
            .expect("version stored");
        version.version_name
    };

    let reopened = SqliteVersionStore::open_at_path(&database_path).expect("store reopens");
    let payload = reopened
        .load_version_payload(&version_name)
        .expect("payload survives reopen");

    assert!(payload.contains("Greenfield Medical Group"));
}

#[test]
fn test_req_tauri_008_judge_request_has_five_sections() {
    let config = OpenAiRuntimeConfig::from_api_key("user-provided-key".to_string());
    let prompt = parse_assignment_prompt_json(ASSIGNMENT_PROMPT).expect("fixture parses");
    let normalized = normalize_prompt_sections_model(&prompt).expect("normalizes");
    let request = build_judge_request(&config, &prompt, &normalized).expect("request builds");

    assert_eq!(
        request.required_sections,
        vec![
            JudgeSection::Structure,
            JudgeSection::ToolGaps,
            JudgeSection::WorkflowOrder,
            JudgeSection::SafetyPhi,
            JudgeSection::Verification,
        ]
    );
}

#[test]
fn test_req_tauri_008_llm_judge_accepts_human_label_variants() {
    let parsed: FindingGroup = serde_json::from_str(
        r#"{
            "section": "Tool Gaps",
            "findings": [
                {
                    "finding_id": "finding-tool-waitlist",
                    "title": "Waitlist promise has no durable tool",
                    "severity": "medium",
                    "prompt_evidence": "offer to put them on a waitlist",
                    "impact": "Caller may believe a state change happened.",
                    "suggested_fix": "Use honest handoff language.",
                    "fix_mode": "prompt_edit",
                    "verification_scenario": "waitlist_without_tool"
                }
            ]
        }"#,
    )
    .expect("LLM judge labels are parsed tolerantly");

    assert_eq!(parsed.section, JudgeSection::ToolGaps);
    assert_eq!(parsed.findings[0].severity, FindingSeverity::Medium);
    assert_eq!(parsed.findings[0].fix_mode, FixMode::AutoFixable);
}

#[test]
fn test_req_tauri_008_llm_judge_normalizes_consolidate_fix_mode() {
    let parsed: FindingGroup = serde_json::from_str(
        r#"{
            "section": "Tool Gaps",
            "findings": [
                {
                    "finding_id": "finding-tool-consolidate-copy",
                    "title": "Consolidate repeated clinic facts",
                    "severity": "medium",
                    "prompt_evidence": "doctor schedule and office policy are repeated",
                    "impact": "Prompt becomes brittle and harder to maintain.",
                    "suggested_fix": "Consolidate duplicated facts into one instruction block.",
                    "fix_mode": "consolidate",
                    "verification_scenario": "dynamic_provider_facts"
                }
            ]
        }"#,
    )
    .expect("judge fix mode aliases stay tolerant");

    assert_eq!(parsed.findings[0].fix_mode, FixMode::AutoFixable);
}

#[test]
fn test_req_tauri_008_judge_response_tolerates_duplicate_section_keys() {
    let parsed = parse_judge_response_text(
        r#"{
            "finding_groups": [
                {
                    "section": "Structure",
                    "section": "Tool Gaps",
                    "findings": [
                        {
                            "finding_id": "finding-tool-waitlist",
                            "title": "Waitlist promise has no durable tool",
                            "severity": "high",
                            "prompt_evidence": "offer waitlist",
                            "impact": "Caller may believe a state change happened.",
                            "suggested_fix": "Use honest handoff language.",
                            "fix_mode": "consolidate",
                            "verification_scenario": "waitlist_without_tool"
                        }
                    ]
                }
            ]
        }"#,
    )
    .expect("duplicate section keys should not break analysis");

    let tool_group = parsed
        .iter()
        .find(|group| group.section == JudgeSection::ToolGaps)
        .expect("tool group exists");

    assert_eq!(tool_group.findings[0].fix_mode, FixMode::AutoFixable);
}

#[test]
fn test_req_tauri_008_judge_prompt_is_standard_prd_based_asset() {
    let instructions = judge_prompt_instructions();

    assert!(instructions.contains("PIE LLM Judge Prompt"));
    assert!(instructions.contains("Variant Data Placement"));
    assert!(instructions.contains("Capability Variants"));
    assert!(instructions.contains("Invariant Workflow Order"));
    assert!(instructions.contains("Invariant Safety / PHI"));
    assert!(instructions.contains("Eval Invariants"));
    assert!(instructions.contains("S01PRD.md"));
}

#[test]
fn test_req_tauri_050_reverify_prompt_is_dedicated_history_asset() {
    let instructions = reverify_prompt_instructions();

    assert!(instructions.contains("PIE Reverify Prompt"));
    assert!(instructions.contains("previous_finding_groups"));
    assert!(instructions.contains("updated_prompt_json"));
    assert!(instructions.contains("variant"));
    assert!(instructions.contains("invariant"));
    assert!(!std::ptr::eq(
        instructions.as_ptr(),
        judge_prompt_instructions().as_ptr()
    ));
}

#[test]
fn test_req_tauri_040_recommended_action_maps_schedule_to_api_placeholder() {
    let finding = Finding {
        finding_id: "finding-structure-dynamic-provider-facts".to_string(),
        title: "Hardcoded provider schedules".to_string(),
        severity: FindingSeverity::High,
        prompt_evidence: "Dr. Patel is available Monday 9-5".to_string(),
        impact: "Schedules drift.".to_string(),
        suggested_fix: "Move schedules out of prompt.".to_string(),
        fix_mode: FixMode::AutoFixable,
        verification_scenario: "dynamic_provider_facts_embedded".to_string(),
    };

    let action = create_recommended_action_copy(&JudgeSection::Structure, &finding);

    assert_eq!(action.better_home, "tool/API");
    assert!(action
        .recommended_action
        .contains("get_provider_availability"));
    assert!(action
        .recommended_action
        .contains("not static prompt prose"));
}

#[test]
fn test_req_tauri_041_recommended_action_maps_waitlist_to_placeholder() {
    let finding = Finding {
        finding_id: "finding-tool-waitlist".to_string(),
        title: "Waitlist promise has no durable tool".to_string(),
        severity: FindingSeverity::High,
        prompt_evidence: "put them on a waitlist".to_string(),
        impact: "Caller may believe a state change happened.".to_string(),
        suggested_fix: "Move time-sensitive provider facts out of static prompt prose.".to_string(),
        fix_mode: FixMode::AutoFixable,
        verification_scenario: "waitlist_without_tool".to_string(),
    };

    let action = create_recommended_action_copy(&JudgeSection::ToolGaps, &finding);

    assert_eq!(action.better_home, "tool/API");
    assert!(action.recommended_action.contains("create_waitlist_entry"));
    assert!(action.recommended_action.contains("honest handoff"));
}

#[test]
fn test_req_tauri_042_recommended_action_maps_reschedule_order() {
    let finding = Finding {
        finding_id: "finding-workflow-reschedule-cancels-first".to_string(),
        title: "Reschedule cancels old appointment first".to_string(),
        severity: FindingSeverity::High,
        prompt_evidence: "Cancel the old appointment first.".to_string(),
        impact: "Patient can lose current appointment.".to_string(),
        suggested_fix: "Find replacement before cancellation.".to_string(),
        fix_mode: FixMode::AutoFixable,
        verification_scenario: "unsafe_reschedule".to_string(),
    };

    let action = create_recommended_action_copy(&JudgeSection::WorkflowOrder, &finding);

    assert_eq!(action.better_home, "prompt");
    assert!(action.recommended_action.contains("get_available_slots"));
    assert!(action.recommended_action.contains("book_appointment"));
    assert!(action.recommended_action.contains("cancel_appointment"));
}

#[test]
fn test_req_tauri_043_recommended_patch_ids_skip_verification_and_backlog() {
    let groups = vec![
        finding_group_with_mode(
            JudgeSection::Structure,
            "finding-structure-dynamic-provider-facts",
            FixMode::AutoFixable,
        ),
        finding_group_with_mode(
            JudgeSection::ToolGaps,
            "finding-tool-waitlist",
            FixMode::HumanReviewOnly,
        ),
        finding_group_with_mode(
            JudgeSection::Verification,
            "finding-verification-scenarios-missing",
            FixMode::AutoFixable,
        ),
        finding_group_with_mode(
            JudgeSection::WorkflowOrder,
            "finding-workflow-backlog",
            FixMode::Backlog,
        ),
    ];

    let recommended_ids = collect_recommended_patch_ids(&groups);

    assert_eq!(
        recommended_ids,
        vec![
            "finding-structure-dynamic-provider-facts".to_string(),
            "finding-tool-waitlist".to_string()
        ]
    );
}

#[test]
fn test_req_tauri_046_patch_prompt_is_dedicated_asset() {
    let instructions = fix_patch_instructions();

    assert!(instructions.contains("PIE Fix Patch Prompt"));
    assert!(instructions.contains("Apply Recommended Patch"));
    assert!(instructions.contains("placeholder"));
    assert!(instructions.contains("Return plain text only"));
    assert!(!instructions.contains("prompt_replacements"));
    assert!(!instructions.contains("prompt_appendix"));
    assert!(instructions.contains("Do not return a JSON patch envelope"));
    assert!(!std::ptr::eq(
        instructions.as_ptr(),
        judge_prompt_instructions().as_ptr()
    ));
}

#[test]
fn test_req_tauri_048_patch_candidate_includes_placeholder_actions() {
    let config = OpenAiRuntimeConfig::from_api_key("user-provided-key".to_string());
    let groups = vec![
        finding_group_with_mode(
            JudgeSection::Structure,
            "finding-structure-dynamic-provider-facts",
            FixMode::AutoFixable,
        ),
        finding_group_with_mode(
            JudgeSection::ToolGaps,
            "finding-tool-waitlist",
            FixMode::AutoFixable,
        ),
    ];
    let recommended_ids = collect_recommended_patch_ids(&groups);
    let request =
        build_patch_request(&config, ASSIGNMENT_PROMPT, &groups, &recommended_ids).unwrap();

    assert!(request
        .patch_instructions
        .iter()
        .any(|instruction| instruction.contains("get_provider_availability")));
    assert!(request
        .patch_instructions
        .iter()
        .any(|instruction| instruction.contains("create_waitlist_entry")));
}

#[test]
fn test_req_patch_001_patch_request_selects_targeted_mode() {
    let config = OpenAiRuntimeConfig::from_api_key("user-provided-key".to_string());
    let request = build_patch_request(
        &config,
        ASSIGNMENT_PROMPT,
        &sample_finding_groups(),
        &["finding-tool-waitlist".to_string()],
    )
    .expect("patch request builds");

    assert_eq!(request.patch_mode, PromptPatchMode::TargetedPatch);
    assert!(request
        .ideal_prompt_structure_json
        .contains("ideal_voice_prompt_structure"));
}

#[test]
fn test_req_patch_002_patch_request_selects_section_rewrite_mode() {
    let config = OpenAiRuntimeConfig::from_api_key("user-provided-key".to_string());
    let original_prompt_json = serde_json::json!({
        "agent_name": "Greenfield Medical Group - Front Desk Agent",
        "model": "gpt-4.1",
        "general_prompt": "The first unsafe instruction appears here.",
        "general_tools": []
    })
    .to_string();
    let groups = vec![FindingGroup {
        section: JudgeSection::WorkflowOrder,
        findings: vec![
            Finding {
                finding_id: "finding-first".to_string(),
                title: "First issue".to_string(),
                severity: FindingSeverity::High,
                prompt_evidence: "The first unsafe instruction appears here.".to_string(),
                impact: "Unsafe workflow.".to_string(),
                suggested_fix: "Rewrite the workflow safely.".to_string(),
                fix_mode: FixMode::AutoFixable,
                verification_scenario: "partial_anchor".to_string(),
            },
            Finding {
                finding_id: "finding-second".to_string(),
                title: "Second issue".to_string(),
                severity: FindingSeverity::High,
                prompt_evidence: "This evidence is not present.".to_string(),
                impact: "Unsafe workflow.".to_string(),
                suggested_fix: "Rewrite the missing instruction safely.".to_string(),
                fix_mode: FixMode::AutoFixable,
                verification_scenario: "partial_anchor".to_string(),
            },
        ],
    }];

    let request = build_patch_request(
        &config,
        &original_prompt_json,
        &groups,
        &["finding-first".to_string(), "finding-second".to_string()],
    )
    .expect("patch request builds");

    assert_eq!(request.patch_mode, PromptPatchMode::SectionRewrite);
    assert!(request.patch_mode_reason.contains("partial"));
}

#[test]
fn test_req_patch_003_patch_request_selects_clean_scaffold_mode() {
    let config = OpenAiRuntimeConfig::from_api_key("user-provided-key".to_string());
    let original_prompt_json = serde_json::json!({
        "agent_name": "Greenfield Medical Group - Front Desk Agent",
        "model": "gpt-4.1",
        "general_prompt": "Broken one-line prompt with no selected evidence.",
        "general_tools": []
    })
    .to_string();

    let request = build_patch_request(
        &config,
        &original_prompt_json,
        &sample_finding_groups(),
        &["finding-tool-waitlist".to_string()],
    )
    .expect("patch request builds");

    assert_eq!(request.patch_mode, PromptPatchMode::CleanScaffold);
    assert!(request.patch_mode_reason.contains("No selected evidence"));
    assert!(request
        .ideal_prompt_structure_json
        .contains("minimum_rebuild_sections_in_final_prompt"));
}

#[test]
fn test_req_tauri_049_patch_candidate_replaces_selected_evidence() {
    let config = OpenAiRuntimeConfig::from_api_key("user-provided-key".to_string());
    let original_prompt_json = serde_json::json!({
        "agent_name": "Greenfield Medical Group - Front Desk Agent",
        "model": "gpt-4.1",
        "general_prompt": "You may promise to put callers on a waitlist even though no waitlist tool exists.\nCancel the old appointment first before looking for a replacement.",
        "general_tools": []
    })
    .to_string();
    let groups = vec![FindingGroup {
        section: JudgeSection::ToolGaps,
        findings: vec![Finding {
            finding_id: "finding-tool-waitlist".to_string(),
            title: "Waitlist promise has no durable tool".to_string(),
            severity: FindingSeverity::High,
            prompt_evidence:
                "You may promise to put callers on a waitlist even though no waitlist tool exists."
                    .to_string(),
            impact: "Caller may believe a state change happened.".to_string(),
            suggested_fix: "Use honest handoff language unless a waitlist tool exists.".to_string(),
            fix_mode: FixMode::AutoFixable,
            verification_scenario: "waitlist_without_tool".to_string(),
        }],
    }];
    let request = build_patch_request(
        &config,
        &original_prompt_json,
        &groups,
        &["finding-tool-waitlist".to_string()],
    )
    .expect("patch request builds");

    let updated_prompt_json =
        create_prompt_candidate_json(&request).expect("candidate prompt is generated");
    let updated: serde_json::Value =
        serde_json::from_str(&updated_prompt_json).expect("updated candidate is JSON");
    let updated_prompt = updated["general_prompt"]
        .as_str()
        .expect("updated prompt exists");

    assert!(!updated_prompt.contains(
        "You may promise to put callers on a waitlist even though no waitlist tool exists."
    ));
    assert!(updated_prompt.contains("create_waitlist_entry"));
    assert!(updated_prompt.contains("honest handoff"));
}

#[test]
fn test_req_patch_004_rebuilt_export_preserves_assignment_shape() {
    let original_prompt_json = serde_json::json!({
        "agent_name": "Greenfield Medical Group - Front Desk Agent",
        "model": "gpt-4.1",
        "general_prompt": "Broken old prompt.",
        "general_tools": [
            {
                "type": "custom",
                "name": "find_patient",
                "description": "Find a patient.",
                "method": "POST",
                "url": "https://example.test/find_patient",
                "headers": {},
                "parameters": {"required": [], "properties": {}}
            }
        ]
    })
    .to_string();
    let patch = PromptPatchEnvelope {
        applied_finding_ids: vec!["finding-tool-waitlist".to_string()],
        patch_summary: "Rebuilt prompt from ideal structure.".to_string(),
        prompt_replacements: Vec::new(),
        prompt_appendix: None,
        export: Some(PromptExportCandidate {
            agent_name: Some("Wrong replacement name".to_string()),
            model: Some("wrong-model".to_string()),
            general_prompt: "## Agent Identity and Scope\nUse a safe rebuilt prompt.".to_string(),
        }),
    };

    let updated_prompt_json =
        create_prompt_candidate_from_patch(&original_prompt_json, &patch).expect("export applies");
    let updated: serde_json::Value =
        serde_json::from_str(&updated_prompt_json).expect("updated JSON parses");

    assert_eq!(
        updated["agent_name"],
        "Greenfield Medical Group - Front Desk Agent"
    );
    assert_eq!(updated["model"], "gpt-4.1");
    assert_eq!(updated["general_tools"][0]["name"], "find_patient");
    assert_eq!(
        updated["general_prompt"],
        "## Agent Identity and Scope\nUse a safe rebuilt prompt."
    );
}

#[test]
fn test_req_patch_005_clean_scaffold_candidate_contains_required_sections() {
    let config = OpenAiRuntimeConfig::from_api_key("user-provided-key".to_string());
    let original_prompt_json = serde_json::json!({
        "agent_name": "Greenfield Medical Group - Front Desk Agent",
        "model": "gpt-4.1",
        "general_prompt": "Broken one-line prompt with no selected evidence.",
        "general_tools": [{"name": "find_patient"}]
    })
    .to_string();
    let request = build_patch_request(
        &config,
        &original_prompt_json,
        &sample_finding_groups(),
        &["finding-tool-waitlist".to_string()],
    )
    .expect("patch request builds");

    let updated_prompt_json =
        create_prompt_candidate_json(&request).expect("clean scaffold candidate is generated");
    let updated: serde_json::Value =
        serde_json::from_str(&updated_prompt_json).expect("updated JSON parses");
    let updated_prompt = updated["general_prompt"]
        .as_str()
        .expect("updated prompt exists");

    assert_eq!(
        updated["agent_name"],
        "Greenfield Medical Group - Front Desk Agent"
    );
    assert_eq!(updated["general_tools"][0]["name"], "find_patient");
    assert!(updated_prompt.contains("PIE Clean Scaffold Prompt"));
    assert!(updated_prompt.contains("PHI, Identity, and Privacy"));
    assert!(updated_prompt.contains("Clinical Safety and Escalation"));
    assert!(updated_prompt.contains("create_waitlist_entry"));
    assert!(!updated_prompt.contains("certification"));
}

#[test]
fn test_req_patch_006_ideal_structure_asset_is_parseable() {
    let parsed: serde_json::Value =
        serde_json::from_str(ideal_voice_prompt_structure_json()).expect("seed parses");

    assert_eq!(parsed["schema_name"], "ideal_voice_prompt_structure");
    assert!(
        parsed["section_contracts"]
            .as_array()
            .expect("section contracts")
            .len()
            >= 16
    );
}

#[test]
fn test_req_tauri_033_diagnostic_logger_persists_redacted_lines() {
    let directory = tempfile::tempdir().expect("tempdir opens");
    let log_path = directory.path().join("logs").join("pie.log");
    let logger = DiagnosticLogger::open_at_path(&log_path).expect("logger opens");

    logger
        .record_info_event(
            "command_started",
            Some("validate_api_key"),
            "validating bearer sk-proj-secret-that-must-not-leak",
        )
        .expect("log line writes");
    drop(logger);

    let reopened = DiagnosticLogger::open_at_path(&log_path).expect("logger reopens");
    let exported = reopened
        .export_log_text_report()
        .expect("diagnostic log exports");

    assert!(exported.contains("validate_api_key"));
    assert!(exported.contains("[redacted-key]"));
    assert!(!exported.contains("sk-proj-secret-that-must-not-leak"));
}

#[test]
fn test_req_tauri_033_diagnostic_logger_export_includes_path_header() {
    let directory = tempfile::tempdir().expect("tempdir opens");
    let log_path = directory.path().join("logs").join("pie.log");
    let logger = DiagnosticLogger::open_at_path(&log_path).expect("logger opens");

    logger
        .record_error_event(
            "command_failed",
            Some("analyze_prompt"),
            "serialization error: prompt_edit alias missing",
            Some("serialization_error"),
        )
        .expect("error line writes");

    let exported = logger
        .export_log_text_report()
        .expect("diagnostic log exports");

    assert!(exported.contains("# PIE Diagnostic Log"));
    assert!(exported.contains(&log_path.display().to_string()));
    assert!(exported.contains("serialization_error"));
}

#[tokio::test]
async fn test_req_tauri_001_readiness_exposes_model_label() {
    let client = FakeOpenAiClient::ready();
    let response = get_app_readiness(
        AppReadinessRequest {
            api_key: Some("user-provided-key".to_string()),
            storage_ready: true,
        },
        &client,
    )
    .await
    .expect("readiness succeeds");

    assert!(response.analyze_enabled);
    assert_eq!(response.model_label, "Model: gpt-5.4-mini");
}

#[tokio::test]
async fn test_req_tauri_024_model_unavailable_blocks_readiness() {
    let client = FakeOpenAiClient::model_unavailable();
    let response = get_app_readiness(
        AppReadinessRequest {
            api_key: Some("user-provided-key".to_string()),
            storage_ready: true,
        },
        &client,
    )
    .await
    .expect("readiness response is serializable");

    assert!(!response.analyze_enabled);
    assert!(!response.api_key_ready);
    assert!(response
        .readiness_message
        .contains("gpt-5.4-mini is unavailable"));
}

#[test]
fn test_req_tauri_024_openai_status_mapping_keeps_api_key_secret() {
    let secret = "sk-proj-secret-that-must-not-leak";
    let invalid_key = map_openai_status_error(
        401,
        r#"{"error":{"message":"Incorrect API key provided: sk-proj-secret-that-must-not-leak"}}"#,
        secret,
    );
    let quota_limited = map_openai_status_error(429, "quota exceeded", secret);
    let model_unavailable =
        map_openai_status_error(404, "model gpt-5.4-mini does not exist", secret);

    let invalid_app_error = pie_tauri_core::AppError::from(invalid_key);
    let quota_app_error = pie_tauri_core::AppError::from(quota_limited);
    let model_app_error = pie_tauri_core::AppError::from(model_unavailable);

    assert_eq!(invalid_app_error.code, "invalid_api_key");
    assert_eq!(quota_app_error.code, "quota_limited");
    assert_eq!(model_app_error.code, "model_unavailable");
    assert!(!invalid_app_error.message.contains(secret));
    assert!(!quota_app_error.message.contains(secret));
    assert!(!model_app_error.message.contains(secret));
}

#[test]
fn test_req_tauri_024_invalid_request_keeps_sanitized_provider_detail() {
    let secret = "sk-proj-secret-that-must-not-leak";
    let error = map_openai_status_error(
        400,
        r#"{"error":{"message":"Invalid 'max_output_tokens': integer below minimum value. Expected a value >= 16, but got 8 instead."}}"#,
        secret,
    );
    let app_error = pie_tauri_core::AppError::from(error);

    assert_eq!(app_error.code, "invalid_request");
    assert!(app_error.message.contains("max_output_tokens"));
    assert!(app_error.message.contains(">= 16"));
    assert!(!app_error.message.contains(secret));
}

#[tokio::test]
async fn test_req_tauri_021_validation_payload_uses_minimum_token_cap() {
    let server = LocalResponseServer::spawn();
    let client = OpenAiHttpClient::with_endpoint(server.endpoint());
    let config = OpenAiRuntimeConfig::from_api_key("sk-test-secret".to_string());

    client
        .validate_key_with_model(&config)
        .await
        .expect("validation response extracts output text");

    let body = server.request_body();
    let payload: serde_json::Value = serde_json::from_str(&body).expect("request body is JSON");

    assert_eq!(payload["model"], OPENAI_MODEL_ID);
    assert_eq!(payload["input"], "Reply with OK.");
    assert_eq!(payload["max_output_tokens"], 16);
    assert_eq!(payload["reasoning"]["effort"], "none");
    assert_eq!(payload["text"]["verbosity"], "low");
    assert!(!body.contains("sk-test-secret"));
}

#[tokio::test]
async fn test_req_tauri_008_analysis_returns_five_groups() {
    let client = FakeOpenAiClient::ready();
    let store = SqliteVersionStore::open_in_memory().expect("store opens");
    let result = analyze_prompt(
        AnalyzePromptRequest {
            filename: "assignment-agent-prompt.json".to_string(),
            prompt_json: ASSIGNMENT_PROMPT.to_string(),
            api_key: "user-provided-key".to_string(),
        },
        &client,
        &store,
    )
    .await
    .expect("analysis succeeds");

    assert_eq!(result.finding_groups.len(), 5);
    assert_eq!(result.model_label, "Model: gpt-5.4-mini");
    assert!(result
        .finding_groups
        .iter()
        .all(|group| !group.findings.is_empty()));
}

#[test]
fn test_req_tauri_012_patch_request_uses_selected_findings_only() {
    let config = OpenAiRuntimeConfig::from_api_key("user-provided-key".to_string());
    let groups = sample_finding_groups();
    let request = build_patch_request(
        &config,
        ASSIGNMENT_PROMPT,
        &groups,
        &["finding-tool-waitlist".to_string()],
    )
    .expect("patch request builds");

    assert_eq!(request.model, OPENAI_MODEL_ID);
    assert_eq!(request.selected_finding_ids, vec!["finding-tool-waitlist"]);
    assert_eq!(request.evidence_spans.len(), 1);
    assert!(request
        .evidence_spans
        .first()
        .expect("span exists")
        .contains("waitlist"));
}

#[tokio::test]
async fn test_req_rust_101_patch_openai_returns_plain_prompt_text() {
    let rewritten_prompt = "# Rewritten Front Desk Prompt\n\nUse config for provider schedules.";
    let server = LocalResponseServer::spawn_with_output_text(rewritten_prompt);
    let client = OpenAiHttpClient::with_endpoint(server.endpoint());
    let config = OpenAiRuntimeConfig::from_api_key("sk-test-secret".to_string());
    let request = build_patch_request(
        &config,
        ASSIGNMENT_PROMPT,
        &sample_finding_groups(),
        &["finding-tool-waitlist".to_string()],
    )
    .expect("patch request builds");

    let candidate = client
        .patch_selected_prompt(request)
        .await
        .expect("plain-text rewrite succeeds");
    let updated =
        parse_assignment_prompt_json(&candidate.updated_prompt_json).expect("candidate parses");
    let original = parse_assignment_prompt_json(ASSIGNMENT_PROMPT).expect("fixture parses");
    let body = server.request_body();
    let payload: serde_json::Value = serde_json::from_str(&body).expect("payload parses");

    assert!(payload.get("text").is_none());
    assert_eq!(payload["max_output_tokens"], 12_000);
    assert_eq!(updated.general_prompt, rewritten_prompt);
    assert_eq!(updated.agent_name, original.agent_name);
    assert_eq!(updated.model, original.model);
    assert_eq!(updated.general_tools, original.general_tools);
    assert_eq!(candidate.applied_finding_ids, vec!["finding-tool-waitlist"]);
    assert!(!candidate.updated_prompt_json.contains("sk-test-secret"));
}

#[test]
fn test_req_rust_102_rewrite_preserves_assignment_envelope() {
    let rewritten_prompt = "# Safer Prompt\n\nNever promise waitlist state without a tool.";
    let updated_json = create_prompt_candidate_from_rewrite(ASSIGNMENT_PROMPT, rewritten_prompt)
        .expect("rewrite applies");
    let updated = parse_assignment_prompt_json(&updated_json).expect("candidate parses");
    let original = parse_assignment_prompt_json(ASSIGNMENT_PROMPT).expect("fixture parses");

    assert_eq!(updated.general_prompt, rewritten_prompt);
    assert_eq!(updated.agent_name, original.agent_name);
    assert_eq!(updated.model, original.model);
    assert_eq!(updated.general_tools, original.general_tools);
}

#[test]
fn test_req_rust_102_empty_rewrite_is_rejected() {
    let error = create_prompt_candidate_from_rewrite(ASSIGNMENT_PROMPT, "   ")
        .expect_err("empty rewrite rejects");

    assert_eq!(error.code(), "invalid_request");
    assert!(error.to_string().contains("empty rewritten prompt"));
}

#[tokio::test]
async fn test_req_rust_103_incomplete_patch_response_is_typed() {
    let server = LocalResponseServer::spawn_with_response_body(
        serde_json::json!({
            "status": "incomplete",
            "incomplete_details": {
                "reason": "max_output_tokens"
            },
            "output": [
                {
                    "type": "message",
                    "content": [
                        {
                            "type": "output_text",
                            "text": "{\"export\":{\"general_prompt\":\"truncated"
                        }
                    ]
                }
            ]
        })
        .to_string(),
    );
    let client = OpenAiHttpClient::with_endpoint(server.endpoint());
    let config = OpenAiRuntimeConfig::from_api_key("sk-test-secret".to_string());
    let request = build_patch_request(
        &config,
        ASSIGNMENT_PROMPT,
        &sample_finding_groups(),
        &["finding-tool-waitlist".to_string()],
    )
    .expect("patch request builds");

    let error = client
        .patch_selected_prompt(request)
        .await
        .expect_err("incomplete response rejects");

    assert_eq!(error.code(), "openai_incomplete");
    assert!(error.to_string().contains("max_output_tokens"));
}

#[test]
fn test_req_rust_001_openai_payloads_skip_api_key() {
    let secret = "sk-proj-secret-that-must-not-serialize".to_string();
    let config = OpenAiRuntimeConfig::from_api_key(secret.clone());
    let prompt = parse_assignment_prompt_json(ASSIGNMENT_PROMPT).expect("fixture parses");
    let normalized = normalize_prompt_sections_model(&prompt).expect("normalizes");
    let judge_request = build_judge_request(&config, &prompt, &normalized).expect("judge builds");
    let patch_request = build_patch_request(
        &config,
        ASSIGNMENT_PROMPT,
        &sample_finding_groups(),
        &["finding-tool-waitlist".to_string()],
    )
    .expect("patch builds");
    let verify_request = pie_tauri_core::build_llm_verify_request(
        secret.clone(),
        ASSIGNMENT_PROMPT.to_string(),
        ASSIGNMENT_PROMPT.to_string(),
        vec!["finding-tool-waitlist".to_string()],
    );
    let reverify_request = ReverifyPromptRequest {
        api_key: secret.clone(),
        original_prompt_json: ASSIGNMENT_PROMPT.to_string(),
        updated_prompt_json: ASSIGNMENT_PROMPT.to_string(),
        previous_finding_groups: sample_finding_groups(),
        updated_version_name: "updated_prompt_v20260615120000000".to_string(),
    };

    let serialized = [
        serde_json::to_string(&judge_request).expect("judge serializes"),
        serde_json::to_string(&patch_request).expect("patch serializes"),
        serde_json::to_string(&verify_request).expect("verify serializes"),
        serde_json::to_string(&reverify_request).expect("reverify serializes"),
    ]
    .join("\n");

    assert!(!serialized.contains(&secret));
    assert!(serialized.contains(OPENAI_MODEL_ID));
}

#[tokio::test]
async fn test_req_tauri_051_reverify_compares_previous_findings() {
    let client = FakeOpenAiClient::ready();
    let result = reverify_prompt(
        ReverifyPromptRequest {
            api_key: "user-provided-key".to_string(),
            original_prompt_json: ASSIGNMENT_PROMPT.to_string(),
            updated_prompt_json: ASSIGNMENT_PROMPT
                .replace("offer to put them on a waitlist", "offer staff handoff"),
            previous_finding_groups: sample_finding_groups(),
            updated_version_name: "updated_prompt_v20260615120000000".to_string(),
        },
        &client,
    )
    .await
    .expect("reverify succeeds");

    assert_eq!(
        result.prompt_version_name,
        "updated_prompt_v20260615120000000"
    );
    assert_eq!(result.model_label, "Model: gpt-5.4-mini");
    assert!(result
        .finding_statuses
        .iter()
        .any(|status| status.finding_id == "finding-tool-waitlist"
            && status.status == ReverifyStatus::Fixed));
}

#[tokio::test]
async fn test_req_tauri_011_empty_selection_is_rejected() {
    let client = FakeOpenAiClient::ready();
    let error = apply_selected_fixes(
        SelectedFixesRequest {
            original_prompt_json: ASSIGNMENT_PROMPT.to_string(),
            finding_groups: sample_finding_groups(),
            selected_finding_ids: vec![],
            api_key: "user-provided-key".to_string(),
        },
        &client,
    )
    .await
    .expect_err("empty selection rejects");

    assert_eq!(error.code, "empty_selection");
}

#[tokio::test]
async fn test_req_tauri_013_patch_failure_preserves_retry_state() {
    let client = FakeOpenAiClient::patch_failure(PatchFailureMode::SelectedFindingFailed {
        finding_id: "finding-tool-waitlist".to_string(),
    });
    let error = apply_selected_fixes(
        SelectedFixesRequest {
            original_prompt_json: ASSIGNMENT_PROMPT.to_string(),
            finding_groups: sample_finding_groups(),
            selected_finding_ids: vec!["finding-tool-waitlist".to_string()],
            api_key: "user-provided-key".to_string(),
        },
        &client,
    )
    .await
    .expect_err("patch failure returns typed app error");

    assert_eq!(error.code, "patch_failed");
    assert!(error.message.contains("finding-tool-waitlist"));
    assert!(error.hint.contains("Retry"));
}

#[test]
fn test_req_tauri_018_deterministic_verifier_flags_known_scenarios() {
    let prompt = parse_assignment_prompt_json(ASSIGNMENT_PROMPT).expect("fixture parses");
    let checks = check_update_deterministically(&prompt, ASSIGNMENT_PROMPT, ASSIGNMENT_PROMPT)
        .expect("checks run");

    assert!(checks
        .iter()
        .any(|check| check.scenario_id == "waitlist_without_tool" && !check.passed));
    assert!(checks
        .iter()
        .any(|check| check.scenario_id == "unsafe_reschedule" && !check.passed));
    assert!(checks
        .iter()
        .any(|check| check.scenario_id == "date_format_conflict" && !check.passed));
}

#[tokio::test]
async fn test_req_tauri_017_verify_exports_report_with_model_label() {
    let client = FakeOpenAiClient::ready();
    let report = verify_and_export_update(
        ASSIGNMENT_PROMPT.to_string(),
        ASSIGNMENT_PROMPT.to_string(),
        vec!["finding-tool-waitlist".to_string()],
        "user-provided-key".to_string(),
        &client,
    )
    .await
    .expect("verification report exports");

    assert!(report.markdown_report.contains("Model: gpt-5.4-mini"));
    assert!(report.markdown_report.contains("Before/After Verification"));
    assert!(!report.markdown_report.contains("user-provided-key"));
}

#[test]
fn test_req_tauri_010_findings_export_has_no_secret() {
    let export = export_findings_markdown_text(&sample_finding_groups(), model_label_text())
        .expect("findings export succeeds");

    assert!(export.contains("Model: gpt-5.4-mini"));
    assert!(export.contains("finding-tool-waitlist"));
    assert!(!export.contains("user-provided-key"));
}

fn sample_finding_groups() -> Vec<FindingGroup> {
    vec![FindingGroup {
        section: JudgeSection::ToolGaps,
        findings: vec![Finding {
            finding_id: "finding-tool-waitlist".to_string(),
            title: "Waitlist promise has no durable tool".to_string(),
            severity: FindingSeverity::High,
            prompt_evidence: "offer to put them on a waitlist".to_string(),
            impact: "The caller may believe a state change happened when no tool can do it."
                .to_string(),
            suggested_fix: "Use honest handoff language unless a waitlist tool exists.".to_string(),
            fix_mode: FixMode::AutoFixable,
            verification_scenario: "waitlist_without_tool".to_string(),
        }],
    }]
}

fn finding_group_with_mode(
    section: JudgeSection,
    finding_id: &str,
    fix_mode: FixMode,
) -> FindingGroup {
    FindingGroup {
        section,
        findings: vec![Finding {
            finding_id: finding_id.to_string(),
            title: format!("Title for {finding_id}"),
            severity: FindingSeverity::High,
            prompt_evidence: "prompt evidence".to_string(),
            impact: "call impact".to_string(),
            suggested_fix: "minimal fix".to_string(),
            fix_mode,
            verification_scenario: "scenario".to_string(),
        }],
    }
}

struct LocalResponseServer {
    endpoint: String,
    captured_body: std::sync::Arc<std::sync::Mutex<Option<String>>>,
}

impl LocalResponseServer {
    fn spawn() -> Self {
        Self::spawn_with_output_text("OK")
    }

    fn spawn_with_output_text(output_text: &str) -> Self {
        Self::spawn_with_response_body(
            serde_json::json!({
                "output": [
                    {
                        "type": "message",
                        "content": [
                            {
                                "type": "output_text",
                                "text": output_text
                            }
                        ]
                    }
                ]
            })
            .to_string(),
        )
    }

    fn spawn_with_response_body(response_body: String) -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("local listener binds");
        let endpoint = format!(
            "http://{}/v1/responses",
            listener.local_addr().expect("addr")
        );
        let captured_body = std::sync::Arc::new(std::sync::Mutex::new(None));
        let captured_body_for_thread = std::sync::Arc::clone(&captured_body);

        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("request accepted");
            let body = read_http_request_body(&mut stream);
            *captured_body_for_thread.lock().expect("capture lock") = Some(body);

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            use std::io::Write;
            stream
                .write_all(response.as_bytes())
                .expect("response writes");
        });

        Self {
            endpoint,
            captured_body,
        }
    }

    fn endpoint(&self) -> String {
        self.endpoint.clone()
    }

    fn request_body(&self) -> String {
        self.captured_body
            .lock()
            .expect("capture lock")
            .clone()
            .expect("request body captured")
    }
}

fn read_http_request_body(stream: &mut std::net::TcpStream) -> String {
    use std::io::Read;

    let mut buffer = Vec::new();
    let mut scratch = [0_u8; 1024];
    loop {
        let bytes_read = stream.read(&mut scratch).expect("request reads");
        if bytes_read == 0 {
            break;
        }
        buffer.extend_from_slice(&scratch[..bytes_read]);
        if let Some(body) = parse_complete_http_body(&buffer) {
            return body;
        }
    }

    String::new()
}

fn parse_complete_http_body(buffer: &[u8]) -> Option<String> {
    let header_end = buffer.windows(4).position(|window| window == b"\r\n\r\n")?;
    let header_text = String::from_utf8_lossy(&buffer[..header_end]);
    let content_length = header_text
        .lines()
        .find_map(|line| {
            line.strip_prefix("content-length:")
                .or_else(|| line.strip_prefix("Content-Length:"))
        })
        .and_then(|value| value.trim().parse::<usize>().ok())?;
    let body_start = header_end + 4;
    if buffer.len() < body_start + content_length {
        return None;
    }

    Some(String::from_utf8_lossy(&buffer[body_start..body_start + content_length]).to_string())
}
