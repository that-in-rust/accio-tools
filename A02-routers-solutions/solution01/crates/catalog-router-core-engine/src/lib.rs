use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RouterTypedErrorKind {
    #[error("dataset path does not exist: {path}")]
    MissingDatasetPath { path: String },
    #[error("failed to read {path}: {message}")]
    ReadFileFailed { path: String, message: String },
    #[error("failed to parse {path}: {message}")]
    ParseJsonFailed { path: String, message: String },
    #[error("catalog validation failed: {message}")]
    CatalogValidationFailed { message: String },
    #[error("query validation failed: {message}")]
    QueryValidationFailed { message: String },
    #[error("unsupported router mode: {mode}")]
    UnsupportedRouterMode { mode: String },
    #[error("judge configuration failed: {message}")]
    JudgeConfigurationFailed { message: String },
}

impl From<serde_json::Error> for RouterTypedErrorKind {
    fn from(error: serde_json::Error) -> Self {
        Self::ParseJsonFailed {
            path: "inline-json".to_string(),
            message: error.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCatalogRecordData {
    pub id: String,
    #[serde(default)]
    pub source_tool_id: Option<String>,
    #[serde(default)]
    pub server_id: Option<String>,
    #[serde(default)]
    pub server_name: Option<String>,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub input_schema: serde_json::Value,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub source: serde_json::Value,
    #[serde(default)]
    pub metadata: serde_json::Value,
    #[serde(flatten)]
    pub unknown_metadata: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GradedRelevanceItemData {
    pub tool_id: String,
    pub relevance: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouteQueryInputData {
    pub id: String,
    pub query: String,
    #[serde(default)]
    pub required_tool_ids: Vec<String>,
    #[serde(default)]
    pub should_route: bool,
    #[serde(default)]
    pub graded_relevance: Vec<GradedRelevanceItemData>,
    #[serde(default)]
    pub source_expected_tools: Vec<String>,
    #[serde(default)]
    pub failure_modes: Vec<String>,
    #[serde(flatten)]
    pub unknown_metadata: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateEvidenceCardData {
    pub rank: usize,
    pub score: f64,
    pub tool_id: String,
    pub matched_terms: Vec<String>,
    pub matched_fields: Vec<String>,
    pub capability_match: Vec<String>,
    pub risk: String,
    pub why_matched: String,
    pub signal_contributions: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RouterModeNameData {
    Lexical,
    SchemaAware,
    Hybrid,
}

impl std::str::FromStr for RouterModeNameData {
    type Err = RouterTypedErrorKind;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "lexical" => Ok(Self::Lexical),
            "schema-aware" | "schema_aware" => Ok(Self::SchemaAware),
            "hybrid" => Ok(Self::Hybrid),
            other => Err(RouterTypedErrorKind::UnsupportedRouterMode {
                mode: other.to_string(),
            }),
        }
    }
}

pub fn validate_catalog_schema_input(
    tools: &[ToolCatalogRecordData],
) -> Result<(), RouterTypedErrorKind> {
    let mut seen_ids = BTreeSet::new();
    for tool in tools {
        if tool.id.trim().is_empty() {
            return Err(RouterTypedErrorKind::CatalogValidationFailed {
                message: "tool id is required".to_string(),
            });
        }
        if !seen_ids.insert(tool.id.clone()) {
            return Err(RouterTypedErrorKind::CatalogValidationFailed {
                message: format!("duplicate tool id {}", tool.id),
            });
        }
        if tool.name.trim().is_empty() {
            return Err(RouterTypedErrorKind::CatalogValidationFailed {
                message: format!("tool {} has empty name", tool.id),
            });
        }
        if tool.description.trim().is_empty() {
            return Err(RouterTypedErrorKind::CatalogValidationFailed {
                message: format!("tool {} has empty description", tool.id),
            });
        }
        validate_optional_metadata_text(&tool.source_tool_id, "source_tool_id", &tool.id)?;
        validate_optional_metadata_text(&tool.server_id, "server_id", &tool.id)?;
        validate_optional_metadata_text(&tool.server_name, "server_name", &tool.id)?;
        if !tool.input_schema.is_object() {
            return Err(RouterTypedErrorKind::CatalogValidationFailed {
                message: format!("tool {} has non-object input schema", tool.id),
            });
        }
    }
    Ok(())
}

fn validate_optional_metadata_text(
    value: &Option<String>,
    field_name: &str,
    tool_id: &str,
) -> Result<(), RouterTypedErrorKind> {
    if value.as_deref().map(str::trim).is_some_and(str::is_empty) {
        return Err(RouterTypedErrorKind::CatalogValidationFailed {
            message: format!("tool {tool_id} has empty {field_name}"),
        });
    }
    Ok(())
}

pub fn validate_query_record_input(
    queries: &[RouteQueryInputData],
) -> Result<(), RouterTypedErrorKind> {
    for query in queries {
        if query.id.trim().is_empty() {
            return Err(RouterTypedErrorKind::QueryValidationFailed {
                message: "query id is required".to_string(),
            });
        }
        if query.query.trim().is_empty() {
            return Err(RouterTypedErrorKind::QueryValidationFailed {
                message: format!("query {} has empty text", query.id),
            });
        }
        if query
            .required_tool_ids
            .iter()
            .any(|tool_id| tool_id.trim().is_empty())
        {
            return Err(RouterTypedErrorKind::QueryValidationFailed {
                message: format!("query {} has an invalid required tool id", query.id),
            });
        }
        if query.should_route && query.required_tool_ids.is_empty() {
            return Err(RouterTypedErrorKind::QueryValidationFailed {
                message: format!("query {} should route but has no required tools", query.id),
            });
        }
    }
    Ok(())
}

pub fn rank_lexical_tools_baseline(
    query: &str,
    tools: &[ToolCatalogRecordData],
    threshold: f64,
    max_candidates: usize,
) -> Result<Vec<CandidateEvidenceCardData>, RouterTypedErrorKind> {
    let (tool_counts, inverse_frequency) = build_tool_term_index(tools);
    let query_counts = count_text_terms_map(query);
    let mut scored: Vec<(f64, String, Vec<String>)> = Vec::new();

    for tool in tools {
        let Some(counts) = tool_counts.get(&tool.id) else {
            continue;
        };
        let mut score = 0.0;
        let mut matched_terms = Vec::new();
        for (term, query_count) in &query_counts {
            if let Some(tool_count) = counts.get(term) {
                let idf = inverse_frequency.get(term).copied().unwrap_or(1.0);
                score += *query_count as f64 * idf * (1.0 + (1.0 + *tool_count as f64).ln());
                matched_terms.push(term.clone());
            }
        }
        if score >= threshold {
            scored.push((score, tool.id.clone(), matched_terms));
        }
    }

    scored.sort_by(|left, right| {
        right
            .0
            .partial_cmp(&left.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| right.1.cmp(&left.1))
    });

    let tools_by_id: HashMap<&str, &ToolCatalogRecordData> =
        tools.iter().map(|tool| (tool.id.as_str(), tool)).collect();
    let mut cards = Vec::new();
    for (index, (score, tool_id, matched_terms)) in
        scored.into_iter().take(max_candidates).enumerate()
    {
        let tool = tools_by_id.get(tool_id.as_str()).ok_or_else(|| {
            RouterTypedErrorKind::CatalogValidationFailed {
                message: format!("ranked tool {tool_id} was not found in catalog"),
            }
        })?;
        let matched_fields = collect_matched_fields_names(tool, &matched_terms);
        let mut signal_contributions = BTreeMap::new();
        signal_contributions.insert("lexical".to_string(), score);
        cards.push(CandidateEvidenceCardData {
            rank: index + 1,
            score,
            tool_id,
            matched_terms: matched_terms.clone(),
            matched_fields,
            capability_match: Vec::new(),
            risk: "low".to_string(),
            why_matched: create_lexical_reason_text(tool, &matched_terms),
            signal_contributions,
        });
    }
    Ok(cards)
}

pub fn score_schema_capability_signals(
    query: &str,
    tools: &[ToolCatalogRecordData],
    threshold: f64,
    max_candidates: usize,
) -> Result<Vec<CandidateEvidenceCardData>, RouterTypedErrorKind> {
    let mut candidates = rank_lexical_tools_baseline(query, tools, 0.0, tools.len())?;
    let tools_by_id: HashMap<&str, &ToolCatalogRecordData> =
        tools.iter().map(|tool| (tool.id.as_str(), tool)).collect();
    let query_terms: BTreeSet<String> = tokenize_text_terms_only(query).into_iter().collect();
    for candidate in &mut candidates {
        let Some(tool) = tools_by_id.get(candidate.tool_id.as_str()) else {
            continue;
        };
        let mut schema_score = 0.0;
        let mut capability_match = Vec::new();
        let name_terms = tokenize_text_terms_only(&tool.name);
        let description_terms = tokenize_text_terms_only(&tool.description);
        let schema_text = flatten_json_schema_text(&tool.input_schema);
        let schema_terms = tokenize_text_terms_only(&schema_text);

        if field_terms_contain_query_term(&name_terms, &query_terms) {
            schema_score += 2.0;
            capability_match.push("operation".to_string());
        }
        if field_terms_contain_query_term(&description_terms, &query_terms) {
            schema_score += 2.0;
            capability_match.push("object".to_string());
        }
        if field_terms_contain_query_term(&schema_terms, &query_terms) {
            schema_score += 1.5;
            capability_match.push("parameter".to_string());
        }
        if query_terms.iter().any(|term| {
            ["delete", "update", "create", "write", "send", "post"].contains(&term.as_str())
        }) {
            if description_terms
                .iter()
                .chain(name_terms.iter())
                .any(|term| {
                    ["delete", "update", "create", "write", "send", "post"].contains(&term.as_str())
                })
            {
                schema_score += 1.25;
                capability_match.push("write_alignment".to_string());
            } else {
                schema_score -= 3.0;
                candidate.risk = "ambiguous_write".to_string();
            }
        }
        candidate.score += schema_score;
        candidate
            .signal_contributions
            .insert("schema".to_string(), schema_score);
        candidate.capability_match = capability_match;
        candidate.why_matched = format!(
            "{} Schema contribution {:.3}.",
            candidate.why_matched, schema_score
        );
    }
    candidates.retain(|candidate| candidate.score >= threshold);
    candidates.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| right.tool_id.cmp(&left.tool_id))
    });
    candidates.truncate(max_candidates);
    for (index, candidate) in candidates.iter_mut().enumerate() {
        candidate.rank = index + 1;
    }
    Ok(candidates)
}

fn field_terms_contain_query_term(field_terms: &[String], query_terms: &BTreeSet<String>) -> bool {
    field_terms
        .iter()
        .any(|field_term| query_terms.contains(field_term))
}

pub fn rank_tools_for_mode(
    query: &str,
    tools: &[ToolCatalogRecordData],
    router_mode: RouterModeNameData,
    threshold: f64,
    max_candidates: usize,
) -> Result<Vec<CandidateEvidenceCardData>, RouterTypedErrorKind> {
    match router_mode {
        RouterModeNameData::Lexical => {
            rank_lexical_tools_baseline(query, tools, threshold, max_candidates)
        }
        RouterModeNameData::SchemaAware => {
            score_schema_capability_signals(query, tools, threshold, max_candidates)
        }
        RouterModeNameData::Hybrid => {
            let lexical = rank_lexical_tools_baseline(query, tools, threshold, max_candidates)?;
            let schema = score_schema_capability_signals(query, tools, threshold, max_candidates)?;
            Ok(fuse_hybrid_rankings_rrf(&lexical, &schema, max_candidates))
        }
    }
}

pub fn fuse_hybrid_rankings_rrf(
    lexical: &[CandidateEvidenceCardData],
    schema: &[CandidateEvidenceCardData],
    max_candidates: usize,
) -> Vec<CandidateEvidenceCardData> {
    let mut scores: HashMap<String, CandidateEvidenceCardData> = HashMap::new();
    for (signal_name, weight, candidates) in
        [("lexical_rrf", 1.0, lexical), ("schema_rrf", 1.4, schema)]
    {
        for candidate in candidates {
            let contribution = weight / (60.0 + candidate.rank as f64);
            let entry = scores.entry(candidate.tool_id.clone()).or_insert_with(|| {
                let mut card = candidate.clone();
                card.score = 0.0;
                card.signal_contributions = BTreeMap::new();
                card
            });
            entry.score += contribution;
            entry
                .signal_contributions
                .insert(signal_name.to_string(), contribution);
            merge_signal_contributions_map(entry, candidate);
            merge_candidate_evidence_lists(entry, candidate);
        }
    }
    let mut fused: Vec<_> = scores.into_values().collect();
    fused.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| right.tool_id.cmp(&left.tool_id))
    });
    fused.truncate(max_candidates);
    for (index, candidate) in fused.iter_mut().enumerate() {
        candidate.rank = index + 1;
    }
    fused
}

