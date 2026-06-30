export type InvokeFunction = <T = unknown>(
  command: string,
  args?: Record<string, unknown>,
) => Promise<T>;

export type DownloadTextFunction = (filename: string, content: string) => string | void;

export type ReadFileTextFunction = (file: File) => Promise<string>;

export interface AppReadiness {
  api_key_ready: boolean;
  storage_ready: boolean;
  analyze_enabled: boolean;
  model_label: string;
  readiness_message: string;
}

export interface Finding {
  finding_id: string;
  title: string;
  severity: "High" | "Medium" | "Low";
  prompt_evidence: string;
  impact: string;
  suggested_fix: string;
  fix_mode: "AutoFixable" | "HumanReviewOnly" | "Backlog";
  verification_scenario: string;
}

export interface FindingGroup {
  section: "Structure" | "ToolGaps" | "WorkflowOrder" | "SafetyPhi" | "Verification";
  findings: Finding[];
}

export interface AnalysisResult {
  prompt_version_name: string;
  model_label: string;
  finding_groups: FindingGroup[];
  normalized_prompt: unknown;
  action_log: string[];
}

export interface ApplySelectedFixesResponse {
  result: {
    candidate: {
      updated_prompt_json: string;
      applied_finding_ids: string[];
      patch_summary: string;
    };
    model_label: string;
  };
  updated_version_name: string;
  action_log: string[];
}

export interface ReverifyFindingStatus {
  finding_id: string;
  status: "Fixed" | "StillFailing" | "Unknown";
  status_label: string;
  rationale: string;
}

export interface ReverifyPromptResult {
  prompt_version_name: string;
  model_label: string;
  finding_statuses: ReverifyFindingStatus[];
  action_log: string[];
}

export interface VerificationExport {
  markdown_report: string;
  updated_prompt_json: string;
  deterministic_checks: unknown[];
  semantic_checks: unknown[];
}
