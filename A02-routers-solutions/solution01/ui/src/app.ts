import type {
  CandidateEvidenceCardData,
  DownloadTextFunction,
  EvaluationPackFileData,
  InvokeFunction,
  MetricReportOutputData,
  RouteEvidencePayloadData,
  RouteQueryInputData,
  RouteToolsRequestData,
  RouteToolsResponseData,
  RoutingMetricsRequestData,
  RouterAppReadinessData,
  RouterModeNameData,
  ToolCatalogRecordData,
} from "./types";

export type { InvokeFunction };

interface RouterWorkbenchOptions {
  downloadText?: DownloadTextFunction;
}

type QuerySourceModeData = "benchmark" | "custom";
type UploadedSourceModeData = "bundled" | "custom";
type AsyncActionStatusData = "idle" | "running" | "complete" | "failed";
type RouteProgressFlowData =
  | "idle"
  | "preview_running"
  | "preview_complete"
  | "preview_failed"
  | "judged_running"
  | "judged_complete"
  | "judged_failed";
type RouteProgressStageStatusData =
  | "waiting"
  | "running"
  | "complete"
  | "skipped"
  | "failed";

interface RouteProgressStageData {
  label: string;
  status: RouteProgressStageStatusData;
}

interface RouterWorkbenchStateData {
  apiKey: string;
  readiness: RouterAppReadinessData;
  keyStatus: AsyncActionStatusData;
  packStatus: AsyncActionStatusData;
  previewStatus: AsyncActionStatusData;
  judgedStatus: AsyncActionStatusData;
  metricsStatus: AsyncActionStatusData;
  modeComparisonStatusState: AsyncActionStatusData;
  routerMode: RouterModeNameData;
  querySource: QuerySourceModeData;
  catalogSourceModeName: UploadedSourceModeData;
  queryPackSourceModeName: UploadedSourceModeData;
  selectedQueryId: string;
  querySearchTextValue: string;
  customQuery: string;
  recentContext: string;
  routeProgressStagesList: RouteProgressStageData[];
  tools: ToolCatalogRecordData[];
  queries: RouteQueryInputData[];
  lastRouteRequest: RouteToolsRequestData | null;
  routeResponse: RouteToolsResponseData | null;
  metricsReport: MetricReportOutputData | null;
  modeComparisonReportsList: MetricReportOutputData[];
  uploadStatusMessage: string;
  exportMessage: string;
  errorMessage: string;
  activityLog: string[];
}

const routerModeLabelsData: Record<RouterModeNameData, string> = {
  lexical: "Lexical BM25",
  schema_aware: "Schema-aware BM25",
  hybrid: "Hybrid RRF",
};

const defaultReadinessData: RouterAppReadinessData = {
  judge_key_ready: false,
  route_preview_enabled: true,
  judged_route_enabled: false,
  model_label: "mock-router-judge",
  readiness_message: "CPU preview is available; judged route needs a key.",
};

export function createRouterWorkbenchApp(
  root: HTMLElement,
  invoke: InvokeFunction,
  options: RouterWorkbenchOptions = {},
) {
  const state = createInitialStateData();
  const downloadText = options.downloadText ?? downloadTextInBrowser;

  const render = () => {
    root.innerHTML = renderRouterWorkbenchView(state);
    bindWorkbenchEventHandlers(root, state, invoke, downloadText, render);
  };

  render();
  void loadEvaluationPackFiles(state, invoke, render);
}

function createInitialStateData(): RouterWorkbenchStateData {
  return {
    apiKey: "",
    readiness: defaultReadinessData,
    keyStatus: "idle",
    packStatus: "idle",
    previewStatus: "idle",
    judgedStatus: "idle",
    metricsStatus: "idle",
    modeComparisonStatusState: "idle",
    routerMode: "lexical",
    querySource: "benchmark",
    catalogSourceModeName: "bundled",
    queryPackSourceModeName: "bundled",
    selectedQueryId: "",
    querySearchTextValue: "",
    customQuery: "",
    recentContext: "",
    routeProgressStagesList: createRouteProgressStagesList("idle"),
    tools: [],
    queries: [],
    lastRouteRequest: null,
    routeResponse: null,
    metricsReport: null,
    modeComparisonReportsList: [],
    uploadStatusMessage: "",
    exportMessage: "",
    errorMessage: "",
    activityLog: ["Opened router workbench."],
  };
}

async function loadEvaluationPackFiles(
  state: RouterWorkbenchStateData,
  invoke: InvokeFunction,
  render: () => void,
) {
  try {
    state.packStatus = "running";
    pushActivityLogEntry(state, "Loading bundled evaluation pack.");
    render();
    const files = await invoke<EvaluationPackFileData[]>(
      "download_evaluation_pack_files",
      { datasetPath: null },
    );
    const toolsFile = files.find((file) => file.filename === "tools.json");
    const queriesFile = files.find((file) => file.filename === "queries.json");
    if (!toolsFile || !queriesFile) {
      throw new Error("Evaluation pack did not include tools.json and queries.json.");
    }
    state.tools = JSON.parse(toolsFile.content) as ToolCatalogRecordData[];
    state.queries = JSON.parse(queriesFile.content) as RouteQueryInputData[];
    state.catalogSourceModeName = "bundled";
    state.queryPackSourceModeName = "bundled";
    state.selectedQueryId = state.queries[0]?.id ?? "";
    state.querySearchTextValue = "";
    state.packStatus = "complete";
    pushActivityLogEntry(
      state,
      `Loaded ${state.tools.length} tools and ${state.queries.length} queries.`,
    );
  } catch (error) {
    state.packStatus = "failed";
    state.errorMessage = errorToMessageText(error);
  }
  render();
}