fn merge_signal_contributions_map(
    target: &mut CandidateEvidenceCardData,
    source: &CandidateEvidenceCardData,
) {
    for (name, value) in &source.signal_contributions {
        target
            .signal_contributions
            .entry(name.clone())
            .or_insert(*value);
    }
    if target.risk == "low" && source.risk != "low" {
        target.risk = source.risk.clone();
    }
}

fn merge_candidate_evidence_lists(
    target: &mut CandidateEvidenceCardData,
    source: &CandidateEvidenceCardData,
) {
    for term in &source.matched_terms {
        if !target.matched_terms.contains(term) {
            target.matched_terms.push(term.clone());
        }
    }
    for field in &source.matched_fields {
        if !target.matched_fields.contains(field) {
            target.matched_fields.push(field.clone());
        }
    }
    for capability in &source.capability_match {
        if !target.capability_match.contains(capability) {
            target.capability_match.push(capability.clone());
        }
    }
}

pub fn tokenize_text_terms_only(value: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let mut current = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            current.push(character.to_ascii_lowercase());
        } else if !current.is_empty() {
            terms.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        terms.push(current);
    }
    terms
}

fn count_text_terms_map(value: &str) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for term in tokenize_text_terms_only(value) {
        *counts.entry(term).or_insert(0) += 1;
    }
    counts
}

