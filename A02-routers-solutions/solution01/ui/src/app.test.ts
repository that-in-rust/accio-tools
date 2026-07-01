import { beforeEach, describe, expect, it, vi } from "vitest";
import { createRouterWorkbenchApp, type InvokeFunction } from "./app";
import type {
  CandidateEvidenceCardData,
  EvaluationPackFileData,
  MetricReportOutputData,
  RouteToolsRequestData,
  RouteToolsResponseData,
  RouterAppReadinessData,
} from "./types";

describe("router workbench journey", () => {
  beforeEach(() => {
    document.body.innerHTML = '<main id="app"></main>';
  });

  it("shows readable benchmark queries and expected tool summary", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();

    expect(getSelectedOptionText("benchmark-query-select")).toContain(
      "Find nearby calendar availability for a follow-up visit 0",
    );
    expect(getSelectedOptionText("benchmark-query-select")).not.toContain(
      "query-00 -> calendar.search_availability",
    );
    expect(readScreenTextContent()).toContain("Expected tool");
    expect(readScreenTextContent()).toContain("Search availability");
  });

  it("runs query-level comparison across all CPU modes", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();
    getButtonByLabelText("Run Routing Comparison").click();
    await flushAsyncViewUpdates();

    expect(invoke).toHaveBeenCalledWith("run_cpu_preview_only", {
      request: expect.objectContaining({ router_mode: "lexical" }),
    });
    expect(invoke).toHaveBeenCalledWith("run_cpu_preview_only", {
      request: expect.objectContaining({ router_mode: "schema_aware" }),
    });
    expect(invoke).toHaveBeenCalledWith("run_cpu_preview_only", {
      request: expect.objectContaining({ router_mode: "hybrid" }),
    });
    expect(readScreenTextContent()).toContain("Query-Level Router Comparison");
    expect(readScreenTextContent()).toContain("Lexical BM25");
    expect(readScreenTextContent()).toContain("Schema-aware BM25");
    expect(readScreenTextContent()).toContain("Hybrid RRF");
    expect(readScreenTextContent()).toContain("Expected in top five");
    expect(readScreenTextContent()).toContain("Judge not run");
  });

  it("keeps benchmark lab controls behind advanced evidence", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();

    const primaryActions = document.querySelector(".action-panel")?.textContent ?? "";
    expect(primaryActions).toContain("Run Routing Comparison");
    expect(primaryActions).not.toContain("Run Benchmark Eval");
    expect(primaryActions).not.toContain("Compare All Modes");
    expect(primaryActions).not.toContain("Export Logs");

    const advanced = document.querySelector("details[data-testid='advanced-evidence']");
    expect(advanced?.hasAttribute("open")).toBe(false);
    expect(advanced?.textContent).toContain("Run Benchmark Eval");
    expect(advanced?.textContent).toContain("Compare All Modes");
    expect(advanced?.textContent).toContain("Export Logs");
  });

  it("shows judge rescued verdict when judge chooses expected top-five tool", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();
    const keyInput = getInputByLabelText("OpenAI API key");
    keyInput.value = "sk-router-test";
    keyInput.dispatchEvent(new Event("input", { bubbles: true }));
    await flushAsyncViewUpdates();
    getButtonByLabelText("Validate Key").click();
    await flushAsyncViewUpdates();
    getButtonByLabelText("Run Routing Comparison").click();
    await flushAsyncViewUpdates();

    expect(invoke).toHaveBeenCalledWith("route_tools_for_query", {
      request: expect.objectContaining({ router_mode: "lexical" }),
    });
    expect(readScreenTextContent()).toContain("Judge rescued");
    expect(readScreenTextContent()).toContain("rank 2");
  });

  it("loads benchmark query picker and training pack counts", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();

    expect(readScreenTextContent()).toContain("Tool Router Evidence Console");
    expect(readScreenTextContent()).toContain("Evaluate Inquiry");
    expect(readScreenTextContent()).toContain("947");
    expect(readScreenTextContent()).toContain("50");
    expect(readScreenTextContent()).toContain("sources");
    expect(readScreenTextContent()).toContain("schemas");
    expect(readScreenTextContent()).toContain("unique ids");
    expect(readScreenTextContent()).toContain("Benchmark Query");
    expect(readScreenTextContent()).toContain("Custom Query");
    expect(readScreenTextContent()).toContain("Search benchmark queries");
    expect(getButtonByLabelText("Run CPU Preview").disabled).toBe(false);
    expect(getButtonByLabelText("Run Judged Route").disabled).toBe(true);
    expect(getButtonByLabelText("Export Judged Route Evidence").disabled).toBe(true);
    expect(getButtonByLabelText("Export Preview Route Evidence").disabled).toBe(true);
  });

  it("filters bundled benchmark queries before routing", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();
    const searchInput = getInputByLabelText("Search benchmark queries");
    searchInput.value = "visit 14";
    searchInput.dispatchEvent(new Event("input", { bubbles: true }));
    await flushAsyncViewUpdates();
    getButtonByLabelText("Run CPU Preview").click();
    await flushAsyncViewUpdates();

    expect(readScreenTextContent()).toContain(
      "Find nearby calendar availability for a follow-up visit 14 - Search availability (query-14)",
    );
    expect(invoke).toHaveBeenCalledWith("run_cpu_preview_only", {
      request: expect.objectContaining({
        query: "Find nearby calendar availability for a follow-up visit 14",
      }),
    });
  });

  it("runs CPU preview with selected benchmark query and router mode", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();
    getButtonByLabelText("Hybrid RRF").click();
    getButtonByLabelText("Run CPU Preview").click();
    await flushAsyncViewUpdates();

    expect(invoke).toHaveBeenCalledWith("run_cpu_preview_only", {
      request: expect.objectContaining({
        query: "Find nearby calendar availability for a follow-up visit 0",
        router_mode: "hybrid",
        api_key: null,
      }),
    });
    expect(readScreenTextContent()).toContain("cpu_only_debug_preview");
    expect(readScreenTextContent()).toContain("calendar.search_availability");
    expect(readScreenTextContent()).toContain("CPU preview returned top five candidates.");
    expect(readScreenTextContent()).toContain("run_cpu_preview_only");
    expect(document.querySelectorAll('[data-testid="candidate-card"]')).toHaveLength(5);
    expect(getProgressStatusByLabel("Judge review")).toBe("skipped");
    expect(getButtonByLabelText("Export Judged Route Evidence").disabled).toBe(true);
    expect(getButtonByLabelText("Export Preview Route Evidence").disabled).toBe(false);
  });

  it("ignores unsupported router mode values", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();
    const hybridButton = getButtonByLabelText("Hybrid RRF");
    hybridButton.dataset.routerMode = "graph-path";
    hybridButton.click();
    await flushAsyncViewUpdates();
    getButtonByLabelText("Run CPU Preview").click();
    await flushAsyncViewUpdates();

    expect(invoke).toHaveBeenCalledWith("run_cpu_preview_only", {
      request: expect.objectContaining({
        router_mode: "lexical",
      }),
    });
    expect(readScreenTextContent()).toContain("Ignored unsupported router mode value.");
  });

  it("validates judge key and runs judged route", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();
    const keyInput = getInputByLabelText("OpenAI API key");
    keyInput.value = "sk-router-test";
    keyInput.dispatchEvent(new Event("input", { bubbles: true }));
    await flushAsyncViewUpdates();
    getButtonByLabelText("Validate Key").click();
    await flushAsyncViewUpdates();
    getButtonByLabelText("Run Judged Route").click();
    await flushAsyncViewUpdates();

    expect(getButtonByLabelText("Validate Again").disabled).toBe(false);
    expect(invoke).toHaveBeenCalledWith("validate_judge_api_key", {
      apiKey: "sk-router-test",
    });
    expect(invoke).toHaveBeenCalledWith("route_tools_for_query", {
      request: expect.objectContaining({
        api_key: "sk-router-test",
        router_mode: "lexical",
      }),
    });
    expect(readScreenTextContent()).toContain("judged_route");
    expect(readScreenTextContent()).toContain("mock judge selected the strongest candidate");
    expect(getProgressStatusByLabel("Catalog validation")).toBe("complete");
    expect(getProgressStatusByLabel("CPU ranking")).toBe("complete");
    expect(getProgressStatusByLabel("Judge review")).toBe("complete");
    expect(getProgressStatusByLabel("Evidence compilation")).toBe("complete");
    expect(document.querySelectorAll('[data-testid="candidate-card"]')).toHaveLength(5);
    expect(getButtonByLabelText("Export Judged Route Evidence").disabled).toBe(false);
  });

  it("routes a custom query instead of benchmark text", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();
    getButtonByLabelText("Custom Query").click();
    await flushAsyncViewUpdates();
    const customInput = getTextAreaByLabelText("Custom inquiry");
    customInput.value = "Send a Slack message to the incident channel";
    customInput.dispatchEvent(new Event("input", { bubbles: true }));
    await flushAsyncViewUpdates();
    getButtonByLabelText("Run CPU Preview").click();
    await flushAsyncViewUpdates();

    expect(invoke).toHaveBeenCalledWith("run_cpu_preview_only", {
      request: expect.objectContaining({
        query: "Send a Slack message to the incident channel",
      }),
    });
  });

  it("routes with uploaded catalog and labeled query files", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();
    await uploadJsonFileByLabel("Custom catalog JSON", [
      {
        id: "custom.slack_post",
        name: "post_message",
        description: "Send a message to a channel",
        input_schema: {
          type: "object",
          properties: {
            channel: { type: "string" },
            message: { type: "string" },
          },
        },
        tags: ["message"],
      },
    ]);
    await flushAsyncViewUpdates();
    await uploadJsonFileByLabel("Custom query JSON", [
      {
        id: "custom-query-01",
        query: "Send a Slack message to the incident channel",
        required_tool_ids: ["custom.slack_post"],
        should_route: true,
        graded_relevance: [{ tool_id: "custom.slack_post", relevance: 3 }],
        source_expected_tools: ["custom.slack_post"],
        failure_modes: ["confuse chat read with chat write"],
      },
    ]);
    await flushAsyncViewUpdates();
    getButtonByLabelText("Run CPU Preview").click();
    await flushAsyncViewUpdates();
    getButtonByLabelText("Run Benchmark Eval").click();
    await flushAsyncViewUpdates();

    expect(readScreenTextContent()).toContain("custom-query-01");
    expect(invoke).toHaveBeenCalledWith("run_cpu_preview_only", {
      request: expect.objectContaining({
        query: "Send a Slack message to the incident channel",
        catalog_tools: [
          expect.objectContaining({
            id: "custom.slack_post",
            name: "post_message",
          }),
        ],
      }),
    });
    expect(invoke).toHaveBeenCalledWith("evaluate_routing_subset_metrics", {
      request: expect.objectContaining({
        catalog_tools: [
          expect.objectContaining({
            id: "custom.slack_post",
            name: "post_message",
          }),
        ],
        query_records: [
          expect.objectContaining({
            id: "custom-query-01",
            required_tool_ids: ["custom.slack_post"],
          }),
        ],
      }),
    });
  });

  it("runs metrics and downloads evidence artifacts", async () => {
    const downloads: Array<{ filename: string; content: string }> = [];
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke, (filename, content) => {
      downloads.push({ filename, content });
    });
    await flushAsyncViewUpdates();
    getButtonByLabelText("Run Benchmark Eval").click();
    await flushAsyncViewUpdates();
    getButtonByLabelText("Run CPU Preview").click();
    await flushAsyncViewUpdates();
    getButtonByLabelText("Export Preview Route Evidence").click();
    await flushAsyncViewUpdates();
    getButtonByLabelText("Export Logs").click();
    await flushAsyncViewUpdates();
    getButtonByLabelText("Download Evaluation Pack").click();
    await flushAsyncViewUpdates();

    expect(readScreenTextContent()).toContain("0.6493");
    expect(readScreenTextContent()).toContain("Recall@1");
    expect(readScreenTextContent()).toContain("Recall@3");
    expect(readScreenTextContent()).toContain("Recall@10");
    expect(readScreenTextContent()).toContain("Abstention");
    expect(readScreenTextContent()).toContain("Judged route");
    expect(readScreenTextContent()).toContain("Failure bucket");
    expect(readScreenTextContent()).toContain("Failure buckets");
    expect(readScreenTextContent()).toContain("wrong_llm_top1");
    expect(readScreenTextContent()).toContain("export_route_evidence_report");
    expect(invoke).toHaveBeenCalledWith("evaluate_routing_subset_metrics", {
      request: {
        dataset_path: null,
        catalog_tools: null,
        query_records: null,
        router_mode: "lexical",
        max_k: 10,
        threshold: 2,
      },
    });
    expect(invoke).toHaveBeenCalledWith("export_route_evidence_report", {
      payload: expect.objectContaining({
        route_request: expect.objectContaining({
          query: "Find nearby calendar availability for a follow-up visit 0",
          router_mode: "lexical",
          api_key: null,
        }),
        catalog_stats: expect.objectContaining({
          tool_count: 947,
          query_count: 50,
          route_required_count: 46,
          abstention_count: 4,
        }),
        benchmark_gold_match: expect.objectContaining({
          query_id: "query-00",
          selected_tool_id: null,
          gold_match_status: "unjudged_cpu_preview",
          failure_bucket: "unjudged_cpu_preview",
        }),
        metrics_report: expect.objectContaining({
          token_reduction_estimate: 0.9894,
        }),
      }),
    });
    expect(downloads).toEqual(
      expect.arrayContaining([
        {
          filename: "tool-router-evidence-report.md",
          content: "# Tool Router Evidence Report",
        },
        {
          filename: "tool-router-diagnostic-log.txt",
          content: "# Tool Router Diagnostic Log",
        },
        expect.objectContaining({ filename: "tools.json" }),
        expect.objectContaining({ filename: "queries.json" }),
        expect.objectContaining({ filename: "manifest.json" }),
      ]),
    );
  });

  it("compares all router modes in benchmark health panel", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();
    getButtonByLabelText("Compare All Modes").click();
    await flushAsyncViewUpdates();

    expect(invoke).toHaveBeenCalledWith("compare_routing_modes_metrics", {
      request: {
        dataset_path: null,
        catalog_tools: null,
        query_records: null,
        router_mode: "lexical",
        max_k: 10,
        threshold: 2,
      },
    });
    expect(readScreenTextContent()).toContain("Mode Comparison");
    expect(readScreenTextContent()).toContain("Lexical BM25");
    expect(readScreenTextContent()).toContain("Schema-aware BM25");
    expect(readScreenTextContent()).toContain("Hybrid RRF");
    expect(readScreenTextContent()).toContain("0.6493");
    expect(readScreenTextContent()).toContain("0.6275");
  });

  it("keeps router UI names and removes stale PIE copy", () => {
    renderRouterWorkbenchApp(createRouterInvokeMock());
    expect(document.querySelector(".router-shell")).not.toBeNull();
    expect(readScreenTextContent()).not.toContain("Assignment Prompt");
    expect(readScreenTextContent()).not.toContain("Analyze Prompt");
  });
});

