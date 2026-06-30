import type {
  AnalysisResult,
  AppReadiness,
  ApplySelectedFixesResponse,
  DownloadTextFunction,
  FindingGroup,
  InvokeFunction,
  ReadFileTextFunction,
  ReverifyPromptResult,
} from "./types";

export type { InvokeFunction };

interface PieAppOptions {
  downloadText?: DownloadTextFunction;
  readFileText?: ReadFileTextFunction;
}

type ApiKeyStatus = "idle" | "validating" | "accepted" | "failed";
type AnalysisStatus = "idle" | "analyzing" | "complete" | "failed";
type PatchStatus = "idle" | "applying";
type ReverifyStatus = "idle" | "running" | "complete" | "failed";

interface PieState {
  apiKey: string;
  apiKeyStatus: ApiKeyStatus;
  analysisStatus: AnalysisStatus;
  analysisProgress: number;
  analysisStage: string;
  readiness: AppReadiness;
  promptFilename: string;
  promptJson: string;
  analysis: AnalysisResult | null;
  reverifySourceAnalysis: AnalysisResult | null;
  reverifyResult: ReverifyPromptResult | null;
  reverifyStatus: ReverifyStatus;
  updatedPromptJson: string;
  updatedVersionName: string;
  patchStatus: PatchStatus;
  exportMessage: string;
  patchMessage: string;
  activityLog: string[];
  errorMessage: string;
}

const defaultReadiness: AppReadiness = {
  api_key_ready: false,
  storage_ready: false,
  analyze_enabled: false,
  model_label: "Model: gpt-5.4-mini",
  readiness_message: "Checking readiness.",
};