fn flatten_json_value_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => map
            .values()
            .map(flatten_json_value_text)
            .collect::<Vec<_>>()
            .join(" "),
        serde_json::Value::Array(values) => values
            .iter()
            .map(flatten_json_value_text)
            .collect::<Vec<_>>()
            .join(" "),
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Null => String::new(),
    }
}

fn flatten_json_schema_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => map
            .iter()
            .flat_map(|(key, value)| [key.clone(), flatten_json_schema_text(value)])
            .collect::<Vec<_>>()
            .join(" "),
        serde_json::Value::Array(values) => values
            .iter()
            .map(flatten_json_schema_text)
            .collect::<Vec<_>>()
            .join(" "),
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Number(number) => number.to_string(),
        serde_json::Value::Bool(boolean) => boolean.to_string(),
        serde_json::Value::Null => String::new(),
    }
}

fn create_tool_search_text(tool: &ToolCatalogRecordData) -> String {
    [
        tool.id.as_str(),
        tool.source_tool_id.as_deref().unwrap_or_default(),
        tool.server_id.as_deref().unwrap_or_default(),
        tool.server_name.as_deref().unwrap_or_default(),
        tool.name.as_str(),
        tool.description.as_str(),
        &tool.tags.join(" "),
        &flatten_json_value_text(&tool.source),
        &flatten_json_value_text(&tool.input_schema),
    ]
    .join(" ")
}