function renderRouterWorkbenchApp(
  invoke: InvokeFunction,
  downloadText?: (filename: string, content: string) => void,
) {
  createRouterWorkbenchApp(document.querySelector("#app") as HTMLElement, invoke, {
    downloadText,
  });
}

function createRouterInvokeMock(): InvokeFunction {
  return vi.fn(async (command: string, args?: Record<string, unknown>) => {
    if (command === "download_evaluation_pack_files") {
      return createPackFilesData();
    }
    if (command === "validate_judge_api_key") {
      return createReadyStateData();
    }
    if (command === "run_cpu_preview_only") {
      return createRouteResponseData("cpu_only_debug_preview", false);
    }
    if (command === "route_tools_for_query") {
      const request = args?.request as RouteToolsRequestData;
      return createRouteResponseData(
        request.api_key ? "judged_route" : "cpu_only_debug_preview",
        true,
      );
    }
    if (command === "evaluate_routing_subset_metrics") {
      return createMetricsReportData();
    }
    if (command === "compare_routing_modes_metrics") {
      return createModeComparisonReports();
    }
    if (command === "export_route_evidence_report") {
      return "# Tool Router Evidence Report";
    }
    if (command === "export_diagnostic_logs_text") {
      return "# Tool Router Diagnostic Log";
    }
    throw new Error(command);
  }) as InvokeFunction;
}