function bindWorkbenchEventHandlers(
  root: HTMLElement,
  state: RouterWorkbenchStateData,
  invoke: InvokeFunction,
  downloadText: DownloadTextFunction,
  render: () => void,
) {
  queryElementById<HTMLInputElement>(root, "judge-key-input")?.addEventListener(
    "input",
    (event) => {
      state.apiKey = (event.target as HTMLInputElement).value;
      state.keyStatus = "idle";
      state.readiness = {
        ...state.readiness,
        judge_key_ready: false,
        judged_route_enabled: false,
        readiness_message: "CPU preview is available; judged route needs a key.",
      };
      render();
    },
  );

  queryElementById<HTMLButtonElement>(root, "validate-key-button")?.addEventListener(
    "click",
    () => {
      void validateJudgeSessionKey(state, invoke, render);
    },
  );

  root.querySelectorAll<HTMLButtonElement>("[data-query-source]").forEach((button) => {
    button.addEventListener("click", () => {
      const source = button.dataset.querySource as QuerySourceModeData;
      selectQuerySourceOption(state, source);
      render();
    });
  });

  queryElementById<HTMLSelectElement>(root, "benchmark-query-select")?.addEventListener(
    "change",
    (event) => {
      state.selectedQueryId = (event.target as HTMLSelectElement).value;
      render();
    },
  );

  queryElementById<HTMLInputElement>(root, "benchmark-query-search")?.addEventListener(
    "input",
    (event) => {
      state.querySearchTextValue = (event.target as HTMLInputElement).value;
      const filteredQueries = getFilteredQueriesList(state);
      state.selectedQueryId = filteredQueries.some(
        (query) => query.id === state.selectedQueryId,
      )
        ? state.selectedQueryId
        : filteredQueries[0]?.id ?? "";
      render();
    },
  );

  queryElementById<HTMLInputElement>(root, "custom-catalog-input")?.addEventListener(
    "change",
    (event) => {
      void loadCustomCatalogFile(event.target as HTMLInputElement, state, render);
    },
  );

  queryElementById<HTMLInputElement>(root, "custom-query-file-input")?.addEventListener(
    "change",
    (event) => {
      void loadCustomQueryFile(event.target as HTMLInputElement, state, render);
    },
  );

  queryElementById<HTMLTextAreaElement>(root, "custom-query-input")?.addEventListener(
    "input",
    (event) => {
      state.customQuery = (event.target as HTMLTextAreaElement).value;
      render();
    },
  );

  queryElementById<HTMLTextAreaElement>(root, "recent-context-input")?.addEventListener(
    "input",
    (event) => {
      state.recentContext = (event.target as HTMLTextAreaElement).value;
    },
  );

  root.querySelectorAll<HTMLButtonElement>("[data-router-mode]").forEach((button) => {
    button.addEventListener("click", () => {
      state.routerMode = button.dataset.routerMode as RouterModeNameData;
      render();
    });
  });

  queryElementById<HTMLButtonElement>(root, "cpu-preview-button")?.addEventListener(
    "click",
    () => {
      void runCpuPreviewOnly(state, invoke, render);
    },
  );

  queryElementById<HTMLButtonElement>(root, "judged-route-button")?.addEventListener(
    "click",
    () => {
      void routeToolsForQuery(state, invoke, render);
    },
  );

  queryElementById<HTMLButtonElement>(root, "metrics-run-button")?.addEventListener(
    "click",
    () => {
      void evaluateRoutingSubsetMetrics(state, invoke, render);
    },
  );

  queryElementById<HTMLButtonElement>(root, "comparison-run-button")?.addEventListener(
    "click",
    () => {
      void compareRoutingModesMetrics(state, invoke, render);
    },
  );

  queryElementById<HTMLButtonElement>(root, "download-pack-button")?.addEventListener(
    "click",
    () => {
      void downloadEvaluationPackFiles(state, invoke, downloadText, render);
    },
  );

  queryElementById<HTMLButtonElement>(root, "export-evidence-button")?.addEventListener(
    "click",
    () => {
      void exportRouteEvidenceReport(state, invoke, downloadText, render);
    },
  );

  queryElementById<HTMLButtonElement>(root, "export-logs-button")?.addEventListener(
    "click",
    () => {
      void exportDiagnosticLogsText(state, invoke, downloadText, render);
    },
  );
}

async function loadCustomCatalogFile(
  input: HTMLInputElement,
  state: RouterWorkbenchStateData,
  render: () => void,
) {
  const file = input.files?.[0];
  if (!file) return;
  try {
    state.errorMessage = "";
    const parsed = await parseUploadedJsonFile(file);
    state.tools = normalizeToolCatalogRecords(parsed);
    state.catalogSourceModeName = "custom";
    state.uploadStatusMessage = `Loaded custom catalog with ${state.tools.length} tools.`;
    pushActivityLogEntry(state, state.uploadStatusMessage);
  } catch (error) {
    state.errorMessage = errorToMessageText(error);
    pushActivityLogEntry(state, "Custom catalog upload failed.");
  }
  render();
}

async function loadCustomQueryFile(
  input: HTMLInputElement,
  state: RouterWorkbenchStateData,
  render: () => void,
) {
  const file = input.files?.[0];
  if (!file) return;
  try {
    state.errorMessage = "";
    state.queries = normalizeRouteQueryRecords(await parseUploadedJsonFile(file));
    state.selectedQueryId = state.queries[0]?.id ?? "";
    state.querySearchTextValue = "";
    state.querySource = "benchmark";
    state.queryPackSourceModeName = "custom";
    state.uploadStatusMessage = `Loaded custom query file with ${state.queries.length} queries.`;
    pushActivityLogEntry(state, state.uploadStatusMessage);
  } catch (error) {
    state.errorMessage = errorToMessageText(error);
    pushActivityLogEntry(state, "Custom query upload failed.");
  }
  render();
}

