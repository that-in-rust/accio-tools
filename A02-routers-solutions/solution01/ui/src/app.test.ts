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

  it("shows selected-router MVP surface without benchmark lab controls", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();

    expect(readScreenTextContent()).toContain("Tool Router Evidence Console");
    expect(readScreenTextContent()).toContain("Evaluate Inquiry");
    expect(getSelectedOptionText("benchmark-query-select")).toContain(
      "Find nearby calendar availability for a follow-up visit 0",
    );
    expect(getSelectedOptionText("benchmark-query-select")).not.toContain(
      "query-00 -> calendar.search_availability",
    );
    expect(readScreenTextContent()).toContain("Expected tool");
    expect(readScreenTextContent()).toContain("Search availability");
    expect(readScreenTextContent()).toContain("Lexical BM25");
    expect(readScreenTextContent()).toContain("Schema-aware BM25");
    expect(readScreenTextContent()).toContain("Hybrid RRF");
    expect(getButtonByLabelText("Run Selected Route Decision").disabled).toBe(true);
    expect(readScreenTextContent()).not.toContain("Run CPU Preview");
    expect(readScreenTextContent()).not.toContain("Run Judged Route");
    expect(readScreenTextContent()).not.toContain("Run Benchmark Eval");
    expect(readScreenTextContent()).not.toContain("Compare All Modes");
    expect(readScreenTextContent()).not.toContain("Download Evaluation Pack");
    expect(readScreenTextContent()).not.toContain("Export Judged Route Evidence");
    expect(readScreenTextContent()).not.toContain("Export Preview Route Evidence");
    expect(readScreenTextContent()).not.toContain("Export Logs");
    expect(readScreenTextContent()).not.toContain("Custom catalog JSON");
    expect(readScreenTextContent()).not.toContain("Custom query JSON");
    expect(readScreenTextContent()).not.toContain("Benchmark Health");
    expect(readScreenTextContent()).not.toContain("Query-Level Router Comparison");
  });

  it("shows judge model and route flow explainer", async () => {
    renderRouterWorkbenchApp(createRouterInvokeMock());
    await flushAsyncViewUpdates();

    const screenText = readScreenTextContent();
    expect(screenText).toContain("Judge model: mock-router-judge");
    expect(screenText).toContain("Selected CPU router");
    expect(screenText).toContain("Lexical BM25");
    expect(screenText).toContain("Top five shortlist");
    expect(screenText).toContain("LLM judge chooses top 1 or abstains");
    expect(screenText).toContain("Compare against benchmark gold");
    expect(screenText).toContain("Benchmark source: curated local subset");
    expect(screenText).toContain("VOTR");
    expect(screenText).toContain("contextweaver");
    expect(screenText).toContain("graph-tool-call");
    expect(screenText).toContain("mcpproxy-go");
    expect(screenText).toContain("LiveMCPBench");
    expect(screenText).toContain("mcp-bench");
    expect(screenText).toContain(
      "Pack: 947 tools, 50 queries, 46 route-required labels, 4 abstention labels.",
    );
  });

  it("updates visible judge model after key validation", async () => {
    const invoke = createRouterInvokeMock({
      readiness: createReadyStateData("gpt-4.1-mini-router"),
    });

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();
    expect(readScreenTextContent()).toContain("Judge model: mock-router-judge");
    const keyInput = getInputByLabelText("OpenAI API key");
    keyInput.value = "sk-router-test";
    keyInput.dispatchEvent(new Event("input", { bubbles: true }));
    await flushAsyncViewUpdates();
    getButtonByLabelText("Validate Key").click();
    await flushAsyncViewUpdates();

    expect(readScreenTextContent()).toContain("Judge model: gpt-4.1-mini-router");
  });

  it("binds route flow explainer to the selected CPU router", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();
    getButtonByLabelText("Hybrid RRF").click();
    await flushAsyncViewUpdates();

    expect(getFlowStageTextByTestId("router-flow-stage-cpu")).toContain("Hybrid RRF");
    expect(getCommandCallsByName(invoke, "compare_routing_modes_metrics")).toHaveLength(0);
  });

  it("switches benchmark comparison copy for custom query", async () => {
    renderRouterWorkbenchApp(createRouterInvokeMock());
    await flushAsyncViewUpdates();
    getButtonByLabelText("Custom Query").click();
    await flushAsyncViewUpdates();

    expect(getFlowStageTextByTestId("router-flow-stage-benchmark")).toContain(
      "No benchmark label",
    );
  });

  it("requires judge readiness before final route decision", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();
    vi.mocked(invoke).mockClear();

    expect(getButtonByLabelText("Run Selected Route Decision").disabled).toBe(true);
    expect(readScreenTextContent()).toContain(
      "Validate a judge key to run the final route decision.",
    );
    getButtonByLabelText("Run Selected Route Decision").click();
    await flushAsyncViewUpdates();

    expect(getCommandCallsByName(invoke, "run_cpu_preview_only")).toHaveLength(0);
    expect(getCommandCallsByName(invoke, "route_tools_for_query")).toHaveLength(0);
  });

  it("runs one selected CPU router through judged route", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();
    const keyInput = getInputByLabelText("OpenAI API key");
    keyInput.value = "sk-router-test";
    keyInput.dispatchEvent(new Event("input", { bubbles: true }));
    await flushAsyncViewUpdates();
    getButtonByLabelText("Validate Key").click();
    await flushAsyncViewUpdates();
    vi.mocked(invoke).mockClear();
    getButtonByLabelText("Hybrid RRF").click();
    await flushAsyncViewUpdates();
    getButtonByLabelText("Run Selected Route Decision").click();
    await flushAsyncViewUpdates();

    const routeCalls = getCommandCallsByName(invoke, "route_tools_for_query");
    expect(routeCalls).toHaveLength(1);
    expect(routeCalls[0]?.[1]).toEqual({
      request: expect.objectContaining({
        query: "Find nearby calendar availability for a follow-up visit 0",
        router_mode: "hybrid",
        api_key: "sk-router-test",
      }),
    });
    expect(getCommandCallsByName(invoke, "run_cpu_preview_only")).toHaveLength(0);
    expect(readScreenTextContent()).toContain("Route Decision");
    expect(readScreenTextContent()).toContain("Hybrid RRF");
    expect(readScreenTextContent()).toContain("Judge rescued");
    expect(readScreenTextContent()).toContain("rank 2");
    expect(readScreenTextContent()).toContain("Search availability");
    expect(document.querySelectorAll('[data-testid="candidate-card"]')).toHaveLength(5);
    expect(getProgressStatusByLabel("Judge review")).toBe("complete");
  });

  it("filters bundled benchmark queries before selected judged route", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();
    const searchInput = getInputByLabelText("Search benchmark queries");
    searchInput.value = "visit 14";
    searchInput.dispatchEvent(new Event("input", { bubbles: true }));
    await flushAsyncViewUpdates();
    const keyInput = getInputByLabelText("OpenAI API key");
    keyInput.value = "sk-router-test";
    keyInput.dispatchEvent(new Event("input", { bubbles: true }));
    await flushAsyncViewUpdates();
    getButtonByLabelText("Validate Key").click();
    await flushAsyncViewUpdates();
    vi.mocked(invoke).mockClear();
    getButtonByLabelText("Run Selected Route Decision").click();
    await flushAsyncViewUpdates();

    expect(readScreenTextContent()).toContain(
      "Find nearby calendar availability for a follow-up visit 14 - Search availability (query-14)",
    );
    expect(invoke).toHaveBeenCalledWith("route_tools_for_query", {
      request: expect.objectContaining({
        query: "Find nearby calendar availability for a follow-up visit 14",
      }),
    });
  });

  it("routes a custom free-text query with no benchmark label", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();
    const keyInput = getInputByLabelText("OpenAI API key");
    keyInput.value = "sk-router-test";
    keyInput.dispatchEvent(new Event("input", { bubbles: true }));
    await flushAsyncViewUpdates();
    getButtonByLabelText("Validate Key").click();
    await flushAsyncViewUpdates();
    getButtonByLabelText("Custom Query").click();
    await flushAsyncViewUpdates();
    const customInput = getTextAreaByLabelText("Custom inquiry");
    customInput.value = "Send a Slack message to the incident channel";
    customInput.dispatchEvent(new Event("input", { bubbles: true }));
    await flushAsyncViewUpdates();
    getButtonByLabelText("Schema-aware BM25").click();
    await flushAsyncViewUpdates();
    vi.mocked(invoke).mockClear();
    getButtonByLabelText("Run Selected Route Decision").click();
    await flushAsyncViewUpdates();

    expect(invoke).toHaveBeenCalledWith("route_tools_for_query", {
      request: expect.objectContaining({
        query: "Send a Slack message to the incident channel",
        router_mode: "schema_aware",
      }),
    });
    expect(readScreenTextContent()).toContain("No benchmark label");
  });

  it("ignores unsupported router mode values before selected route", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();
    const keyInput = getInputByLabelText("OpenAI API key");
    keyInput.value = "sk-router-test";
    keyInput.dispatchEvent(new Event("input", { bubbles: true }));
    await flushAsyncViewUpdates();
    getButtonByLabelText("Validate Key").click();
    await flushAsyncViewUpdates();
    const hybridButton = getButtonByLabelText("Hybrid RRF");
    hybridButton.dataset.routerMode = "graph-path";
    hybridButton.click();
    await flushAsyncViewUpdates();
    vi.mocked(invoke).mockClear();
    getButtonByLabelText("Run Selected Route Decision").click();
    await flushAsyncViewUpdates();

    expect(invoke).toHaveBeenCalledWith("route_tools_for_query", {
      request: expect.objectContaining({
        router_mode: "lexical",
      }),
    });
    expect(readScreenTextContent()).toContain("Ignored unsupported router mode value.");
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

interface RouterInvokeMockOptionsData {
  readiness?: RouterAppReadinessData;
}

function createRouterInvokeMock(
  options: RouterInvokeMockOptionsData = {},
): InvokeFunction {
  return vi.fn(async (command: string, args?: Record<string, unknown>) => {
    if (command === "download_evaluation_pack_files") {
      return createPackFilesData();
    }
    if (command === "validate_judge_api_key") {
      return options.readiness ?? createReadyStateData();
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

function createReadyStateData(
  modelLabel = "mock-router-judge",
): RouterAppReadinessData {
  return {
    judge_key_ready: true,
    route_preview_enabled: true,
    judged_route_enabled: true,
    model_label: modelLabel,
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

function getFlowStageTextByTestId(testId: string): string {
  const element = document.querySelector(`[data-testid="${testId}"]`);
  if (!element) throw new Error(`Missing flow stage ${testId}`);
  return element.textContent ?? "";
}

function getTextAreaByLabelText(label: string): HTMLTextAreaElement {
  const element = Array.from(document.querySelectorAll("label")).find(
    (candidate) => candidate.textContent?.includes(label),
  );
  const textarea = element?.querySelector("textarea");
  if (!textarea) throw new Error(`Missing textarea ${label}`);
  return textarea;
}

function getCommandCallsByName(invoke: InvokeFunction, commandName: string) {
  return vi.mocked(invoke).mock.calls.filter(([command]) => command === commandName);
}