function createPackFilesData(): EvaluationPackFileData[] {
  const tools = Array.from({ length: 947 }, (_, index) => ({
    id: index === 0 ? "calendar.search_availability" : `tool.${index}`,
    name: index === 0 ? "Search availability" : `Tool ${index}`,
    description:
      index === 0
        ? "Find calendar availability for appointments and follow-up visits."
        : `Reference tool ${index}`,
    input_schema: {},
    tags: index === 0 ? ["calendar", "availability"] : [],
  }));
  const queries = Array.from({ length: 50 }, (_, index) => ({
    id: `query-${index.toString().padStart(2, "0")}`,
    query: `Find nearby calendar availability for a follow-up visit ${index}`,
    required_tool_ids: index < 46 ? ["calendar.search_availability"] : [],
    should_route: index < 46,
    graded_relevance: [
      { tool_id: "calendar.search_availability", relevance: 3 },
    ],
    source_expected_tools: ["calendar.search_availability"],
    failure_modes: ["confuse search with booking"],
  }));
  return [
    { filename: "tools.json", content: JSON.stringify(tools) },
    { filename: "queries.json", content: JSON.stringify(queries) },
    { filename: "manifest.json", content: JSON.stringify({ version: "0.0.1" }) },
  ];
}

function createReadyStateData(): RouterAppReadinessData {
  return {
    judge_key_ready: true,
    route_preview_enabled: true,
    judged_route_enabled: true,
    model_label: "mock-router-judge",
    readiness_message: "Judge key accepted for local route execution.",
  };
}