async function validateJudgeSessionKey(
  state: RouterWorkbenchStateData,
  invoke: InvokeFunction,
  render: () => void,
) {
  if (state.keyStatus === "running") return;
  try {
    state.errorMessage = "";
    state.keyStatus = "running";
    pushActivityLogEntry(state, "validate_judge_api_key validating judge key.");
    render();
    state.readiness = await invoke<RouterAppReadinessData>(
      "validate_judge_api_key",
      { apiKey: state.apiKey || null },
    );
    state.keyStatus = state.readiness.judge_key_ready ? "complete" : "failed";
    pushActivityLogEntry(state, state.readiness.readiness_message);
  } catch (error) {
    state.keyStatus = "failed";
    state.errorMessage = errorToMessageText(error);
  }
  render();
}

function selectQuerySourceOption(
  state: RouterWorkbenchStateData,
  source: QuerySourceModeData,
) {
  state.querySource = source;
  state.routeResponse = null;
  state.lastRouteRequest = null;
  state.errorMessage = "";
  pushActivityLogEntry(
    state,
    source === "benchmark" ? "Using benchmark query." : "Using custom query.",
  );
}

async function runCpuPreviewOnly(
  state: RouterWorkbenchStateData,
  invoke: InvokeFunction,
  render: () => void,
) {
  if (!canRunPreviewNow(state) || state.previewStatus === "running") return;
  try {
    state.errorMessage = "";
    state.previewStatus = "running";
    state.routeResponse = null;
    state.lastRouteRequest = null;
    state.routeProgressStagesList =
      createRouteProgressStagesList("preview_running");
    pushActivityLogEntry(
      state,
      `run_cpu_preview_only running ${routerModeLabelsData[state.routerMode]} preview.`,
    );
    render();
    const routeRequest = createRouteRequestData(state, false);
    state.routeResponse = await invoke<RouteToolsResponseData>(
      "run_cpu_preview_only",
      { request: routeRequest },
    );
    state.lastRouteRequest = {
      ...routeRequest,
      api_key: null,
      catalog_tools: null,
    };
    state.previewStatus = "complete";
    state.routeProgressStagesList =
      createRouteProgressStagesList("preview_complete");
    pushActivityLogEntry(
      state,
      "run_cpu_preview_only CPU preview returned top five candidates.",
    );
  } catch (error) {
    state.previewStatus = "failed";
    state.routeProgressStagesList =
      createRouteProgressStagesList("preview_failed");
    state.errorMessage = errorToMessageText(error);
  }
  render();
}

async function routeToolsForQuery(
  state: RouterWorkbenchStateData,
  invoke: InvokeFunction,
  render: () => void,
) {
  if (!canRunJudgedRoute(state) || state.judgedStatus === "running") return;
  try {
    state.errorMessage = "";
    state.judgedStatus = "running";
    state.routeResponse = null;
    state.lastRouteRequest = null;
    state.routeProgressStagesList =
      createRouteProgressStagesList("judged_running");
    pushActivityLogEntry(state, "route_tools_for_query running judged route.");
    render();
    const routeRequest = createRouteRequestData(state, true);
    state.routeResponse = await invoke<RouteToolsResponseData>(
      "route_tools_for_query",
      { request: routeRequest },
    );
    state.lastRouteRequest = {
      ...routeRequest,
      api_key: null,
      catalog_tools: null,
    };
    state.judgedStatus = "complete";
    state.routeProgressStagesList =
      createRouteProgressStagesList("judged_complete");
    pushActivityLogEntry(state, "route_tools_for_query selected top route result.");
  } catch (error) {
    state.judgedStatus = "failed";
    state.routeProgressStagesList =
      createRouteProgressStagesList("judged_failed");
    state.errorMessage = errorToMessageText(error);
  }
  render();
}

async function evaluateRoutingSubsetMetrics(
  state: RouterWorkbenchStateData,
  invoke: InvokeFunction,
  render: () => void,
) {
  if (state.metricsStatus === "running") return;
  try {
    state.errorMessage = "";
    state.metricsStatus = "running";
    pushActivityLogEntry(
      state,
      `evaluate_routing_subset_metrics evaluating ${routerModeLabelsData[state.routerMode]}.`,
    );
    render();
    state.metricsReport = await invoke<MetricReportOutputData>(
      "evaluate_routing_subset_metrics",
      {
        request: createMetricsRequestData(state),
      },
    );
    state.metricsStatus = "complete";
    pushActivityLogEntry(state, "evaluate_routing_subset_metrics completed.");
  } catch (error) {
    state.metricsStatus = "failed";
    state.errorMessage = errorToMessageText(error);
  }
  render();
}

async function compareRoutingModesMetrics(
  state: RouterWorkbenchStateData,
  invoke: InvokeFunction,
  render: () => void,
) {
  if (state.modeComparisonStatusState === "running") return;
  try {
    state.errorMessage = "";
    state.modeComparisonStatusState = "running";
    pushActivityLogEntry(state, "compare_routing_modes_metrics comparing all router modes.");
    render();
    state.modeComparisonReportsList = await invoke<MetricReportOutputData[]>(
      "compare_routing_modes_metrics",
      {
        request: createMetricsRequestData(state),
      },
    );
    state.modeComparisonStatusState = "complete";
    pushActivityLogEntry(state, "compare_routing_modes_metrics completed.");
  } catch (error) {
    state.modeComparisonStatusState = "failed";
    state.errorMessage = errorToMessageText(error);
  }
  render();
}