export function createPieApp(
  root: HTMLElement,
  invoke: InvokeFunction,
  options: PieAppOptions = {},
) {
  const state: PieState = {
    apiKey: "",
    apiKeyStatus: "idle",
    analysisStatus: "idle",
    analysisProgress: 0,
    analysisStage: "Waiting for prompt.",
    readiness: defaultReadiness,
    promptFilename: "",
    promptJson: "",
    analysis: null,
    reverifySourceAnalysis: null,
    reverifyResult: null,
    reverifyStatus: "idle",
    updatedPromptJson: "",
    updatedVersionName: "",
    patchStatus: "idle",
    exportMessage: "",
    patchMessage: "",
    activityLog: ["Opened PIE."],
    errorMessage: "",
  };

  const downloadText = options.downloadText ?? downloadTextInBrowser;
  const readFileText = options.readFileText ?? ((file: File) => file.text());
  let analysisProgressTimer: ReturnType<typeof window.setInterval> | null = null;

  const refreshReadiness = async () => {
    try {
      state.readiness = await invoke<AppReadiness>("get_app_readiness", {
        apiKey: state.apiKey || null,
      });
      state.apiKeyStatus = state.readiness.api_key_ready ? "accepted" : "idle";
      pushActivityMessage(state.readiness.readiness_message);
    } catch (error) {
      const message = errorToMessage(error);
      state.errorMessage = message;
      markReadinessFailed(message);
    }
    render();
  };

  const validateKey = async () => {
    if (!state.apiKey || state.apiKeyStatus === "validating") return;
    try {
      state.errorMessage = "";
      state.apiKeyStatus = "validating";
      state.readiness = {
        ...state.readiness,
        api_key_ready: false,
        analyze_enabled: false,
        readiness_message: "Validating OpenAI key...",
      };
      pushActivityMessage("Validating OpenAI key.");
      render();

      state.readiness = await invoke<AppReadiness>("validate_api_key", {
        apiKey: state.apiKey,
      });
      state.apiKeyStatus = state.readiness.api_key_ready ? "accepted" : "failed";
      pushActivityMessage(state.readiness.readiness_message);
    } catch (error) {
      const message = errorToMessage(error);
      state.errorMessage = message;
      markReadinessFailed(message);
    }
    render();
  };

  const markReadinessFailed = (message: string) => {
    state.apiKeyStatus = "failed";
    state.readiness = {
      ...state.readiness,
      api_key_ready: false,
      analyze_enabled: false,
      readiness_message: "OpenAI readiness failed.",
    };
    pushActivityMessage(`Readiness check failed: ${message}`);
  };

  const pushActivityMessage = (message: string) => {
    if (state.activityLog.at(-1) !== message) {
      state.activityLog.push(message);
    }
  };

  const analyzePrompt = async (command = "analyze_prompt") => {
    try {
      state.errorMessage = "";
      state.analysisStatus = "analyzing";
      state.analysisProgress = 12;
      state.analysisStage = "Parsing prompt JSON.";
      pushActivityMessage("Analyzing prompt.");
      startAnalysisProgressTicker();
      render();

      state.analysis = await invoke<AnalysisResult>(command, {
        request: {
          filename: state.promptFilename,
          prompt_json: state.promptJson,
          api_key: state.apiKey,
        },
      });
      stopAnalysisProgressTicker();
      state.analysisStatus = "complete";
      state.analysisProgress = 100;
      state.analysisStage = "Analysis complete.";
      state.reverifySourceAnalysis = null;
      state.reverifyResult = null;
      state.reverifyStatus = "idle";
      state.analysis.action_log.forEach(pushActivityMessage);
    } catch (error) {
      stopAnalysisProgressTicker();
      state.analysisStatus = "failed";
      state.analysisStage = "Analysis failed.";
      state.errorMessage = errorToMessage(error);
    }
    render();
  };

  const startAnalysisProgressTicker = () => {
    stopAnalysisProgressTicker();
    const stages = [
      { progress: 28, label: "Normalizing prompt sections." },
      { progress: 46, label: "Checking tool and workflow risks." },
      { progress: 64, label: "Running LLM Judge." },
      { progress: 82, label: "Preparing finding groups." },
      { progress: 92, label: "Waiting for model response." },
    ];
    let index = 0;
    analysisProgressTimer = window.setInterval(() => {
      const next = stages[Math.min(index, stages.length - 1)];
      state.analysisProgress = Math.max(state.analysisProgress, next.progress);
      state.analysisStage = next.label;
      index += 1;
      render();
    }, 900);
  };

  const stopAnalysisProgressTicker = () => {
    if (analysisProgressTimer !== null) {
      window.clearInterval(analysisProgressTimer);
      analysisProgressTimer = null;
    }
  };

  const exportFindings = async () => {
    if (!state.analysis) return;
    try {
      const content = await invoke<string>("export_findings", {
        findingGroups: state.analysis.finding_groups,
      });
      const filename = findingsExportFilename(state.analysis.prompt_version_name);
      const target = downloadText(filename, content);
      state.exportMessage = downloadCompletionMessage("findings", filename, target);
      pushActivityMessage(state.exportMessage);
    } catch (error) {
      state.errorMessage = errorToMessage(error);
    }
    render();
  };

  const exportDiagnosticLogs = async () => {
    try {
      state.errorMessage = "";
      const content = await invoke<string>("export_diagnostic_logs");
      downloadText("pie-diagnostic-log.txt", content);
      pushActivityMessage("Downloaded diagnostic logs.");
    } catch (error) {
      state.errorMessage = errorToMessage(error);
    }
    render();
  };

  const applyRecommendedPatch = async () => {
    if (!state.analysis || state.patchStatus === "applying") return;
    const recommended_finding_ids = recommendedPatchFindingIds(state.analysis);
    if (recommended_finding_ids.length === 0) return;

    try {
      state.errorMessage = "";
      state.patchStatus = "applying";
      state.patchMessage = "Applying recommended patch.";
      render();
      const patch = await invoke<ApplySelectedFixesResponse>("apply_selected_fixes", {
        request: {
          original_prompt_json: state.promptJson,
          finding_groups: state.analysis.finding_groups,
          selected_finding_ids: recommended_finding_ids,
          api_key: state.apiKey,
        },
      });
      state.updatedPromptJson = patch.result.candidate.updated_prompt_json;
      state.updatedVersionName = patch.updated_version_name;
      state.patchMessage = `Patch applied: ${patch.result.candidate.patch_summary}`;
      patch.action_log.forEach(pushActivityMessage);
      pushActivityMessage("Patch applied.");
    } catch (error) {
      state.errorMessage = errorToMessage(error);
      state.updatedPromptJson = "";
      state.updatedVersionName = "";
      state.patchMessage = "";
    } finally {
      state.patchStatus = "idle";
    }
    render();
  };

  const reverifyPrompt = async () => {
    const sourceAnalysis = state.analysis ?? state.reverifySourceAnalysis;
    if (!sourceAnalysis || !state.updatedPromptJson || state.reverifyStatus === "running") {
      return;
    }

    try {
      state.errorMessage = "";
      state.reverifyStatus = "running";
      state.reverifyResult = null;
      state.reverifySourceAnalysis = sourceAnalysis;
      state.analysis = null;
      pushActivityMessage("Re-verifying updated prompt.");
      render();

      state.reverifyResult = await invoke<ReverifyPromptResult>("reverify_prompt", {
        request: {
          api_key: state.apiKey,
          original_prompt_json: state.promptJson,
          updated_prompt_json: state.updatedPromptJson,
          previous_finding_groups: sourceAnalysis.finding_groups,
          updated_version_name: state.updatedVersionName || "updated_prompt",
        },
      });
      state.reverifyStatus = "complete";
      state.reverifyResult.action_log.forEach(pushActivityMessage);
    } catch (error) {
      state.reverifyStatus = "failed";
      state.errorMessage = errorToMessage(error);
    }
    render();
  };

  const handlePromptFile = async (file: File | undefined) => {
    if (!file) return;
    try {
      state.promptFilename = file.name;
      state.promptJson = await readFileText(file);
      pushActivityMessage(`Uploaded ${file.name}.`);
    } catch (error) {
      state.errorMessage = errorToMessage(error);
    }
    render();
  };

  const bindEvents = () => {
    root.querySelector<HTMLInputElement>("#api-key")?.addEventListener("input", (event) => {
      state.apiKey = (event.currentTarget as HTMLInputElement).value;
      state.apiKeyStatus = "idle";
      if (state.readiness.api_key_ready || state.readiness.analyze_enabled) {
        state.readiness = {
          ...state.readiness,
          api_key_ready: false,
          analyze_enabled: false,
          readiness_message: "API key changed. Validate again.",
        };
      }
      render();
    });
    root.querySelector<HTMLButtonElement>("#validate-key")?.addEventListener("click", () => {
      void validateKey();
    });
    root.querySelector<HTMLInputElement>("#prompt-file")?.addEventListener("change", (event) => {
      void handlePromptFile((event.currentTarget as HTMLInputElement).files?.[0]);
    });
    root.querySelector<HTMLButtonElement>("#analyze-prompt")?.addEventListener("click", () => {
      void analyzePrompt();
    });
    root.querySelector<HTMLButtonElement>("#download-findings")?.addEventListener("click", () => {
      void exportFindings();
    });
    root.querySelector<HTMLButtonElement>("#download-logs")?.addEventListener("click", () => {
      void exportDiagnosticLogs();
    });
    root.querySelector<HTMLButtonElement>("#apply-recommended")?.addEventListener("click", () => {
      void applyRecommendedPatch();
    });
    root.querySelector<HTMLButtonElement>("#download-updated")?.addEventListener("click", () => {
      if (state.updatedPromptJson) {
        const filename = `${state.updatedVersionName || "updated_prompt"}.json`;
        const target = downloadText(filename, state.updatedPromptJson);
        state.exportMessage = downloadCompletionMessage("updated prompt", filename, target);
        pushActivityMessage(state.exportMessage);
        render();
      }
    });
    root.querySelector<HTMLButtonElement>("#reverify-prompt")?.addEventListener("click", () => {
      void reverifyPrompt();
    });
  };

  const render = () => {
    const isAnalyzing = state.analysisStatus === "analyzing";
    const isBusy = isAnalyzing || state.patchStatus === "applying" || state.reverifyStatus === "running";
    const canAnalyze = state.readiness.analyze_enabled && Boolean(state.promptJson) && !isBusy;
    const canApply =
      Boolean(state.analysis) &&
      recommendedPatchFindingIds(state.analysis).length > 0 &&
      state.patchStatus !== "applying";
    const readiness = workspaceReadinessCopy(state);
    root.innerHTML = `
      <section class="app-shell">
        <section class="workbench">
          <header class="workspace-header">
            <div class="workspace-brand">
              <p class="eyebrow">Prompt Iteration Engine</p>
              <h1>PIE</h1>
              <p class="hero-copy">Versioned prompt review for healthcare voice-agent operators.</p>
            </div>
            <section class="workspace-status workspace-status--${readiness.tone}">
              <div class="workspace-chip">
                <span class="status-dot" aria-hidden="true"></span>
                <strong>${escapeHtml(readiness.title)}</strong>
              </div>
              <p>${escapeHtml(readiness.detail)}</p>
              <dl class="workspace-meta">
                <div>
                  <dt>Model</dt>
                  <dd>${escapeHtml(state.readiness.model_label)}</dd>
                </div>
                <div>
                  <dt>Storage</dt>
                  <dd>${state.readiness.storage_ready ? "Ready" : "Blocked"}</dd>
                </div>
              </dl>
            </section>
          </header>

          ${renderCredentialStrip(state)}

          <section class="prompt-input prompt-panel">
            <header class="panel-header">
              <div>
                <p class="eyebrow">Prompt workspace</p>
                <h2>Assignment Prompt</h2>
                <p>Upload the assignment prompt JSON, then run the five-part judge.</p>
              </div>
            </header>
            <div class="prompt-actions">
              <label class="file-picker" for="prompt-file">
                <span>Select Prompt JSON</span>
                <input id="prompt-file" type="file" accept="application/json,.json" />
              </label>
              <div class="file-badge ${state.promptFilename ? "file-badge--loaded" : ""}">
                ${state.promptFilename ? `Loaded ${escapeHtml(state.promptFilename)}` : "No prompt loaded"}
              </div>
              <button id="analyze-prompt" class="primary-action ${isAnalyzing ? "is-loading" : ""}" ${canAnalyze ? "" : "disabled"}>
                ${isAnalyzing ? `<span class="spinner" aria-hidden="true"></span>Analyzing` : "Analyze Prompt"}
              </button>
            </div>
            ${renderAnalysisProgress(state)}
          </section>

          ${state.errorMessage ? `<section class="error">${escapeHtml(state.errorMessage)}</section>` : ""}
          ${renderUpdatedPrompt(state)}
          ${renderFindings(state)}
        </section>

        <aside class="activity-log">
          <header class="activity-header">
            <h2>Activity</h2>
            <button id="download-logs" class="secondary-action">Download Logs</button>
          </header>
          <ol>${state.activityLog
            .slice(-8)
            .map((item) => `<li>${escapeHtml(item)}</li>`)
            .join("")}</ol>
        </aside>
      </section>
    `;

    root.querySelector<HTMLButtonElement>("#download-findings")?.toggleAttribute(
      "disabled",
      !state.analysis,
    );
    root.querySelector<HTMLButtonElement>("#apply-recommended")?.toggleAttribute("disabled", !canApply);
    root.querySelector<HTMLButtonElement>("#reverify-prompt")?.toggleAttribute(
      "disabled",
      !state.updatedPromptJson || isBusy,
    );
    bindEvents();
  };

  render();
  void refreshReadiness();
}

