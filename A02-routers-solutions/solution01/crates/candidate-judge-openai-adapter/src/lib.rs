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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenAiJudgeConfigData {
    pub api_key: String,
    pub model: String,
    pub endpoint: String,
}

#[derive(Debug, Serialize)]
struct CompactCandidateCardData<'a> {
    rank: usize,
    score: f64,
    tool_id: &'a str,
    matched_terms: &'a [String],
    matched_fields: &'a [String],
    capability_match: &'a [String],
    risk: &'a str,
    why_matched: &'a str,
}

#[derive(Debug, Deserialize)]
struct OpenAiDecisionWireData {
    selected_tool_id: Option<String>,
    confidence: f64,
    reason: String,
    decision: JudgeDecisionKindData,
    needs_more_metadata: bool,
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
    if request
        .candidates
        .first()
        .is_some_and(|candidate| candidate.risk == "ambiguous_write")
    {
        return Ok(JudgeDecisionOutputData {
            selected_tool_id: None,
            confidence: 0.2,
            reason: "Top CPU candidate had ambiguous write risk, so the judge abstained."
                .to_string(),
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

pub fn create_openai_responses_payload(
    request: &JudgeCandidateRequestData,
    model: &str,
) -> Result<serde_json::Value, RouterTypedErrorKind> {
    validate_judge_candidate_limit(request)?;
    let compact_candidates = request
        .candidates
        .iter()
        .map(|candidate| CompactCandidateCardData {
            rank: candidate.rank,
            score: candidate.score,
            tool_id: &candidate.tool_id,
            matched_terms: &candidate.matched_terms,
            matched_fields: &candidate.matched_fields,
            capability_match: &candidate.capability_match,
            risk: &candidate.risk,
            why_matched: &candidate.why_matched,
        })
        .collect::<Vec<_>>();
    let user_payload = serde_json::json!({
        "query": request.query,
        "recent_context": request.recent_context,
        "candidates": compact_candidates,
    });
    Ok(serde_json::json!({
        "model": model,
        "input": [
            {
                "role": "system",
                "content": [
                    {
                        "type": "input_text",
                        "text": "You are a tool routing judge. Review only the provided CPU top-five candidate cards. Return one JSON decision: select_tool when one candidate safely satisfies the query, otherwise abstain. Prefer abstention over unsafe write exposure when write intent is ambiguous. Never invent tool ids."
                    }
                ]
            },
            {
                "role": "user",
                "content": [
                    {
                        "type": "input_text",
                        "text": serde_json::to_string(&user_payload).map_err(RouterTypedErrorKind::from)?
                    }
                ]
            }
        ],
        "text": {
            "format": {
                "type": "json_schema",
                "name": "tool_route_judge_decision",
                "strict": true,
                "schema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "decision": {
                            "type": "string",
                            "enum": ["select_tool", "abstain"]
                        },
                        "selected_tool_id": {
                            "type": ["string", "null"]
                        },
                        "confidence": {
                            "type": "number",
                            "minimum": 0,
                            "maximum": 1
                        },
                        "reason": {
                            "type": "string"
                        },
                        "needs_more_metadata": {
                            "type": "boolean"
                        }
                    },
                    "required": [
                        "decision",
                        "selected_tool_id",
                        "confidence",
                        "reason",
                        "needs_more_metadata"
                    ]
                }
            }
        },
        "temperature": 0
    }))
}

pub fn parse_openai_decision_response(
    value: &serde_json::Value,
) -> Result<JudgeDecisionOutputData, RouterTypedErrorKind> {
    let text = extract_openai_output_text(value).ok_or_else(|| {
        RouterTypedErrorKind::JudgeConfigurationFailed {
            message: "OpenAI judge response did not include output text".to_string(),
        }
    })?;
    let wire: OpenAiDecisionWireData =
        serde_json::from_str(text).map_err(RouterTypedErrorKind::from)?;
    Ok(JudgeDecisionOutputData {
        selected_tool_id: wire.selected_tool_id,
        confidence: wire.confidence.clamp(0.0, 1.0),
        reason: wire.reason,
        decision: wire.decision,
        needs_more_metadata: wire.needs_more_metadata,
    })
}

pub async fn judge_candidate_tools_with_openai(
    request: &JudgeCandidateRequestData,
    config: &OpenAiJudgeConfigData,
) -> Result<JudgeDecisionOutputData, RouterTypedErrorKind> {
    if config.api_key.trim().is_empty() {
        return Err(RouterTypedErrorKind::JudgeConfigurationFailed {
            message: "OpenAI judge API key is required".to_string(),
        });
    }
    if config.model.trim().is_empty() {
        return Err(RouterTypedErrorKind::JudgeConfigurationFailed {
            message: "OpenAI judge model is required".to_string(),
        });
    }
    let endpoint = if config.endpoint.trim().is_empty() {
        "https://api.openai.com/v1/responses"
    } else {
        config.endpoint.trim()
    };
    let payload = create_openai_responses_payload(request, config.model.trim())?;
    let response = reqwest::Client::new()
        .post(endpoint)
        .bearer_auth(config.api_key.trim())
        .json(&payload)
        .send()
        .await
        .map_err(|error| RouterTypedErrorKind::JudgeConfigurationFailed {
            message: format!("OpenAI judge request failed: {error}"),
        })?;
    let status = response.status();
    let response_text =
        response
            .text()
            .await
            .map_err(|error| RouterTypedErrorKind::JudgeConfigurationFailed {
                message: format!("OpenAI judge response read failed: {error}"),
            })?;
    if !status.is_success() {
        return Err(RouterTypedErrorKind::JudgeConfigurationFailed {
            message: format!("OpenAI judge request returned status {status}: {response_text}"),
        });
    }
    let response_json: serde_json::Value =
        serde_json::from_str(&response_text).map_err(RouterTypedErrorKind::from)?;
    parse_openai_decision_response(&response_json)
}

fn validate_judge_candidate_limit(
    request: &JudgeCandidateRequestData,
) -> Result<(), RouterTypedErrorKind> {
    if request.candidates.len() > 5 {
        return Err(RouterTypedErrorKind::JudgeConfigurationFailed {
            message: "judge payload cannot contain more than five candidates".to_string(),
        });
    }
    Ok(())
}

fn extract_openai_output_text(value: &serde_json::Value) -> Option<&str> {
    value
        .get("output_text")
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            value
                .get("output")
                .and_then(serde_json::Value::as_array)?
                .iter()
                .flat_map(|item| item.get("content").and_then(serde_json::Value::as_array))
                .flat_map(|content| content.iter())
                .find_map(|content| {
                    content
                        .get("text")
                        .and_then(serde_json::Value::as_str)
                        .or_else(|| {
                            content
                                .get("output_text")
                                .and_then(serde_json::Value::as_str)
                        })
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

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
    fn judge_abstains_for_ambiguous_write() {
        let mut candidate = create_candidate_card_data("chat.read_message", 1);
        candidate.risk = "ambiguous_write".to_string();
        let request = JudgeCandidateRequestData {
            query: "send a message to the incident channel".to_string(),
            recent_context: None,
            candidates: vec![candidate],
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

    #[test]
    fn openai_payload_contains_only_compact_candidate_cards() {
        let request = JudgeCandidateRequestData {
            query: "search calendar availability".to_string(),
            recent_context: Some("previous user asked for Tuesday".to_string()),
            candidates: vec![create_candidate_card_data("calendar.search", 1)],
        };
        let payload = create_openai_responses_payload(&request, "gpt-4.1-mini")
            .expect("payload should build");
        let payload_text = serde_json::to_string(&payload).expect("payload should serialize");

        assert!(payload_text.contains("gpt-4.1-mini"));
        assert!(payload_text.contains("calendar.search"));
        assert!(payload_text.contains("search calendar availability"));
        assert!(!payload_text.contains("required_tool_ids"));
        assert!(!payload_text.contains("graded_relevance"));
        assert!(!payload_text.contains("source_expected_tools"));
        assert!(!payload_text.contains("failure_modes"));
    }

    #[test]
    fn openai_payload_rejects_more_than_five_candidates() {
        let request = JudgeCandidateRequestData {
            query: "search".to_string(),
            recent_context: None,
            candidates: (0..6)
                .map(|index| create_candidate_card_data(&format!("tool.{index}"), index + 1))
                .collect(),
        };
        let error = create_openai_responses_payload(&request, "gpt-4.1-mini")
            .expect_err("payload should reject too many candidates");

        assert!(error
            .to_string()
            .contains("cannot contain more than five candidates"));
    }

    #[test]
    fn openai_response_parses_decision() {
        let response = serde_json::json!({
            "output_text": "{\"decision\":\"select_tool\",\"selected_tool_id\":\"calendar.search\",\"confidence\":0.83,\"reason\":\"calendar search matches availability\",\"needs_more_metadata\":false}"
        });
        let decision = parse_openai_decision_response(&response).expect("decision should parse");

        assert_eq!(decision.decision, JudgeDecisionKindData::SelectTool);
        assert_eq!(
            decision.selected_tool_id.as_deref(),
            Some("calendar.search")
        );
        assert_eq!(decision.confidence, 0.83);
        assert!(!decision.needs_more_metadata);
    }

    #[tokio::test]
    async fn openai_adapter_posts_authorized_payload() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let endpoint = format!(
            "http://{}",
            listener.local_addr().expect("addr should exist")
        );
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("request should arrive");
            let mut request_text = String::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let read = stream.read(&mut buffer).expect("read should succeed");
                if read == 0 {
                    break;
                }
                request_text.push_str(&String::from_utf8_lossy(&buffer[..read]));
                if request_text.contains("\r\n\r\n") && request_text.contains("calendar.search") {
                    break;
                }
            }
            let body = serde_json::json!({
                "output_text": "{\"decision\":\"select_tool\",\"selected_tool_id\":\"calendar.search\",\"confidence\":0.91,\"reason\":\"calendar search is the safest match\",\"needs_more_metadata\":false}"
            })
            .to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .expect("response should write");
            request_text
        });
        let request = JudgeCandidateRequestData {
            query: "search calendar availability".to_string(),
            recent_context: None,
            candidates: vec![create_candidate_card_data("calendar.search", 1)],
        };
        let config = OpenAiJudgeConfigData {
            api_key: "sk-local-test".to_string(),
            model: "gpt-4.1-mini".to_string(),
            endpoint,
        };
        let decision = judge_candidate_tools_with_openai(&request, &config)
            .await
            .expect("local adapter call should succeed");
        let request_text = server.join().expect("server should finish");

        assert_eq!(
            decision.selected_tool_id.as_deref(),
            Some("calendar.search")
        );
        assert_eq!(decision.confidence, 0.91);
        assert!(request_text
            .to_ascii_lowercase()
            .contains("authorization: bearer sk-local-test"));
        assert!(request_text.contains("gpt-4.1-mini"));
        assert!(request_text.contains("calendar.search"));
        assert!(!request_text.contains("required_tool_ids"));
    }
}
