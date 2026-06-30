import type {
  CandidateEvidenceCardData,
  DownloadTextFunction,
  EvaluationPackFileData,
  InvokeFunction,
  MetricReportOutputData,
  RouteQueryInputData,
  RouteToolsRequestData,
  RouteToolsResponseData,
  RouterAppReadinessData,
  RouterModeNameData,
  ToolCatalogRecordData,
} from "./types";

export type { InvokeFunction };

interface RouterWorkbenchOptions {
  downloadText?: DownloadTextFunction;
}

type QuerySourceModeData = "benchmark" | "custom";
type AsyncActionStatusData = "idle" | "running" | "complete" | "failed";

interface RouterWorkbenchStateData {
  apiKey: string;
  readiness: RouterAppReadinessData;
  keyStatus: AsyncActionStatusData;
  packStatus: AsyncActionStatusData;
  previewStatus: AsyncActionStatusData;
  judgedStatus: AsyncActionStatusData;
  metricsStatus: AsyncActionStatusData;
  routerMode: RouterModeNameData;
  querySource: QuerySourceModeData;
  selectedQueryId: string;
  customQuery: string;
  recentContext: string;
  tools: ToolCatalogRecordData[];
  queries: RouteQueryInputData[];
  routeResponse: RouteToolsResponseData | null;
  metricsReport: MetricReportOutputData | null;
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
    routerMode: "lexical",
    querySource: "benchmark",
    selectedQueryId: "",
    customQuery: "",
    recentContext: "",
    tools: [],
    queries: [],
    routeResponse: null,
    metricsReport: null,
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
    state.selectedQueryId = state.queries[0]?.id ?? "";
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

async function validateJudgeSessionKey(
  state: RouterWorkbenchStateData,
  invoke: InvokeFunction,
  render: () => void,
) {
  if (state.keyStatus === "running") return;
  try {
    state.errorMessage = "";
    state.keyStatus = "running";
    pushActivityLogEntry(state, "Validating judge key.");
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
    pushActivityLogEntry(state, `Running ${routerModeLabelsData[state.routerMode]} preview.`);
    render();
    state.routeResponse = await invoke<RouteToolsResponseData>(
      "run_cpu_preview_only",
      { request: createRouteRequestData(state, false) },
    );
    state.previewStatus = "complete";
    pushActivityLogEntry(state, "CPU preview returned top five candidates.");
  } catch (error) {
    state.previewStatus = "failed";
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
    pushActivityLogEntry(state, "Running judged route.");
    render();
    state.routeResponse = await invoke<RouteToolsResponseData>(
      "route_tools_for_query",
      { request: createRouteRequestData(state, true) },
    );
    state.judgedStatus = "complete";
    pushActivityLogEntry(state, "Judge selected top route result.");
  } catch (error) {
    state.judgedStatus = "failed";
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
    pushActivityLogEntry(state, `Evaluating ${routerModeLabelsData[state.routerMode]}.`);
    render();
    state.metricsReport = await invoke<MetricReportOutputData>(
      "evaluate_routing_subset_metrics",
      {
        request: {
          dataset_path: null,
          router_mode: state.routerMode,
          max_k: 10,
          threshold: 2,
        },
      },
    );
    state.metricsStatus = "complete";
    pushActivityLogEntry(state, "Benchmark metrics completed.");
  } catch (error) {
    state.metricsStatus = "failed";
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
    state.exportMessage = `Downloaded ${files.length} benchmark fixture files.`;
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
      response: state.routeResponse,
    });
    downloadText("tool-router-evidence-report.md", content);
    state.exportMessage = "Downloaded route evidence report.";
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
    state.exportMessage = "Downloaded diagnostic log.";
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
    query: getActiveQueryTextValue(state),
    recent_context: state.recentContext.trim() || null,
    router_mode: state.routerMode,
    api_key: includeKey ? state.apiKey.trim() || null : null,
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
  return state.queries.slice(0, 50);
}

function canRunPreviewNow(state: RouterWorkbenchStateData): boolean {
  return Boolean(getActiveQueryTextValue(state)) && state.packStatus === "complete";
}

function canRunJudgedRoute(state: RouterWorkbenchStateData): boolean {
  return canRunPreviewNow(state) && state.readiness.judged_route_enabled;
}

function renderRouterWorkbenchView(state: RouterWorkbenchStateData): string {
  return `
    <main class="router-shell">
      <section class="router-main">
        <header class="workspace-header workspace-header--router">
          <div>
            <p class="eyebrow">Tool Routing MVP</p>
            <h1>Evaluate Inquiry</h1>
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
  return `
    <section class="pack-panel">
      <div class="section-title-row">
        <div>
          <h2>Training Benchmark</h2>
          <p>Bundled subset with labeled tool ids, abstentions, relevance grades, and failure modes.</p>
        </div>
        <button id="download-pack-button" class="secondary-action" ${state.packStatus !== "complete" ? "disabled" : ""}>Download Training Pack</button>
      </div>
      <div class="metric-grid">
        <div><strong>${state.tools.length}</strong><span>tools</span></div>
        <div><strong>${state.queries.length}</strong><span>queries</span></div>
        <div><strong>${state.queries.filter((query) => query.should_route).length}</strong><span>route labels</span></div>
        <div><strong>${state.queries.filter((query) => !query.should_route).length}</strong><span>abstains</span></div>
      </div>
    </section>
  `;
}

function renderQuerySourcePanel(state: RouterWorkbenchStateData): string {
  const selected = getSelectedQueryRecordData(state);
  const options = getFilteredQueriesList(state)
    .map((query) => {
      const expected = query.required_tool_ids[0] ?? "abstain";
      return `<option value="${escapeAttributeText(query.id)}" ${query.id === state.selectedQueryId ? "selected" : ""}>${escapeHtmlText(`${query.id}: ${query.query.slice(0, 92)} -> ${expected}`)}</option>`;
    })
    .join("");
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
            <label class="field-stack" for="benchmark-query-select">
              <span>Benchmark query</span>
              <select id="benchmark-query-select">${options}</select>
            </label>
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
      <button id="export-evidence-button" class="secondary-action" ${!state.routeResponse ? "disabled" : ""}>Export Evidence</button>
      <button id="export-logs-button" class="secondary-action">Export Logs</button>
      ${state.exportMessage ? `<p class="export-message">${escapeHtmlText(state.exportMessage)}</p>` : ""}
    </section>
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
            <article class="candidate-card">
              <div class="candidate-rank">#${candidate.rank}</div>
              <div>
                <h3>${escapeHtmlText(candidate.tool_id)}</h3>
                <p>${escapeHtmlText(candidate.why_matched)}</p>
                <dl>
                  <div><dt>Score</dt><dd>${formatMetricValue(candidate.score)}</dd></div>
                  <div><dt>Fields</dt><dd>${escapeHtmlText(candidate.matched_fields.join(", ") || "none")}</dd></div>
                  <div><dt>Terms</dt><dd>${escapeHtmlText(candidate.matched_terms.slice(0, 8).join(", ") || "none")}</dd></div>
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
              <div><strong>${formatMetricValue(report.recall_at_k["5"] ?? 0)}</strong><span>Recall@5</span></div>
              <div><strong>${formatMetricValue(report.mrr)}</strong><span>MRR</span></div>
              <div><strong>${formatMetricValue(report.ndcg_at_10)}</strong><span>nDCG@10</span></div>
              <div><strong>${formatMetricValue(report.abstention_accuracy)}</strong><span>Abstention</span></div>
            </div>
          `
          : `<p class="empty-state">Run benchmark eval to compare the selected CPU router.</p>`
      }
    </section>
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