function renderCredentialStrip(state: PieState) {
  const status = keyStatusCopy(state);
  const isValidating = state.apiKeyStatus === "validating";
  const buttonLabel = isValidating
    ? "Validating"
    : state.apiKeyStatus === "accepted"
      ? "Validate Again"
      : "Validate Key";

  return `
    <section class="session-card" aria-label="API key setup">
      <header class="session-header">
        <div>
          <p class="eyebrow">Session key</p>
          <h2>OpenAI API key</h2>
        </div>
        <p class="session-note">Kept in memory for this session only. Never exported or stored in prompt versions.</p>
      </header>
      <div class="session-controls">
        <label class="session-input">OpenAI API key
          <input id="api-key" type="password" value="${escapeAttribute(state.apiKey)}" autocomplete="off" />
        </label>
        <button id="validate-key" class="validate-button ${isValidating ? "is-loading" : ""}" ${
          state.apiKey && !isValidating ? "" : "disabled"
        }>
          ${isValidating ? `<span class="spinner" aria-hidden="true"></span>` : ""}
          ${buttonLabel}
        </button>
      </div>
      <div class="session-foot">
        <div class="key-status key-status--${status.tone}" role="status">
          <span class="status-dot" aria-hidden="true"></span>
          <span>
            <strong>${escapeHtml(status.title)}</strong>
            <small>${escapeHtml(status.detail)}</small>
          </span>
        </div>
        <p class="privacy-note">Session key unlocks analysis only after validation with ${escapeHtml(state.readiness.model_label)}.</p>
      </div>
    </section>
  `;
}