async function downloadEvaluationPackFiles(
  state: RouterWorkbenchStateData,
  invoke: InvokeFunction,
  downloadText: DownloadTextFunction,
  render: () => void,
) {
  try {
    state.errorMessage = "";
    const files = await invoke<EvaluationPackFileData[]>(
      "download_evaluation_pack_files",
      { datasetPath: null },
    );
    files.forEach((file) => downloadText(file.filename, file.content));
    state.exportMessage = `download_evaluation_pack_files downloaded ${files.length} benchmark fixture files.`;
    pushActivityLogEntry(state, state.exportMessage);
  } catch (error) {
    state.errorMessage = errorToMessageText(error);
  }
  render();
}

async function exportRouteEvidenceReport(
  state: RouterWorkbenchStateData,
  invoke: InvokeFunction,
  downloadText: DownloadTextFunction,
  render: () => void,
) {
  if (!state.routeResponse) return;
  try {
    state.errorMessage = "";
    const content = await invoke<string>("export_route_evidence_report", {
      payload: createRouteEvidencePayload(state),
    });
    downloadText("tool-router-evidence-report.md", content);
    state.exportMessage = "export_route_evidence_report downloaded route evidence report.";
    pushActivityLogEntry(state, state.exportMessage);
  } catch (error) {
    state.errorMessage = errorToMessageText(error);
  }
  render();
}

async function exportDiagnosticLogsText(
  state: RouterWorkbenchStateData,
  invoke: InvokeFunction,
  downloadText: DownloadTextFunction,
  render: () => void,
) {
  try {
    state.errorMessage = "";
    const content = await invoke<string>("export_diagnostic_logs_text");
    downloadText("tool-router-diagnostic-log.txt", content);
    state.exportMessage = "export_diagnostic_logs_text downloaded diagnostic log.";
    pushActivityLogEntry(state, state.exportMessage);
  } catch (error) {
    state.errorMessage = errorToMessageText(error);
  }
  render();
}

function createRouteRequestData(
  state: RouterWorkbenchStateData,
  includeKey: boolean,
): RouteToolsRequestData {
  return {
    dataset_path: null,
    catalog_tools: state.catalogSourceModeName === "custom" ? state.tools : null,
    query: getActiveQueryTextValue(state),
    recent_context: state.recentContext.trim() || null,
    router_mode: state.routerMode,
    api_key: includeKey ? state.apiKey.trim() || null : null,
  };
}

function createMetricsRequestData(
  state: RouterWorkbenchStateData,
): RoutingMetricsRequestData {
  return {
    dataset_path: null,
    catalog_tools: state.catalogSourceModeName === "custom" ? state.tools : null,
    query_records: state.queryPackSourceModeName === "custom" ? state.queries : null,
    router_mode: state.routerMode,
    max_k: 10,
    threshold: 2,
  };
}

function createRouteEvidencePayload(
  state: RouterWorkbenchStateData,
): RouteEvidencePayloadData {
  return {
    route_request:
      state.lastRouteRequest ?? {
        ...createRouteRequestData(state, false),
        api_key: null,
        catalog_tools: null,
      },
    route_response: state.routeResponse as RouteToolsResponseData,
    catalog_stats: createCatalogStatsSummary(state),
    benchmark_gold_match: createBenchmarkGoldMatch(state),
    metrics_report: state.metricsReport,
  };
}

function createCatalogStatsSummary(state: RouterWorkbenchStateData) {
  const sourceValues = new Set(
    state.tools
      .map((tool) => tool.server_name ?? tool.server_id ?? tool.source_tool_id ?? "")
      .filter(Boolean),
  );
  return {
    tool_count: state.tools.length,
    query_count: state.queries.length,
    source_count: sourceValues.size,
    schema_count: state.tools.filter((tool) => Boolean(tool.input_schema)).length,
    route_required_count: state.queries.filter((query) => query.should_route).length,
    abstention_count: state.queries.filter((query) => !query.should_route).length,
  };
}

function createDuplicateToolStatusText(tools: ToolCatalogRecordData[]): string {
  const seenToolIds = new Set<string>();
  const duplicateToolIds = new Set<string>();
  tools.forEach((tool) => {
    if (seenToolIds.has(tool.id)) {
      duplicateToolIds.add(tool.id);
    }
    seenToolIds.add(tool.id);
  });
  return duplicateToolIds.size === 0
    ? "unique ids"
    : `${duplicateToolIds.size} duplicate ids`;
}

function createBenchmarkGoldMatch(state: RouterWorkbenchStateData) {
  const query = getSelectedQueryRecordData(state);
  if (state.querySource !== "benchmark" || !query || !state.routeResponse) {
    return null;
  }
  const decision = state.routeResponse.judge_decision;
  const selectedToolId = decision?.selected_tool_id ?? null;
  const cpuRequiredToolSurvived = state.routeResponse.candidates
    .slice(0, 5)
    .some((candidate) => query.required_tool_ids.includes(candidate.tool_id));
  const selectedRequiredTool = selectedToolId
    ? query.required_tool_ids.includes(selectedToolId)
    : false;
  const goldMatchStatus = !decision
    ? "unjudged_cpu_preview"
    : query.should_route && !cpuRequiredToolSurvived
      ? "missing_required_tool"
      : query.should_route && selectedRequiredTool
        ? "matched_required_tool"
        : query.should_route
          ? "wrong_llm_top1"
          : decision.decision === "abstain" || !selectedToolId
            ? "correct_abstain"
            : "abstention_miss";
  return {
    query_id: query.id,
    should_route: query.should_route,
    required_tool_ids: query.required_tool_ids,
    selected_tool_id: selectedToolId,
    gold_match_status: goldMatchStatus,
    failure_bucket:
      goldMatchStatus === "matched_required_tool" || goldMatchStatus === "correct_abstain"
        ? "none"
        : goldMatchStatus,
  };
}

function getActiveQueryTextValue(state: RouterWorkbenchStateData): string {
  if (state.querySource === "custom") {
    return state.customQuery.trim();
  }
  return getSelectedQueryRecordData(state)?.query ?? "";
}