function createRouteResponseData(
  routeLabel: string,
  judged: boolean,
): RouteToolsResponseData {
  return {
    route_label: routeLabel,
    candidates: Array.from({ length: 5 }, (_, index) =>
      createCandidateCardData(index, judged),
    ),
    judge_decision: judged
      ? {
          decision: "select_tool",
          selected_tool_id: "calendar.search_availability",
          confidence: 0.5,
          reason: "mock judge selected the strongest candidate",
        }
      : null,
  };
}

function createCandidateCardData(
  index = 0,
  judged = false,
): CandidateEvidenceCardData {
  const toolId = judged
    ? index === 0
      ? "calendar.nearby_booking"
      : index === 1
        ? "calendar.search_availability"
        : `calendar.candidate_${index}`
    : index === 0
      ? "calendar.search_availability"
      : `calendar.candidate_${index}`;
  return {
    rank: index + 1,
    score: 12.3456 - index,
    tool_id: toolId,
    matched_terms: ["calendar", "availability"],
    matched_fields: ["name", "description"],
    capability_match: ["calendar"],
    risk: index === 0 ? "low" : "medium",
    why_matched: "Matched calendar availability language.",
    signal_contributions: { lexical: 12.3456 },
  };
}

function createMetricsReportData(): MetricReportOutputData {
  return {
    queries: 50,
    route_required_queries: 46,
    abstention_queries: 4,
    recall_at_k: { "1": 0.5, "3": 0.6, "5": 0.6493, "10": 0.7 },
    mrr: 0.5223,
    ndcg_at_10: 0.5553,
    abstention_accuracy: 0,
    judged_route_accuracy: 0.5,
    failure_bucket_counts: {
      none: 19,
      wrong_llm_top1: 27,
      abstention_miss: 4,
    },
    average_selected_candidate_count: 5,
    token_reduction_estimate: 0.9894,
    router_mode: "lexical",
  };
}