function workspaceReadinessCopy(state: PieState) {
  if (!state.readiness.storage_ready) {
    return {
      tone: "failed",
      title: "Storage blocked",
      detail: "Local storage is not ready for prompt versioning.",
    };
  }

  if (state.apiKeyStatus === "validating") {
    return {
      tone: "validating",
      title: "Checking key",
      detail: "Validating session access before analysis.",
    };
  }

  if (state.analysisStatus === "analyzing") {
    return {
      tone: "validating",
      title: "Analysis running",
      detail: state.analysisStage,
    };
  }

  if (state.errorMessage) {
    return {
      tone: "failed",
      title: "Attention needed",
      detail: state.errorMessage,
    };
  }

  if (state.readiness.analyze_enabled) {
    return {
      tone: "accepted",
      title: "Ready",
      detail: "Prompt can be analyzed with the current session key.",
    };
  }

  return {
    tone: "idle",
    title: "Session setup",
    detail: state.readiness.readiness_message,
  };
}

function keyStatusCopy(state: PieState) {
  switch (state.apiKeyStatus) {
    case "validating":
      return {
        tone: "validating",
        title: "Validating key",
        detail: "Checking gpt-5.4-mini access.",
      };
    case "accepted":
      return {
        tone: "accepted",
        title: "API key accepted",
        detail: state.readiness.readiness_message,
      };
    case "failed":
      return {
        tone: "failed",
        title: "Key not ready",
        detail: state.readiness.readiness_message,
      };
    case "idle":
      return {
        tone: "idle",
        title: state.apiKey ? "Key not validated" : "Key required",
        detail: state.apiKey
          ? "Validate this session key before analysis."
          : "Enter a session key to unlock analysis.",
      };
  }
}

