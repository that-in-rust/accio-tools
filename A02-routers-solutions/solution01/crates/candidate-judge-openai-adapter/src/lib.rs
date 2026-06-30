use catalog_router_core_engine::{CandidateEvidenceCardData, RouterTypedErrorKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JudgeCandidateRequestData {
    pub query: String,
    pub recent_context: Option<String>,
    pub candidates: Vec<CandidateEvidenceCardData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JudgeDecisionKindData {
    SelectTool,
    Abstain,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JudgeDecisionOutputData {
    pub selected_tool_id: Option<String>,
    pub confidence: f64,
    pub reason: String,
    pub decision: JudgeDecisionKindData,
    pub needs_more_metadata: bool,
}

pub fn judge_candidate_tools_top(
    request: &JudgeCandidateRequestData,
) -> Result<JudgeDecisionOutputData, RouterTypedErrorKind> {
    if request.candidates.len() > 5 {
        return Err(RouterTypedErrorKind::JudgeConfigurationFailed {
            message: "judge payload cannot contain more than five candidates".to_string(),
        });
    }
    if request.candidates.is_empty() {
        return Ok(JudgeDecisionOutputData {
            selected_tool_id: None,
            confidence: 0.0,
            reason: "No CPU candidate had enough evidence to route safely.".to_string(),
            decision: JudgeDecisionKindData::Abstain,
            needs_more_metadata: true,
        });
    }
    let selected_tool_id = request
        .candidates
        .first()
        .map(|candidate| candidate.tool_id.clone());
    Ok(JudgeDecisionOutputData {
        selected_tool_id,
        confidence: 0.5,
        reason: "Mock judge selects the highest CPU-ranked candidate.".to_string(),
        decision: JudgeDecisionKindData::SelectTool,
        needs_more_metadata: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn create_candidate_card_data(id: &str, rank: usize) -> CandidateEvidenceCardData {
        CandidateEvidenceCardData {
            rank,
            score: 1.0,
            tool_id: id.to_string(),
            matched_terms: vec![],
            matched_fields: vec![],
            capability_match: vec![],
            risk: "low".to_string(),
            why_matched: "test candidate".to_string(),
            signal_contributions: BTreeMap::new(),
        }
    }

    #[test]
    fn test_judge_payload_limit() {
        let request = JudgeCandidateRequestData {
            query: "search".to_string(),
            recent_context: None,
            candidates: (0..6)
                .map(|index| create_candidate_card_data(&format!("tool.{index}"), index + 1))
                .collect(),
        };
        assert!(judge_candidate_tools_top(&request).is_err());
    }

    #[test]
    fn judge_abstains_without_candidates() {
        let request = JudgeCandidateRequestData {
            query: "route unsupported action".to_string(),
            recent_context: None,
            candidates: Vec::new(),
        };
        let decision = judge_candidate_tools_top(&request).expect("judge should respond");

        assert_eq!(decision.decision, JudgeDecisionKindData::Abstain);
        assert_eq!(decision.selected_tool_id, None);
        assert!(decision.needs_more_metadata);
    }

    #[test]
    fn judge_payload_excludes_labels() {
        let request = JudgeCandidateRequestData {
            query: "search".to_string(),
            recent_context: None,
            candidates: vec![create_candidate_card_data("tool.1", 1)],
        };
        let payload = serde_json::to_string(&request).expect("payload should serialize");

        for forbidden in [
            "required_tool_ids",
            "should_route",
            "graded_relevance",
            "source_expected_tools",
            "failure_modes",
        ] {
            assert!(!payload.contains(forbidden));
        }
    }
}
