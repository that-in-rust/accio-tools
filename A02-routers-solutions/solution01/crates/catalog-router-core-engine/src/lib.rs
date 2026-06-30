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
            "lexical" | "lexical-bm25" => Ok(Self::Lexical),
            "schema-aware" | "schema_aware" | "schema-aware-bm25" => Ok(Self::SchemaAware),
            "hybrid" | "hybrid-rrf" => Ok(Self::Hybrid),
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
        if !tool.input_schema.is_object() {
            return Err(RouterTypedErrorKind::CatalogValidationFailed {
                message: format!("tool {} has non-object input schema", tool.id),
            });
        }
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

    let mut cards = Vec::new();
    for (index, (score, tool_id, matched_terms)) in
        scored.into_iter().take(max_candidates).enumerate()
    {
        let tool = tools
            .iter()
            .find(|candidate| candidate.id == tool_id)
            .ok_or_else(|| RouterTypedErrorKind::CatalogValidationFailed {
                message: format!("ranked tool {tool_id} was not found in catalog"),
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
    let query_terms: BTreeSet<String> = tokenize_text_terms_only(query).into_iter().collect();
    for candidate in &mut candidates {
        let Some(tool) = tools.iter().find(|tool| tool.id == candidate.tool_id) else {
            continue;
        };
        let mut schema_score = 0.0;
        let mut capability_match = Vec::new();
        let name_terms: BTreeSet<String> =
            tokenize_text_terms_only(&tool.name).into_iter().collect();
        let description_terms: BTreeSet<String> = tokenize_text_terms_only(&tool.description)
            .into_iter()
            .collect();
        let schema_text = flatten_json_value_text(&tool.input_schema);
        let schema_terms: BTreeSet<String> =
            tokenize_text_terms_only(&schema_text).into_iter().collect();

        if query_terms.iter().any(|term| name_terms.contains(term)) {
            schema_score += 2.0;
            capability_match.push("operation".to_string());
        }
        if query_terms
            .iter()
            .any(|term| description_terms.contains(term))
        {
            schema_score += 2.0;
            capability_match.push("object".to_string());
        }
        if query_terms.iter().any(|term| schema_terms.contains(term)) {
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

pub fn fuse_hybrid_rankings_rrf(
    lexical: &[CandidateEvidenceCardData],
    schema: &[CandidateEvidenceCardData],
    max_candidates: usize,
) -> Vec<CandidateEvidenceCardData> {
    let mut scores: HashMap<String, CandidateEvidenceCardData> = HashMap::new();
    for (weight, candidates) in [(1.0, lexical), (1.4, schema)] {
        for candidate in candidates {
            let entry = scores
                .entry(candidate.tool_id.clone())
                .or_insert_with(|| candidate.clone());
            entry.score += weight / (60.0 + candidate.rank as f64);
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

fn create_tool_search_text(tool: &ToolCatalogRecordData) -> String {
    [
        tool.id.as_str(),
        tool.source_tool_id.as_deref().unwrap_or_default(),
        tool.server_name.as_deref().unwrap_or_default(),
        tool.name.as_str(),
        tool.description.as_str(),
        &tool.tags.join(" "),
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
        ("server_name", tool.server_name.clone().unwrap_or_default()),
        ("name", tool.name.clone()),
        ("description", tool.description.clone()),
        ("tags", tool.tags.join(" ")),
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

    #[test]
    fn test_catalog_duplicate_rejects() {
        let tools = vec![
            create_tool_record_value("tool.search", "search", "search docs"),
            create_tool_record_value("tool.search", "search again", "search docs"),
        ];
        assert!(validate_catalog_schema_input(&tools).is_err());
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
}