function renderAnalysisProgress(state: PieState) {
  if (state.analysisStatus === "idle") return "";

  const statusLabel =
    state.analysisStatus === "analyzing"
      ? "Analyzing prompt"
      : state.analysisStatus === "complete"
        ? "Analysis complete"
        : "Analysis failed";

  return `
    <div class="analysis-progress analysis-progress--${state.analysisStatus}">
      <div class="progress-header">
        <strong>${escapeHtml(statusLabel)}</strong>
        <span class="progress-text">${state.analysisProgress}%</span>
      </div>
      <div class="progress-track" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow="${state.analysisProgress}">
        <span class="progress-fill" style="width: ${state.analysisProgress}%"></span>
      </div>
      <p>${escapeHtml(state.analysisStage)}</p>
    </div>
  `;
}

function renderFindings(state: PieState) {
  if (state.reverifyStatus === "running") {
    return `
      <section class="findings">
        <header>
          <div>
            <h2>Reverify Results</h2>
            <p>Re-verifying updated prompt against prior findings and patch history.</p>
          </div>
          <span>${escapeHtml(state.readiness.model_label)}</span>
        </header>
        <div class="analysis-progress analysis-progress--analyzing">
          <div class="progress-header">
            <strong>Re-verifying updated prompt</strong>
            <span class="progress-text">Working</span>
          </div>
          <div class="progress-track" role="progressbar" aria-valuemin="0" aria-valuemax="100">
            <span class="progress-fill" style="width: 82%"></span>
          </div>
          <p>Old findings are hidden while PIE compares the updated prompt with previous issues.</p>
        </div>
      </section>
    `;
  }

  if (state.reverifyResult && state.reverifySourceAnalysis) {
    return renderReverifyResults(state);
  }

  if (!state.analysis) return "";
  const visibleGroups = visibleFindingGroups(state.analysis);
  const reverifyTarget = state.updatedVersionName || state.analysis.prompt_version_name;
  const rows = visibleGroups
    .flatMap((group) =>
      group.findings.map((finding) => {
        const action = recommendedActionForFinding(group.section, finding);
        return `
          <tr>
            <td>
              <p class="finding-section">${escapeHtml(sectionLabel(group.section))}</p>
              <strong>${escapeHtml(finding.title)}</strong>
              <small>${escapeHtml(finding.severity)} / ${escapeHtml(finding.fix_mode)}</small>
              <span class="finding-impact">${escapeHtml(finding.impact)}</span>
            </td>
            <td>
              <strong class="recommendation-home">${escapeHtml(action.betterHome)}</strong>
              <small class="recommendation-explain">${escapeHtml(actionExplanationForHome(action.betterHome))}</small>
              ${renderRecommendationBullets(action.bullets)}
            </td>
          </tr>
        `;
      }),
    )
    .join("");

  return `
    <section class="findings">
      <header>
        <div>
          <h2>Findings</h2>
          <p>Review evidence, then apply one recommended patch across the non-backlog findings.</p>
        </div>
        <span>${escapeHtml(state.analysis.model_label)}</span>
      </header>
      <div class="finding-actions">
        <button id="download-findings">Download Findings</button>
        <button id="apply-recommended" class="${state.patchStatus === "applying" ? "is-loading" : ""}">
          ${state.patchStatus === "applying" ? `<span class="spinner" aria-hidden="true"></span>Applying Patch` : "Apply Recommended Patch"}
        </button>
        <button id="reverify-prompt" class="secondary-action reverify-action" ${
          state.updatedPromptJson ? "" : "disabled"
        }>
          Re-verify Updated Prompt
        </button>
      </div>
      <p class="reverify-target">Re-verify target: ${escapeHtml(reverifyTarget)}</p>
      ${
        state.exportMessage.startsWith("Downloaded findings")
          ? `<p class="download-status">${escapeHtml(state.exportMessage)}</p>`
          : ""
      }
      <table class="findings-table">
        <thead>
          <tr>
            <th>Findings</th>
            <th>Recommended Actions</th>
          </tr>
        </thead>
        <tbody>${rows}</tbody>
      </table>
    </section>
  `;
}

