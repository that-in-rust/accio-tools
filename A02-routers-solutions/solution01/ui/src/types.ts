export type InvokeFunction = <T = unknown>(
  command: string,
  args?: Record<string, unknown>,
) => Promise<T>;

export type DownloadTextFunction = (
  filename: string,
  content: string,
) => string | void;

export type RouterModeNameData = "lexical" | "schema_aware" | "hybrid";

export interface RouterAppReadinessData {
  judge_key_ready: boolean;
  route_preview_enabled: boolean;
  judged_route_enabled: boolean;
  model_label: string;
  readiness_message: string;
}

export interface ToolCatalogRecordData {
  id: string;
  source_tool_id?: string | null;
  server_id?: string | null;
  server_name?: string | null;
  name: string;
  description: string;
  input_schema: unknown;
  tags?: string[];
  source?: unknown;
  metadata?: unknown;
}

export interface GradedRelevanceItemData {
  tool_id: string;
  relevance: number;
}

export interface RouteQueryInputData {
  id: string;
  query: string;
  required_tool_ids: string[];
  should_route: boolean;
  graded_relevance: GradedRelevanceItemData[];
  source_expected_tools: string[];
  failure_modes: string[];
}

export interface EvaluationPackFileData {
  filename: string;
  content: string;
}

export interface CandidateEvidenceCardData {
  rank: number;
  score: number;
  tool_id: string;
  matched_terms: string[];
  matched_fields: string[];
  capability_match: string[];
  risk: string;
  why_matched: string;
  signal_contributions: Record<string, number>;
}

export interface JudgeDecisionOutputData {
  decision: string;
  selected_tool_id?: string | null;
  confidence: number;
  reason: string;
}

export interface RouteToolsRequestData {
  dataset_path?: string | null;
  catalog_tools?: ToolCatalogRecordData[] | null;
  query: string;
  recent_context?: string | null;
  router_mode: RouterModeNameData;
  api_key?: string | null;
}

export interface RouteToolsResponseData {
  route_label: string;
  candidates: CandidateEvidenceCardData[];
  judge_decision?: JudgeDecisionOutputData | null;
}

export interface RoutingMetricsRequestData {
  dataset_path?: string | null;
  catalog_tools?: ToolCatalogRecordData[] | null;
  query_records?: RouteQueryInputData[] | null;
  router_mode: RouterModeNameData;
  max_k: number;
  threshold: number;
}

export interface MetricReportOutputData {
  queries: number;
  route_required_queries: number;
  abstention_queries: number;
  recall_at_k: Record<string, number>;
  mrr: number;
  ndcg_at_10: number;
  abstention_accuracy: number;
  average_selected_candidate_count: number;
  token_reduction_estimate: number;
  router_mode: RouterModeNameData;
}

export interface CatalogStatsSummaryData {
  tool_count: number;
  query_count: number;
  source_count: number;
  schema_count: number;
  route_required_count: number;
  abstention_count: number;
}

export interface BenchmarkGoldMatchData {
  query_id: string;
  should_route: boolean;
  required_tool_ids: string[];
  selected_tool_id?: string | null;
  gold_match_status: string;
  failure_bucket: string;
}

export interface RouteEvidencePayloadData {
  route_request: RouteToolsRequestData;
  route_response: RouteToolsResponseData;
  catalog_stats: CatalogStatsSummaryData;
  benchmark_gold_match?: BenchmarkGoldMatchData | null;
  metrics_report?: MetricReportOutputData | null;
}