function createModeComparisonReports(): MetricReportOutputData[] {
  return [
    createMetricsReportData(),
    {
      ...createMetricsReportData(),
      recall_at_k: { "1": 0.48, "3": 0.61, "5": 0.6275, "10": 0.69 },
      mrr: 0.5258,
      ndcg_at_10: 0.5589,
      router_mode: "schema_aware",
    },
    {
      ...createMetricsReportData(),
      recall_at_k: { "1": 0.47, "3": 0.6, "5": 0.6275, "10": 0.68 },
      mrr: 0.5146,
      ndcg_at_10: 0.5508,
      router_mode: "hybrid",
    },
  ];
}

async function flushAsyncViewUpdates() {
  for (let index = 0; index < 6; index += 1) {
    await Promise.resolve();
  }
}

function readScreenTextContent(): string {
  return document.body.textContent ?? "";
}

function getButtonByLabelText(label: string): HTMLButtonElement {
  const element = Array.from(document.querySelectorAll("button")).find(
    (button) => button.textContent?.includes(label),
  );
  if (!element) throw new Error(`Missing button ${label}`);
  return element as HTMLButtonElement;
}

function getInputByLabelText(label: string): HTMLInputElement {
  const element = Array.from(document.querySelectorAll("label")).find(
    (candidate) => candidate.textContent?.includes(label),
  );
  const input = element?.querySelector("input");
  if (!input) throw new Error(`Missing input ${label}`);
  return input;
}

function getSelectedOptionText(selectId: string): string {
  const select = document.querySelector<HTMLSelectElement>(`#${selectId}`);
  if (!select) throw new Error(`Missing select ${selectId}`);
  return select.selectedOptions[0]?.textContent ?? "";
}

function getProgressStatusByLabel(label: string): string | undefined {
  return document
    .querySelector(`[data-progress-stage="${label}"]`)
    ?.getAttribute("data-progress-status") ?? undefined;
}

async function uploadJsonFileByLabel(label: string, value: unknown) {
  const input = getInputByLabelText(label);
  const file = new File([JSON.stringify(value)], `${label}.json`, {
    type: "application/json",
  });
  Object.defineProperty(input, "files", {
    configurable: true,
    value: [file],
  });
  input.dispatchEvent(new Event("change", { bubbles: true }));
  for (let index = 0; index < 4; index += 1) {
    await new Promise((resolve) => setTimeout(resolve, 0));
  }
  await flushAsyncViewUpdates();
}

function getTextAreaByLabelText(label: string): HTMLTextAreaElement {
  const element = Array.from(document.querySelectorAll("label")).find(
    (candidate) => candidate.textContent?.includes(label),
  );
  const textarea = element?.querySelector("textarea");
  if (!textarea) throw new Error(`Missing textarea ${label}`);
  return textarea;
}