function renderReverifyResults(state: PieState) {
  const sourceAnalysis = state.reverifySourceAnalysis;
  const result = state.reverifyResult;
  if (!sourceAnalysis || !result) return "";

  const statuses = new Map(result.finding_statuses.map((status) => [status.finding_id, status]));
  const rows = visibleFindingGroups(sourceAnalysis)
    .flatMap((group) =>
      group.findings.map((finding) => {
        const action = recommendedActionForFinding(group.section, finding);
        const status = statuses.get(finding.finding_id) ?? {
          finding_id: finding.finding_id,
          status: "Unknown" as const,
          status_label: "Unknown",
          rationale: "No reverify status was returned for this finding.",
        };
        return `
          <tr>
            <td>
              <p class="finding-section">${escapeHtml(sectionLabel(group.section))}</p>
              <strong>${escapeHtml(finding.title)}</strong>
              <small>${escapeHtml(finding.severity)} / ${escapeHtml(finding.fix_mode)}</small>
              <span class="finding-impact">${escapeHtml(finding.impact)}</span>
            </td>
            <td>
              <strong class="recommendation-home">${escapeHtml(action.betterHome)}</strong>
              <small class="recommendation-explain">${escapeHtml(actionExplanationForHome(action.betterHome))}</small>
              ${renderRecommendationBullets(action.bullets)}
            </td>
            <td>
              <strong class="reverify-status ${statusClassName(status.status)}">${escapeHtml(status.status_label)}</strong>
              <span class="reverify-rationale">${escapeHtml(status.rationale)}</span>
            </td>
          </tr>
        `;
      }),
    )
    .join("");

  return `
    <section class="findings">
      <header>
        <div>
          <h2>Reverify Results</h2>
          <p>Comparison against the prior findings for ${escapeHtml(result.prompt_version_name)}.</p>
        </div>
        <span>${escapeHtml(result.model_label)}</span>
      </header>
      <div class="finding-actions">
        <button id="download-findings" disabled>Download Findings</button>
        <button id="apply-recommended" disabled>Apply Recommended Patch</button>
        <button id="reverify-prompt" class="secondary-action reverify-action">Re-verify Updated Prompt</button>
      </div>
      <table class="findings-table findings-table--reverify">
        <thead>
          <tr>
            <th>Findings</th>
            <th>Recommended Actions</th>
            <th>Reverify Status</th>
          </tr>
        </thead>
        <tbody>${rows}</tbody>
      </table>
    </section>
  `;
}

