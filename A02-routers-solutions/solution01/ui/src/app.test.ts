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

  it("loads benchmark query picker and training pack counts", async () => {
    const invoke = createRouterInvokeMock();

    renderRouterWorkbenchApp(invoke);
    await flushAsyncViewUpdates();

    expect(readScreenTextContent()).toContain("Evaluate Inquiry");
    expect(readScreenTextContent()).toContain("947");
    expect(readScreenTextContent()).toContain("50");
    expect(readScreenTextContent()).toContain("Benchmark Query");
    expect(readScreenTextContent()).toContain("Custom Query");
    expect(getButtonByLabelText("Run CPU Preview").disabled).toBe(false);
    expect(getButtonByLabelText("Run Judged Route").disabled).toBe(true);
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
    getButtonByLabelText("Export Evidence").click();
    await flushAsyncViewUpdates();
    getButtonByLabelText("Export Logs").click();
    await flushAsyncViewUpdates();
    getButtonByLabelText("Download Training Pack").click();
    await flushAsyncViewUpdates();

    expect(readScreenTextContent()).toContain("0.6493");
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
      return createRouteResponseData(request.api_key ? "judged_route" : "cpu_only_debug_preview", true);
    }
    if (command === "evaluate_routing_subset_metrics") {
      return createMetricsReportData();
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
    candidates: [createCandidateCardData()],
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

function createCandidateCardData(): CandidateEvidenceCardData {
  return {
    rank: 1,
    score: 12.3456,
    tool_id: "calendar.search_availability",
    matched_terms: ["calendar", "availability"],
    matched_fields: ["name", "description"],
    capability_match: ["calendar"],
    risk: "low",
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
    average_selected_candidate_count: 5,
    router_mode: "lexical",
  };
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

function getTextAreaByLabelText(label: string): HTMLTextAreaElement {
  const element = Array.from(document.querySelectorAll("label")).find(
    (candidate) => candidate.textContent?.includes(label),
  );
  const textarea = element?.querySelector("textarea");
  if (!textarea) throw new Error(`Missing textarea ${label}`);
  return textarea;
}