fn build_tool_term_index(
    tools: &[ToolCatalogRecordData],
) -> (
    HashMap<String, BTreeMap<String, usize>>,
    HashMap<String, f64>,
) {
    let mut term_counts = HashMap::new();
    let mut document_frequency: HashMap<String, usize> = HashMap::new();
    for tool in tools {
        let counts = count_text_terms_map(&create_tool_search_text(tool));
        for term in counts.keys() {
            *document_frequency.entry(term.clone()).or_insert(0) += 1;
        }
        term_counts.insert(tool.id.clone(), counts);
    }
    let total_documents = tools.len() as f64;
    let inverse_frequency = document_frequency
        .into_iter()
        .map(|(term, frequency)| {
            let idf = ((1.0 + total_documents) / (1.0 + frequency as f64)).ln() + 1.0;
            (term, idf)
        })
        .collect();
    (term_counts, inverse_frequency)
}

fn collect_matched_fields_names(
    tool: &ToolCatalogRecordData,
    matched_terms: &[String],
) -> Vec<String> {
    let mut fields = BTreeSet::new();
    let matched: BTreeSet<&str> = matched_terms.iter().map(String::as_str).collect();
    let field_values = [
        ("id", tool.id.as_str().to_string()),
        (
            "source_tool_id",
            tool.source_tool_id.clone().unwrap_or_default(),
        ),
        ("server_id", tool.server_id.clone().unwrap_or_default()),
        ("server_name", tool.server_name.clone().unwrap_or_default()),
        ("name", tool.name.clone()),
        ("description", tool.description.clone()),
        ("tags", tool.tags.join(" ")),
        ("source", flatten_json_value_text(&tool.source)),
        ("input_schema", flatten_json_value_text(&tool.input_schema)),
    ];
    for (field_name, value) in field_values {
        if tokenize_text_terms_only(&value)
            .iter()
            .any(|term| matched.contains(term.as_str()))
        {
            fields.insert(field_name.to_string());
        }
    }
    fields.into_iter().collect()
}