function renderUpdatedPrompt(state: PieState) {
  if (!state.updatedPromptJson && !state.patchMessage) return "";
  return `
    <section class="updated-prompt">
      <div>
        <p class="eyebrow">Patch result</p>
        <h2>Updated Prompt</h2>
        ${state.patchMessage ? `<p>${escapeHtml(state.patchMessage)}</p>` : ""}
        ${state.updatedVersionName ? `<small>Version: ${escapeHtml(state.updatedVersionName)}</small>` : ""}
        ${
          state.exportMessage.startsWith("Downloaded updated prompt")
            ? `<p class="download-status">${escapeHtml(state.exportMessage)}</p>`
            : ""
        }
      </div>
      <button id="download-updated" ${state.updatedPromptJson ? "" : "disabled"}>
        Download Updated Prompt
      </button>
    </section>
  `;
}

function visibleFindingGroups(analysis: AnalysisResult) {
  return analysis.finding_groups.filter((group) => group.section !== "Verification");
}

function recommendedPatchFindingIds(analysis: AnalysisResult | null) {
  if (!analysis) return [];
  return visibleFindingGroups(analysis)
    .flatMap((group) => group.findings)
    .filter((finding) => finding.fix_mode !== "Backlog")
    .map((finding) => finding.finding_id);
}