function getSelectedQueryRecordData(
  state: RouterWorkbenchStateData,
): RouteQueryInputData | undefined {
  return state.queries.find((query) => query.id === state.selectedQueryId);
}

function getFilteredQueriesList(
  state: RouterWorkbenchStateData,
): RouteQueryInputData[] {
  const searchText = state.querySearchTextValue.trim().toLowerCase();
  const queryList = searchText
    ? state.queries.filter((query) =>
        [
          query.id,
          query.query,
          ...query.required_tool_ids,
          ...query.source_expected_tools,
          ...query.failure_modes,
        ].some((value) => value.toLowerCase().includes(searchText)),
      )
    : state.queries;
  return queryList.slice(0, 50);
}

function createRouteProgressStagesList(
  flow: RouteProgressFlowData,
): RouteProgressStageData[] {
  const previewFlow = flow.startsWith("preview");
  const failedFlow = flow.endsWith("failed");
  const completeFlow = flow.endsWith("complete");
  const runningFlow = flow.endsWith("running");
  const waitingStatus = "waiting" satisfies RouteProgressStageStatusData;
  const judgeStatus = previewFlow
    ? "skipped"
    : failedFlow
      ? "failed"
      : completeFlow
        ? "complete"
        : runningFlow
          ? "running"
          : waitingStatus;
  return [
    {
      label: "Catalog validation",
      status: failedFlow ? "failed" : runningFlow || completeFlow ? "complete" : waitingStatus,
    },
    {
      label: "CPU ranking",
      status: failedFlow
        ? "failed"
        : runningFlow
          ? "running"
          : completeFlow
            ? "complete"
            : waitingStatus,
    },
    {
      label: "Judge review",
      status: judgeStatus,
    },
    {
      label: "Evidence compilation",
      status: failedFlow
        ? "failed"
        : completeFlow
          ? "complete"
          : waitingStatus,
    },
  ];
}

function canRunPreviewNow(state: RouterWorkbenchStateData): boolean {
  return Boolean(getActiveQueryTextValue(state)) && state.packStatus === "complete";
}

function canRunJudgedRoute(state: RouterWorkbenchStateData): boolean {
  return canRunPreviewNow(state) && state.readiness.judged_route_enabled;
}

async function parseUploadedJsonFile(file: File): Promise<unknown> {
  try {
    return JSON.parse(await readUploadedFileText(file)) as unknown;
  } catch (error) {
    throw new Error(`Invalid JSON upload: ${errorToMessageText(error)}`);
  }
}

function readUploadedFileText(file: File): Promise<string> {
  if ("text" in file && typeof file.text === "function") {
    return file.text();
  }
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.addEventListener("load", () => resolve(String(reader.result ?? "")));
    reader.addEventListener("error", () => reject(reader.error ?? new Error("file read failed")));
    reader.readAsText(file);
  });
}

function normalizeToolCatalogRecords(value: unknown): ToolCatalogRecordData[] {
  if (!Array.isArray(value)) {
    throw new Error("Custom catalog JSON must be an array of tools.");
  }
  const seenIds = new Set<string>();
  return value.map((item, index) => {
    const record = asObjectRecordValue(item, `tool ${index + 1}`);
    const id = requireStringFieldValue(record, "id", `tool ${index + 1}`);
    const name = requireStringFieldValue(record, "name", id);
    const description = requireStringFieldValue(record, "description", id);
    const inputSchema = record.input_schema ?? {};
    if (!isObjectRecordValue(inputSchema)) {
      throw new Error(`Tool ${id} must include an object input_schema.`);
    }
    if (seenIds.has(id)) {
      throw new Error(`Duplicate tool id ${id}.`);
    }
    seenIds.add(id);
    return {
      ...record,
      id,
      name,
      description,
      input_schema: inputSchema,
      tags: normalizeStringListValue(record.tags),
    } as ToolCatalogRecordData;
  });
}

function normalizeRouteQueryRecords(value: unknown): RouteQueryInputData[] {
  if (!Array.isArray(value)) {
    throw new Error("Custom query JSON must be an array of query records.");
  }
  return value.map((item, index) => {
    const record = asObjectRecordValue(item, `query ${index + 1}`);
    const id = requireStringFieldValue(record, "id", `query ${index + 1}`);
    const query = requireStringFieldValue(record, "query", id);
    const requiredToolIds = normalizeStringListValue(record.required_tool_ids);
    const shouldRoute = record.should_route === true;
    if (shouldRoute && requiredToolIds.length === 0) {
      throw new Error(`Query ${id} should route but has no required_tool_ids.`);
    }
    return {
      ...record,
      id,
      query,
      required_tool_ids: requiredToolIds,
      should_route: shouldRoute,
      graded_relevance: normalizeGradedRelevanceValue(record.graded_relevance),
      source_expected_tools: normalizeStringListValue(record.source_expected_tools),
      failure_modes: normalizeStringListValue(record.failure_modes),
    } as RouteQueryInputData;
  });
}

function normalizeGradedRelevanceValue(value: unknown) {
  if (!Array.isArray(value)) {
    return [];
  }
  return value.map((item, index) => {
    const record = asObjectRecordValue(item, `graded relevance ${index + 1}`);
    return {
      tool_id: requireStringFieldValue(record, "tool_id", `graded relevance ${index + 1}`),
      relevance: typeof record.relevance === "number" ? record.relevance : 0,
    };
  });
}

function normalizeStringListValue(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value.filter((item): item is string => typeof item === "string");
}

function asObjectRecordValue(
  value: unknown,
  label: string,
): Record<string, unknown> {
  if (!isObjectRecordValue(value)) {
    throw new Error(`${label} must be an object.`);
  }
  return value;
}