fn create_lexical_reason_text(tool: &ToolCatalogRecordData, matched_terms: &[String]) -> String {
    if matched_terms.is_empty() {
        format!("{} matched through low lexical fallback.", tool.id)
    } else {
        format!("{} matched terms: {}.", tool.id, matched_terms.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_tool_record_value(id: &str, name: &str, description: &str) -> ToolCatalogRecordData {
        ToolCatalogRecordData {
            id: id.to_string(),
            source_tool_id: None,
            server_id: None,
            server_name: Some("test".to_string()),
            name: name.to_string(),
            description: description.to_string(),
            input_schema: serde_json::json!({"type":"object","properties":{"query":{"type":"string"}}}),
            tags: vec![],
            source: serde_json::json!({}),
            metadata: serde_json::json!({}),
            unknown_metadata: BTreeMap::new(),
        }
    }

    fn create_channel_tool_data(id: &str, name: &str, description: &str) -> ToolCatalogRecordData {
        ToolCatalogRecordData {
            id: id.to_string(),
            source_tool_id: None,
            server_id: None,
            server_name: Some("test".to_string()),
            name: name.to_string(),
            description: description.to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "channel": { "type": "string" },
                    "message": { "type": "string" }
                }
            }),
            tags: vec![],
            source: serde_json::json!({}),
            metadata: serde_json::json!({}),
            unknown_metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn test_catalog_duplicate_rejects() {
        let tools = vec![
            create_tool_record_value("tool.search", "search", "search docs"),
            create_tool_record_value("tool.search", "search again", "search docs"),
        ];
        assert!(validate_catalog_schema_input(&tools).is_err());
    }

    #[test]
    fn catalog_metadata_rejects_empty_present_values() {
        let mut tool = create_tool_record_value("tool.search", "search", "search docs");
        tool.source_tool_id = Some(" ".to_string());
        let error = validate_catalog_schema_input(&[tool])
            .expect_err("present source metadata must not be blank");

        assert!(error.to_string().contains("empty source_tool_id"));
    }

    #[test]
    fn unknown_metadata_preserves_extra_fields() {
        let parsed: ToolCatalogRecordData = serde_json::from_value(serde_json::json!({
            "id": "tool.extra",
            "name": "extra_search",
            "description": "Search extra records",
            "input_schema": {"type": "object"},
            "tags": [],
            "custom_vendor_field": {"tier": "gold"}
        }))
        .expect("tool should parse");

        assert_eq!(
            parsed.unknown_metadata.get("custom_vendor_field"),
            Some(&serde_json::json!({"tier": "gold"}))
        );
    }

    #[test]
    fn lexical_scores_server_source_text() {
        let mut tool = create_tool_record_value("tool.github", "list_issues", "List issues");
        tool.server_id = Some("graph_openapi.github".to_string());
        tool.server_name = Some("github".to_string());
        tool.source = serde_json::json!({
            "repo": "graph_openapi",
            "path": "git-ref-repo/graph_openapi/github.json"
        });
        let ranked = rank_lexical_tools_baseline("github graph_openapi", &[tool], 0.0, 1)
            .expect("ranker should include source metadata");

        assert_eq!(ranked[0].tool_id, "tool.github");
        assert!(ranked[0].matched_fields.contains(&"server_id".to_string()));
        assert!(ranked[0]
            .matched_fields
            .contains(&"server_name".to_string()));
        assert!(ranked[0].matched_fields.contains(&"source".to_string()));
    }

    #[test]
    fn runtime_modes_reject_doc_label_aliases() {
        assert_eq!(
            "lexical".parse::<RouterModeNameData>().expect("lexical"),
            RouterModeNameData::Lexical
        );
        assert_eq!(
            "schema-aware"
                .parse::<RouterModeNameData>()
                .expect("schema-aware"),
            RouterModeNameData::SchemaAware
        );
        assert_eq!(
            "hybrid".parse::<RouterModeNameData>().expect("hybrid"),
            RouterModeNameData::Hybrid
        );

        for alias in ["lexical-bm25", "schema-aware-bm25", "hybrid-rrf"] {
            assert!(alias.parse::<RouterModeNameData>().is_err());
        }
    }

    #[test]
    fn test_lexical_baseline_parity() {
        let tools = vec![
            create_tool_record_value(
                "tool.search",
                "search_documents",
                "Search documents by keyword",
            ),
            create_tool_record_value("tool.delete", "delete_document", "Delete one document"),
        ];
        let ranked = rank_lexical_tools_baseline("search documents", &tools, 0.0, 2)
            .expect("ranker should run");
        assert_eq!(ranked[0].tool_id, "tool.search");
        assert!(ranked[0].matched_terms.contains(&"search".to_string()));
    }

    #[test]
    fn mode_router_adds_schema() {
        let tools = vec![create_channel_tool_data(
            "tool.channel",
            "send_notification",
            "Post a message",
        )];
        let ranked = rank_tools_for_mode(
            "send message to channel",
            &tools,
            RouterModeNameData::SchemaAware,
            0.0,
            5,
        )
        .expect("schema mode should rank");

        assert_eq!(ranked[0].tool_id, "tool.channel");
        assert!(ranked[0].signal_contributions.contains_key("schema"));
        assert!(ranked[0]
            .capability_match
            .contains(&"parameter".to_string()));
    }

    #[test]
    fn query_required_tool_ids_reject_empty_values() {
        let error = validate_query_record_input(&[RouteQueryInputData {
            id: "query-empty-tool".to_string(),
            query: "send message".to_string(),
            required_tool_ids: vec!["".to_string()],
            should_route: true,
            graded_relevance: vec![],
            source_expected_tools: vec![],
            failure_modes: vec![],
            unknown_metadata: BTreeMap::new(),
        }])
        .expect_err("empty required tool ids should fail validation");

        assert!(error.to_string().contains("invalid required tool id"));
    }

    #[test]
    fn hybrid_fusion_keeps_signals() {
        let tools = vec![
            create_tool_record_value("tool.channel", "send_notification", "Post a message"),
            create_tool_record_value("tool.reader", "message_reader", "Read a message"),
        ];
        let ranked = rank_tools_for_mode(
            "send message to channel",
            &tools,
            RouterModeNameData::Hybrid,
            0.0,
            5,
        )
        .expect("hybrid mode should rank");

        assert!(ranked[0].signal_contributions.contains_key("lexical_rrf"));
        assert!(ranked[0].signal_contributions.contains_key("schema_rrf"));
        assert!(ranked[0].signal_contributions.contains_key("schema"));
    }
}