function recommendedActionForFinding(
  section: FindingGroup["section"],
  finding: FindingGroup["findings"][number],
) {
  const text = [
    finding.finding_id,
    finding.title,
    finding.verification_scenario,
  ]
    .join(" ")
    .toLowerCase();

  if (section === "SafetyPhi") {
    return {
      betterHome: "Invariant Policy",
      bullets: [
        "Verify identity before any PHI disclosure.",
        "Keep no-medical-advice and emergency escalation gates explicit.",
        "Do not continue normal scheduling when safety gates trigger.",
      ],
    };
  }

  if (text.includes("waitlist")) {
    return {
      betterHome: "Capability Variant",
      bullets: [
        "Use `create_waitlist_entry(...)` only if that tool exists.",
        "Otherwise replace the promise with honest staff handoff language.",
      ],
    };
  }

  if (text.includes("interpreter") || text.includes("language")) {
    return {
      betterHome: "Capability Variant",
      bullets: [
        "Add `request_interpreter(patient_id, language, appointment_id)` as a placeholder capability.",
        "If unavailable, route to staff without claiming support was arranged.",
      ],
    };
  }

  if (
    text.includes("callback") ||
    text.includes("followup") ||
    text.includes("follow-up") ||
    text.includes("note-taking") ||
    text.includes("note taking") ||
    text.includes("internal-note")
  ) {
    return {
      betterHome: "Capability Variant",
      bullets: [
        "Add `create_staff_followup_task(patient_id, reason, priority)`.",
        "If unavailable, use real transfer/handoff instead of a promise.",
      ],
    };
  }

  if (text.includes("reschedule") || text.includes("cancel")) {
    return {
      betterHome: "Invariant Workflow",
      bullets: [
        "Call `get_available_slots` before any state change.",
        "Read back the replacement slot and get confirmation.",
        "Book the new appointment before canceling the old one.",
      ],
    };
  }

  if (
    text.includes("schedule") ||
    text.includes("provider") ||
    text.includes("availability") ||
    text.includes("office hours") ||
    text.includes("insurance") ||
    text.includes("appointment duration")
  ) {
    return {
      betterHome: "Variant Data",
      bullets: [
        "Move provider facts out of static prompt prose.",
        "Read schedules through `get_provider_availability(...)`.",
        "Read clinic facts through `get_clinic_config(config_key)`.",
      ],
    };
  }

  return {
    betterHome:
      section === "ToolGaps"
        ? "Capability Variant"
        : section === "WorkflowOrder"
          ? "Invariant Workflow"
          : section === "Structure"
            ? "Variant Data"
            : "Eval Invariant",
    bullets: splitActionIntoBullets(finding.suggested_fix),
  };
}

function sectionLabel(section: FindingGroup["section"]) {
  switch (section) {
    case "Structure":
      return "Variant Data Placement";
    case "ToolGaps":
      return "Capability Variants";
    case "WorkflowOrder":
      return "Invariant Workflow Order";
    case "SafetyPhi":
      return "Invariant Safety / PHI";
    case "Verification":
      return "Eval Invariants";
    default:
      return section;
  }
}

function actionExplanationForHome(betterHome: string) {
  switch (betterHome) {
    case "Variant Data":
      return "Clinic or provider facts that should live in config or a lookup, not buried in static prompt prose.";
    case "Capability Variant":
      return "A promise is safe only when the tool surface can actually complete that action.";
    case "Invariant Workflow":
      return "Stable sequencing rule: keep state changes after lookup, readback, and confirmation.";
    case "Invariant Policy":
      return "Safety and PHI rule that should apply across every clinic-specific workflow.";
    case "Eval Invariant":
      return "Evidence requirement: prove the fix with before/after checks and regressions.";
    default:
      return "Classify the finding by the safest source of truth before changing prompt text.";
  }
}

function renderRecommendationBullets(bullets: string[]) {
  return `
    <ul class="recommendation-list">
      ${bullets.map((bullet) => `<li>${escapeHtml(bullet)}</li>`).join("")}
    </ul>
  `;
}

function splitActionIntoBullets(action: string) {
  const parts = action
    .split(/(?<=\.)\s+/)
    .map((part) => part.trim())
    .filter(Boolean);

  return parts.length > 0 ? parts.slice(0, 3) : ["Review and apply the smallest safe prompt change."];
}

function statusClassName(status: string) {
  switch (status) {
    case "Fixed":
      return "status-fixed";
    case "StillFailing":
      return "status-still-failing";
    default:
      return "status-unknown";
  }
}

function findingsExportFilename(promptVersionName: string) {
  return `pie-findings-${promptVersionName.replace(/[^a-zA-Z0-9._-]/g, "-")}.md`;
}

function downloadCompletionMessage(kind: string, filename: string, target: string | void) {
  return `Downloaded ${kind} to ${target || `~/Downloads/${filename}`}.`;
}

function downloadTextInBrowser(filename: string, content: string) {
  const blob = new Blob([content], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.click();
  URL.revokeObjectURL(url);
  return `~/Downloads/${filename}`;
}

function errorToMessage(error: unknown) {
  if (typeof error === "object" && error && "message" in error) {
    return String((error as { message: unknown }).message);
  }
  return String(error);
}

function escapeHtml(value: string) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function escapeAttribute(value: string) {
  return escapeHtml(value);
}