function isObjectRecordValue(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function requireStringFieldValue(
  record: Record<string, unknown>,
  field: string,
  label: string,
): string {
  const value = record[field];
  if (typeof value !== "string" || !value.trim()) {
    throw new Error(`${label} is missing ${field}.`);
  }
  return value.trim();
}

function renderRouterWorkbenchView(state: RouterWorkbenchStateData): string {
  return `
    <main class="router-shell">
      <section class="router-main">
        <header class="workspace-header workspace-header--router">
          <div>
            <p class="eyebrow">Tool Routing MVP</p>
            <h1>Tool Router Evidence Console</h1>
            <p class="subhead">Evaluate Inquiry</p>
            <p class="hero-copy">CPU routers shortlist top five tools. A cheap judge reviews that evidence and returns one route or abstains.</p>
          </div>
          <dl class="workspace-meta">
            <div><dt>Tools</dt><dd>${state.tools.length || "..."}</dd></div>
            <div><dt>Queries</dt><dd>${state.queries.length || "..."}</dd></div>
            <div><dt>Mode</dt><dd>${routerModeLabelsData[state.routerMode]}</dd></div>
            <div><dt>Judge</dt><dd>${state.readiness.judge_key_ready ? "Ready" : "Preview only"}</dd></div>
          </dl>
        </header>
        ${renderJudgeKeyCard(state)}
        ${renderEvaluationPackCard(state)}
        ${renderQuerySourcePanel(state)}
        ${renderRouterModePanel(state)}
        ${renderRouteActionPanel(state)}
        ${renderRouteResultPanel(state)}
        ${renderBenchmarkHealthPanel(state)}
        ${state.errorMessage ? `<section class="error" role="alert">${escapeHtmlText(state.errorMessage)}</section>` : ""}
      </section>
      ${renderActivityLogPanel(state)}
    </main>
  `;
}

function renderJudgeKeyCard(state: RouterWorkbenchStateData): string {
  const keyLabel =
    state.keyStatus === "running"
      ? "Validating"
      : state.readiness.judge_key_ready
        ? "Validate Again"
        : "Validate Key";
  return `
    <section class="session-card">
      <div class="section-title-row">
        <div>
          <h2>Judge Session</h2>
          <p>CPU preview stays available without a key. Judged route unlocks after validation.</p>
        </div>
        <span class="status-pill">${escapeHtmlText(state.readiness.model_label)}</span>
      </div>
      <div class="session-controls">
        <label class="field-stack" for="judge-key-input">
          <span>OpenAI API key</span>
          <input id="judge-key-input" type="password" value="${escapeAttributeText(state.apiKey)}" placeholder="sk-..." autocomplete="off" />
        </label>
        <button id="validate-key-button" ${state.keyStatus === "running" || !state.apiKey.trim() ? "disabled" : ""}>${keyLabel}</button>
      </div>
      <p class="privacy-note">${escapeHtmlText(state.readiness.readiness_message)}</p>
    </section>
  `;
}

function renderEvaluationPackCard(state: RouterWorkbenchStateData): string {
  const catalogStats = createCatalogStatsSummary(state);
  return `
    <section class="pack-panel">
      <div class="section-title-row">
        <div>
          <h2>Training Benchmark</h2>
          <p>Bundled subset with labeled tool ids, abstentions, relevance grades, and failure modes.</p>
        </div>
        <button id="download-pack-button" class="secondary-action" ${state.packStatus !== "complete" ? "disabled" : ""}>Download Evaluation Pack</button>
      </div>
      <div class="metric-grid">
        <div><strong>${catalogStats.tool_count}</strong><span>tools</span></div>
        <div><strong>${catalogStats.query_count}</strong><span>queries</span></div>
        <div><strong>${catalogStats.source_count}</strong><span>sources</span></div>
        <div><strong>${catalogStats.schema_count}</strong><span>schemas</span></div>
        <div><strong>${catalogStats.route_required_count}</strong><span>route labels</span></div>
        <div><strong>${catalogStats.abstention_count}</strong><span>abstains</span></div>
        <div><strong>${escapeHtmlText(createDuplicateToolStatusText(state.tools))}</strong><span>duplicate-id status</span></div>
      </div>
      <div class="upload-grid">
        <label class="field-stack" for="custom-catalog-input">
          <span>Custom catalog JSON</span>
          <input id="custom-catalog-input" type="file" accept="application/json,.json" />
        </label>
        <label class="field-stack" for="custom-query-file-input">
          <span>Custom query JSON</span>
          <input id="custom-query-file-input" type="file" accept="application/json,.json" />
        </label>
      </div>
      <p class="query-summary">Catalog: ${state.catalogSourceModeName}. Queries: ${state.queryPackSourceModeName}.${state.uploadStatusMessage ? ` ${escapeHtmlText(state.uploadStatusMessage)}` : ""}</p>
    </section>
  `;
}

function renderQuerySourcePanel(state: RouterWorkbenchStateData): string {
  const selected = getSelectedQueryRecordData(state);
  const filteredQueries = getFilteredQueriesList(state);
  const options = filteredQueries
    .map((query) => {
      const expected = query.required_tool_ids[0] ?? "abstain";
      return `<option value="${escapeAttributeText(query.id)}" ${query.id === state.selectedQueryId ? "selected" : ""}>${escapeHtmlText(`${query.id} -> ${expected}`)}</option>`;
    })
    .join("") || `<option value="">No matching benchmark query</option>`;
  return `
    <section class="query-panel">
      <div class="section-title-row">
        <div>
          <h2>Inquiry Input</h2>
          <p>Start from a labeled benchmark query or type your own live inquiry.</p>
        </div>
        <div class="segmented-control" aria-label="Query source">
          <button data-query-source="benchmark" class="${state.querySource === "benchmark" ? "is-selected" : ""}">Benchmark Query</button>
          <button data-query-source="custom" class="${state.querySource === "custom" ? "is-selected" : ""}">Custom Query</button>
        </div>
      </div>
      ${
        state.querySource === "benchmark"
          ? `
            <label class="field-stack" for="benchmark-query-search">
              <span>Search benchmark queries</span>
              <input id="benchmark-query-search" type="search" value="${escapeAttributeText(state.querySearchTextValue)}" placeholder="Search id, query, expected tool, or failure mode" />
            </label>
            <label class="field-stack" for="benchmark-query-select">
              <span>Benchmark query</span>
              <select id="benchmark-query-select" ${filteredQueries.length === 0 ? "disabled" : ""}>${options}</select>
            </label>
            <p class="query-summary">Showing ${filteredQueries.length} of ${state.queries.length} benchmark queries.</p>
            <p class="query-summary">${renderQuerySummaryText(selected)}</p>
          `
          : `
            <label class="field-stack" for="custom-query-input">
              <span>Custom inquiry</span>
              <textarea id="custom-query-input" rows="4" placeholder="Describe the user request that needs a tool route.">${escapeHtmlText(state.customQuery)}</textarea>
            </label>
          `
      }
      <label class="field-stack" for="recent-context-input">
        <span>Recent conversation context</span>
        <textarea id="recent-context-input" rows="3" placeholder="Optional prior turns or constraints.">${escapeHtmlText(state.recentContext)}</textarea>
      </label>
    </section>
  `;
}

function renderRouterModePanel(state: RouterWorkbenchStateData): string {
  return `
    <section class="mode-panel">
      <div class="section-title-row">
        <div>
          <h2>CPU Router</h2>
          <p>Pick the deterministic shortlist strategy before judge review.</p>
        </div>
      </div>
      <div class="mode-grid">
        ${Object.entries(routerModeLabelsData)
          .map(
            ([mode, label]) => `
              <button data-router-mode="${mode}" class="mode-button ${state.routerMode === mode ? "is-selected" : ""}">
                <strong>${label}</strong>
                <span>${mode === "lexical" ? "Fast keyword baseline" : mode === "schema_aware" ? "Adds schema signals" : "Fuses lexical and schema ranks"}</span>
              </button>
            `,
          )
          .join("")}
      </div>
    </section>
  `;
}

function renderRouteActionPanel(state: RouterWorkbenchStateData): string {
  return `
    <section class="action-panel">
      <button id="cpu-preview-button" ${!canRunPreviewNow(state) || state.previewStatus === "running" ? "disabled" : ""}>${state.previewStatus === "running" ? "Running Preview" : "Run CPU Preview"}</button>
      <button id="judged-route-button" ${!canRunJudgedRoute(state) || state.judgedStatus === "running" ? "disabled" : ""}>${state.judgedStatus === "running" ? "Running Judge" : "Run Judged Route"}</button>
      <button id="metrics-run-button" class="secondary-action" ${state.metricsStatus === "running" || state.packStatus !== "complete" ? "disabled" : ""}>${state.metricsStatus === "running" ? "Evaluating" : "Run Benchmark Eval"}</button>
      <button id="comparison-run-button" class="secondary-action" ${state.modeComparisonStatusState === "running" || state.packStatus !== "complete" ? "disabled" : ""}>${state.modeComparisonStatusState === "running" ? "Comparing Modes" : "Compare All Modes"}</button>
      <button id="export-evidence-button" class="secondary-action" ${!state.routeResponse ? "disabled" : ""}>Export Evidence</button>
      <button id="export-logs-button" class="secondary-action">Export Logs</button>
      ${state.exportMessage ? `<p class="export-message">${escapeHtmlText(state.exportMessage)}</p>` : ""}
      ${renderRouteProgressStagesList(state.routeProgressStagesList)}
    </section>
  `;
}

function renderRouteProgressStagesList(
  stages: RouteProgressStageData[],
): string {
  return `
    <ol class="route-progress-strip" aria-label="Route progress stages">
      ${stages
        .map(
          (stage) => `
            <li class="route-progress-stage is-${stage.status}" data-progress-stage="${escapeAttributeText(stage.label)}" data-progress-status="${stage.status}">
              <span>${escapeHtmlText(stage.label)}</span>
              <strong>${escapeHtmlText(stage.status)}</strong>
            </li>
          `,
        )
        .join("")}
    </ol>
  `;
}

function renderRouteResultPanel(state: RouterWorkbenchStateData): string {
  if (!state.routeResponse) {
    return `
      <section class="result-panel">
        <h2>Route Result</h2>
        <p class="empty-state">Run a preview or judged route to inspect ranked evidence.</p>
      </section>
    `;
  }
  const decision = state.routeResponse.judge_decision;
  return `
    <section class="result-panel">
      <div class="section-title-row">
        <div>
          <h2>Route Result</h2>
          <p>${escapeHtmlText(state.routeResponse.route_label)}</p>
        </div>
        ${
          decision
            ? `<div class="decision-box"><strong>${escapeHtmlText(decision.selected_tool_id ?? "abstain")}</strong><span>${formatMetricValue(decision.confidence)} confidence</span></div>`
            : `<div class="decision-box"><strong>Top five</strong><span>CPU preview only</span></div>`
        }
      </div>
      ${decision ? `<p class="judge-reason">${escapeHtmlText(decision.reason)}</p>` : ""}
      ${renderCandidateEvidenceCards(state.routeResponse.candidates)}
    </section>
  `;
}

function renderCandidateEvidenceCards(
  candidates: CandidateEvidenceCardData[],
): string {
  return `
    <div class="candidate-list">
      ${candidates
        .map(
          (candidate) => `
            <article class="candidate-card" data-testid="candidate-card">
              <div class="candidate-rank">#${candidate.rank}</div>
              <div>
                <h3>${escapeHtmlText(candidate.tool_id)}</h3>
                <p>${escapeHtmlText(candidate.why_matched)}</p>
                <dl>
                  <div><dt>Score</dt><dd>${formatMetricValue(candidate.score)}</dd></div>
                  <div><dt>Fields</dt><dd>${escapeHtmlText(candidate.matched_fields.join(", ") || "none")}</dd></div>
                  <div><dt>Terms</dt><dd>${escapeHtmlText(candidate.matched_terms.slice(0, 8).join(", ") || "none")}</dd></div>
                  <div><dt>Risk</dt><dd>${escapeHtmlText(candidate.risk || "none")}</dd></div>
                </dl>
              </div>
            </article>
          `,
        )
        .join("")}
    </div>
  `;
}

function renderBenchmarkHealthPanel(state: RouterWorkbenchStateData): string {
  const report = state.metricsReport;
  const gold = createBenchmarkGoldMatch(state);
  return `
    <section class="metrics-panel">
      <div class="section-title-row">
        <div>
          <h2>Benchmark Health</h2>
          <p>Metrics are computed from the same bundled 50-query subset.</p>
        </div>
      </div>
      ${
        report
          ? `
            <div class="metric-grid">
              <div><strong>${formatMetricValue(report.recall_at_k["1"] ?? 0)}</strong><span>Recall@1</span></div>
              <div><strong>${formatMetricValue(report.recall_at_k["3"] ?? 0)}</strong><span>Recall@3</span></div>
              <div><strong>${formatMetricValue(report.recall_at_k["5"] ?? 0)}</strong><span>Recall@5</span></div>
              <div><strong>${formatMetricValue(report.recall_at_k["10"] ?? 0)}</strong><span>Recall@10</span></div>
              <div><strong>${formatMetricValue(report.mrr)}</strong><span>MRR</span></div>
              <div><strong>${formatMetricValue(report.ndcg_at_10)}</strong><span>nDCG@10</span></div>
              <div><strong>${formatMetricValue(report.abstention_accuracy)}</strong><span>Abstention</span></div>
              <div><strong>${formatMetricValue(report.judged_route_accuracy)}</strong><span>Judged route</span></div>
              <div><strong>${formatMetricValue(report.token_reduction_estimate)}</strong><span>Token reduction</span></div>
            </div>
          `
          : `<p class="empty-state">Run benchmark eval to compare the selected CPU router.</p>`
      }
      ${
        gold
          ? `
            <div class="metric-grid benchmark-gold-grid">
              <div><strong>${escapeHtmlText(gold.gold_match_status)}</strong><span>Gold status</span></div>
              <div><strong>${escapeHtmlText(gold.failure_bucket)}</strong><span>Failure bucket</span></div>
            </div>
          `
          : ""
      }
      ${renderModeComparisonTable(state.modeComparisonReportsList)}
    </section>
  `;
}

function renderModeComparisonTable(reports: MetricReportOutputData[]): string {
  if (reports.length === 0) {
    return "";
  }
  return `
    <div class="comparison-block">
      <h3>Mode Comparison</h3>
      <div class="comparison-table-wrap">
        <table class="comparison-table">
          <thead>
            <tr>
              <th>Mode</th>
              <th>Recall@5</th>
              <th>MRR</th>
              <th>nDCG@10</th>
              <th>Abstain</th>
              <th>Judged</th>
              <th>Token Cut</th>
            </tr>
          </thead>
          <tbody>
            ${reports
              .map(
                (report) => `
                  <tr>
                    <td>${routerModeLabelsData[report.router_mode]}</td>
                    <td>${formatMetricValue(report.recall_at_k["5"] ?? 0)}</td>
                    <td>${formatMetricValue(report.mrr)}</td>
                    <td>${formatMetricValue(report.ndcg_at_10)}</td>
                    <td>${formatMetricValue(report.abstention_accuracy)}</td>
                    <td>${formatMetricValue(report.judged_route_accuracy)}</td>
                    <td>${formatMetricValue(report.token_reduction_estimate)}</td>
                  </tr>
                `,
              )
              .join("")}
          </tbody>
        </table>
      </div>
    </div>
  `;
}

function renderActivityLogPanel(state: RouterWorkbenchStateData): string {
  return `
    <aside class="activity-log">
      <h2>Routing Trace</h2>
      <ol>
        ${state.activityLog
          .slice(-10)
          .map((entry) => `<li>${escapeHtmlText(entry)}</li>`)
          .join("")}
      </ol>
    </aside>
  `;
}

function renderQuerySummaryText(query: RouteQueryInputData | undefined): string {
  if (!query) return "No benchmark query selected.";
  const expected = query.required_tool_ids.join(", ") || "abstain";
  const failureModes = query.failure_modes.join(", ") || "none listed";
  return `Expected: ${expected}. Should route: ${query.should_route ? "yes" : "no"}. Failure modes: ${failureModes}.`;
}

function formatMetricValue(value: number): string {
  return Number.isFinite(value) ? value.toFixed(4) : "0.0000";
}

function pushActivityLogEntry(
  state: RouterWorkbenchStateData,
  message: string,
) {
  if (state.activityLog.at(-1) !== message) {
    state.activityLog.push(message);
  }
}

function queryElementById<T extends HTMLElement>(
  root: HTMLElement,
  id: string,
): T | null {
  return root.querySelector<T>(`#${id}`);
}

function errorToMessageText(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  if (error && typeof error === "object" && "message" in error) {
    return String((error as { message: unknown }).message);
  }
  return "Unknown router error.";
}

function escapeHtmlText(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function escapeAttributeText(value: string): string {
  return escapeHtmlText(value).replaceAll("'", "&#39;");
}

function downloadTextInBrowser(filename: string, content: string): string {
  const blob = new Blob([content], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
  return filename;
}
