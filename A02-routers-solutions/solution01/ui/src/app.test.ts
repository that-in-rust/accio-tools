import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPieApp, type InvokeFunction } from "./app";
import styles from "./styles.css?raw";

const assignmentPrompt = JSON.stringify({
  agent_name: "Greenfield Medical Group - Front Desk Agent",
  model: "gpt-4.1",
  general_prompt: "waitlist\nCancel the old appointment first",
  general_tools: [],
});

describe("PIE S04 journey", () => {
  beforeEach(() => {
    document.body.innerHTML = '<main id="app"></main>';
  });

  it("shows missing-key setup, privacy note, and fixed model label", async () => {
    const invoke = vi.fn(async (command: string) => {
      if (command === "get_app_readiness") {
        return {
          api_key_ready: false,
          storage_ready: true,
          analyze_enabled: false,
          model_label: "Model: gpt-5.4-mini",
          readiness_message: "OpenAI API key is missing.",
        };
      }
      throw new Error(command);
    }) as InvokeFunction;

    createPieApp(document.querySelector("#app") as HTMLElement, invoke);
    await Promise.resolve();

    expect(screenText()).toContain("Model: gpt-5.4-mini");
    expect(screenText()).toContain("OpenAI API key");
    expect(screenText()).toContain("Kept in memory for this session only");
    expect(button("Analyze Prompt").disabled).toBe(true);
  });

  it("puts key readiness at the top and prompt work in the primary panel", async () => {
    const invoke = vi.fn(async (command: string) => {
      if (command === "get_app_readiness") {
        return {
          api_key_ready: false,
          storage_ready: true,
          analyze_enabled: false,
          model_label: "Model: gpt-5.4-mini",
          readiness_message: "OpenAI API key is missing.",
        };
      }
      throw new Error(command);
    }) as InvokeFunction;

    createPieApp(document.querySelector("#app") as HTMLElement, invoke);
    await Promise.resolve();

    const keyStrip = document.querySelector(".session-card");
    const promptPanel = document.querySelector(".prompt-panel");

    expect(keyStrip?.textContent).toContain("OpenAI API key");
    expect(promptPanel?.querySelector("h2")?.textContent).toBe("Assignment Prompt");
    expect(keyStrip?.compareDocumentPosition(promptPanel as Node)).toBe(
      Node.DOCUMENT_POSITION_FOLLOWING,
    );
  });

  it("shows sanitized readiness failure instead of stale missing-key status", async () => {
    const invoke = vi.fn(async (command: string) => {
      if (command === "get_app_readiness") {
        return {
          api_key_ready: false,
          storage_ready: true,
          analyze_enabled: false,
          model_label: "Model: gpt-5.4-mini",
          readiness_message: "OpenAI API key is missing.",
        };
      }
      if (command === "validate_api_key") {
        throw {
          code: "invalid_request",
          message:
            "invalid request: Invalid 'max_output_tokens': integer below minimum value. Expected a value >= 16, but got 8 instead.",
          hint: "Correct the request and retry.",
        };
      }
      throw new Error(command);
    }) as InvokeFunction;

    createPieApp(document.querySelector("#app") as HTMLElement, invoke);
    await Promise.resolve();
    input("OpenAI API key").value = "sk-test";
    input("OpenAI API key").dispatchEvent(new Event("input", { bubbles: true }));
    await click("Validate Key");

    expect(document.querySelector(".workspace-status")?.textContent).toContain("Attention needed");
    expect(screenText()).toContain("max_output_tokens");
    expect(screenText()).toContain("Readiness check failed");
    expect(button("Analyze Prompt").disabled).toBe(true);
  });

  it("shows immediate key validation progress and accepted state in setup", async () => {
    let resolveValidation: ((value: ReturnType<typeof readyState>) => void) | undefined;
    const invoke = vi.fn((command: string) => {
      if (command === "get_app_readiness") {
        return Promise.resolve({
          api_key_ready: false,
          storage_ready: true,
          analyze_enabled: false,
          model_label: "Model: gpt-5.4-mini",
          readiness_message: "OpenAI API key is missing.",
        });
      }
      if (command === "validate_api_key") {
        return new Promise((resolve) => {
          resolveValidation = resolve;
        });
      }
      return Promise.reject(new Error(command));
    }) as InvokeFunction;

    createPieApp(document.querySelector("#app") as HTMLElement, invoke);
    await Promise.resolve();
    input("OpenAI API key").value = "sk-test";
    input("OpenAI API key").dispatchEvent(new Event("input", { bubbles: true }));
    button("Validate Key").click();
    await Promise.resolve();

    expect(document.querySelector(".session-card")?.textContent).toContain("Validating key");
    expect(button("Validating").disabled).toBe(true);
    expect(screenText()).toContain("Checking gpt-5.4-mini access");

    resolveValidation?.(readyState());
    await Promise.resolve();
    await Promise.resolve();

    expect(document.querySelector(".session-card")?.textContent).toContain("API key accepted");
    expect(document.querySelector(".session-card")?.textContent).toContain(
      "Ready to analyze prompt.",
    );
    expect(button("Validate Again").disabled).toBe(false);
  });

  it("keeps session status stacked inside the session card to prevent overlap", async () => {
    const invoke = vi.fn(async (command: string) => {
      if (command === "get_app_readiness") {
        return readyState();
      }
      throw new Error(command);
    }) as InvokeFunction;

    createPieApp(document.querySelector("#app") as HTMLElement, invoke);
    await Promise.resolve();

    expect(document.querySelector(".session-card .session-foot .key-status")).not.toBeNull();
    expect(document.querySelector(".session-card .session-controls .key-status")).toBeNull();
    expect(screenText()).toContain("Session key");
    expect(screenText()).toContain("Assignment Prompt");
  });

  it("runs analyze, renders a findings table, hides verification, and exports findings with status", async () => {
    const downloads: Array<{ filename: string; content: string }> = [];
    const invoke = vi.fn(async (command: string) => {
      if (command === "get_app_readiness" || command === "validate_api_key") {
        return readyState();
      }
      if (command === "analyze_prompt") {
        return analysisResult();
      }
      if (command === "export_findings") {
        return "# PIE Findings Export\n\nModel: gpt-5.4-mini";
      }
      throw new Error(command);
    }) as InvokeFunction;

    createPieApp(document.querySelector("#app") as HTMLElement, invoke, {
      downloadText: (filename, content) => {
        downloads.push({ filename, content });
      },
      readFileText: async () => assignmentPrompt,
    });

    await Promise.resolve();
    input("OpenAI API key").value = "sk-test";
    input("OpenAI API key").dispatchEvent(new Event("input", { bubbles: true }));
    await click("Validate Key");
    await uploadPrompt("assignment-agent-prompt.json");
    await click("Analyze Prompt");

    expect(document.querySelector(".findings-table")?.textContent).toContain("Findings");
    expect(document.querySelector(".findings-table")?.textContent).toContain(
      "Recommended Actions",
    );
    expect(screenText()).toContain("Variant Data");
    expect(screenText()).toContain("Clinic or provider facts that should live in config");
    expect(screenText()).toContain("Capability Variant");
    expect(screenText()).toContain("A promise is safe only when the tool surface");
    expect(screenText()).toContain("get_provider_availability");
    expect(screenText()).toContain("create_waitlist_entry");
    expect(screenText()).toContain("request_interpreter");
    expect(screenText()).toContain("Invariant Policy");
    expect(screenText()).toContain("Verify identity before any PHI disclosure");
    expect(screenText()).not.toContain("Verification");
    expect(document.querySelectorAll('input[type="checkbox"]').length).toBe(0);
    expect(button("Apply Recommended Patch").disabled).toBe(false);
    expect(button("Re-verify Updated Prompt").disabled).toBe(true);
    expect(findingActionLabels()).toEqual([
      "Download Findings",
      "Apply Recommended Patch",
      "Re-verify Updated Prompt",
    ]);

    await click("Download Findings");
    expect(downloads[0]).toEqual({
      filename: "pie-findings-assignment-agent-prompt_v20260615120000000.md",
      content: "# PIE Findings Export\n\nModel: gpt-5.4-mini",
    });
    expect(document.querySelector(".findings .download-status")?.textContent).toContain(
      "Downloaded findings to ~/Downloads/pie-findings-assignment-agent-prompt_v20260615120000000.md",
    );
    expect(document.querySelector(".workbench > .export-message")).toBeNull();
  });

  it("keeps recommended actions in native right-hand table cells", () => {
    expect(styles).not.toMatch(/\.findings-table td:first-child[\s\S]*display:\s*grid/);
    expect(styles).not.toMatch(/\.findings-table td:last-child[\s\S]*display:\s*grid/);
  });

  it("shows immediate analysis progress while judge is running", async () => {
    let resolveAnalysis: ((value: ReturnType<typeof analysisResult>) => void) | undefined;
    const invoke = vi.fn((command: string) => {
      if (command === "get_app_readiness" || command === "validate_api_key") {
        return Promise.resolve(readyState());
      }
      if (command === "analyze_prompt") {
        return new Promise((resolve) => {
          resolveAnalysis = resolve;
        });
      }
      return Promise.reject(new Error(command));
    }) as InvokeFunction;

    createPieApp(document.querySelector("#app") as HTMLElement, invoke, {
      readFileText: async () => assignmentPrompt,
    });

    await Promise.resolve();
    input("OpenAI API key").value = "sk-test";
    input("OpenAI API key").dispatchEvent(new Event("input", { bubbles: true }));
    await click("Validate Key");
    await uploadPrompt("assignment-agent-prompt.json");
    button("Analyze Prompt").click();
    await Promise.resolve();

    expect(document.querySelector(".prompt-panel")?.textContent).toContain("Analyzing prompt");
    expect(document.querySelector(".progress-text")?.textContent).toContain("%");
    expect(button("Analyzing").disabled).toBe(true);

    resolveAnalysis?.(analysisResult());
    await Promise.resolve();
    await Promise.resolve();

    expect(document.querySelector(".prompt-panel")?.textContent).toContain("Analysis complete");
    expect(document.querySelector(".progress-text")?.textContent).toContain("100%");
    expect(screenText()).toContain("Variant Data Placement");
  });

  it("downloads diagnostic logs from the activity panel", async () => {
    const downloads: Array<{ filename: string; content: string }> = [];
    const invoke = vi.fn(async (command: string) => {
      if (command === "get_app_readiness") {
        return {
          api_key_ready: false,
          storage_ready: true,
          analyze_enabled: false,
          model_label: "Model: gpt-5.4-mini",
          readiness_message: "OpenAI API key is missing.",
        };
      }
      if (command === "export_diagnostic_logs") {
        return "# PIE Diagnostic Log\n\ncommand_started validate_api_key";
      }
      throw new Error(command);
    }) as InvokeFunction;

    createPieApp(document.querySelector("#app") as HTMLElement, invoke, {
      downloadText: (filename, content) => {
        downloads.push({ filename, content });
      },
    });
    await Promise.resolve();
    await click("Download Logs");

    expect(invoke).toHaveBeenCalledWith("export_diagnostic_logs");
    expect(downloads[0]).toEqual({
      filename: "pie-diagnostic-log.txt",
      content: "# PIE Diagnostic Log\n\ncommand_started validate_api_key",
    });
    expect(screenText()).toContain("Downloaded diagnostic logs.");
  });

  it("shows patch progress and blocks duplicate patch submits", async () => {
    let resolvePatch: ((value: ApplySelectedFixesResponseFixture) => void) | undefined;
    const invoke = vi.fn((command: string) => {
      if (command === "get_app_readiness" || command === "validate_api_key") {
        return Promise.resolve(readyState());
      }
      if (command === "analyze_prompt") {
        return Promise.resolve(analysisResult());
      }
      if (command === "apply_selected_fixes") {
        return new Promise((resolve) => {
          resolvePatch = resolve;
        });
      }
      return Promise.reject(new Error(command));
    }) as InvokeFunction;

    createPieApp(document.querySelector("#app") as HTMLElement, invoke, {
      readFileText: async () => assignmentPrompt,
    });

    await Promise.resolve();
    input("OpenAI API key").value = "sk-test";
    input("OpenAI API key").dispatchEvent(new Event("input", { bubbles: true }));
    await click("Validate Key");
    await uploadPrompt("assignment-agent-prompt.json");
    await click("Analyze Prompt");
    button("Apply Recommended Patch").click();
    await Promise.resolve();

    expect(button("Applying Patch").disabled).toBe(true);
    expect(screenText()).toContain("Applying recommended patch.");
    expectPatchResultBeforeFindings();
    button("Applying Patch").click();
    await Promise.resolve();
    expect(invoke).toHaveBeenCalledTimes(4);

    resolvePatch?.(patchResponse());
    await Promise.resolve();
    await Promise.resolve();

    expect(button("Apply Recommended Patch").disabled).toBe(false);
    expect(screenText()).toContain("Patch applied");
    expectPatchResultBeforeFindings();
  });

  it("clears stale patch result when recommended patch fails", async () => {
    const invoke = vi.fn(async (command: string) => {
      if (command === "get_app_readiness" || command === "validate_api_key") {
        return readyState();
      }
      if (command === "analyze_prompt") {
        return analysisResult();
      }
      if (command === "apply_selected_fixes") {
        throw {
          code: "openai_incomplete",
          message: "OpenAI response was incomplete: max_output_tokens",
          hint: "Retry with a smaller patch or higher output cap.",
        };
      }
      throw new Error(command);
    }) as InvokeFunction;

    createPieApp(document.querySelector("#app") as HTMLElement, invoke, {
      readFileText: async () => assignmentPrompt,
    });

    await Promise.resolve();
    input("OpenAI API key").value = "sk-test";
    input("OpenAI API key").dispatchEvent(new Event("input", { bubbles: true }));
    await click("Validate Key");
    await uploadPrompt("assignment-agent-prompt.json");
    await click("Analyze Prompt");
    await click("Apply Recommended Patch");

    expect(screenText()).toContain("OpenAI response was incomplete");
    expect(screenText()).not.toContain("Applying recommended patch.");
    expect(document.querySelector(".updated-prompt")).toBeNull();
    expect(document.querySelector(".findings-table")).not.toBeNull();
  });

  it("applies recommended patch without verifier, downloads updated prompt, and supports history-aware reverify", async () => {
    const downloads: Array<{ filename: string; content: string }> = [];
    let resolveReverify: ((value: ReturnType<typeof reverifyResult>) => void) | undefined;
    const invoke = vi.fn(async (command: string) => {
      if (command === "get_app_readiness" || command === "validate_api_key") {
        return readyState();
      }
      if (command === "analyze_prompt" || command === "analyze_updated_prompt") {
        return analysisResult();
      }
      if (command === "apply_selected_fixes") {
        return patchResponse();
      }
      if (command === "verify_and_export_update") {
        throw new Error("verifier should not run during recommended patch");
      }
      if (command === "reverify_prompt") {
        return new Promise((resolve) => {
          resolveReverify = resolve;
        });
      }
      throw new Error(command);
    }) as InvokeFunction;

    createPieApp(document.querySelector("#app") as HTMLElement, invoke, {
      downloadText: (filename, content) => {
        downloads.push({ filename, content });
      },
      readFileText: async () => assignmentPrompt,
    });

    await Promise.resolve();
    input("OpenAI API key").value = "sk-test";
    input("OpenAI API key").dispatchEvent(new Event("input", { bubbles: true }));
    await click("Validate Key");
    await uploadPrompt("assignment-agent-prompt.json");
    await click("Analyze Prompt");
    expect(screenText()).toContain("assignment-agent-prompt_v20260615120000000");
    expect(button("Re-verify Updated Prompt").disabled).toBe(true);
    await click("Apply Recommended Patch");

    expect(invoke).not.toHaveBeenCalledWith("verify_and_export_update", expect.anything());
    expect(screenText()).toContain("Patch applied");
    expect(screenText()).toContain("updated_prompt_v20260615120000000");
    expect(button("Re-verify Updated Prompt").disabled).toBe(false);
    await click("Download Updated Prompt");
    expect(screenText()).toContain("Downloaded updated prompt to");
    button("Re-verify Updated Prompt").click();
    await Promise.resolve();

    expect(downloads[0].filename).toBe("updated_prompt_v20260615120000000.json");
    expect(invoke).not.toHaveBeenCalledWith("analyze_updated_prompt", expect.anything());
    expect(invoke).toHaveBeenCalledWith(
      "reverify_prompt",
      expect.objectContaining({
        request: expect.objectContaining({
          previous_finding_groups: analysisResult().finding_groups,
          updated_prompt_json: assignmentPrompt.replace("waitlist", "handoff"),
        }),
      }),
    );
    expect(screenText()).toContain("Re-verifying updated prompt");
    expect(screenText()).not.toContain("Hardcoded clinic facts");

    resolveReverify?.(reverifyResult());
    await Promise.resolve();
    await Promise.resolve();

    expect(document.querySelector(".findings-table")?.textContent).toContain("Reverify Status");
    expect(screenText()).toContain("Fixed");
    expect(screenText()).toContain("Still failing");
    expect(document.querySelector(".status-fixed")?.textContent).toContain("Fixed");
    expect(document.querySelector(".status-still-failing")?.textContent).toContain(
      "Still failing",
    );
  });
});

type ApplySelectedFixesResponseFixture = {
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
};

function readyState() {
  return {
    api_key_ready: true,
    storage_ready: true,
    analyze_enabled: true,
    model_label: "Model: gpt-5.4-mini",
    readiness_message: "Ready to analyze prompt.",
  };
}

function analysisResult() {
  return {
    prompt_version_name: "assignment-agent-prompt_v20260615120000000",
    model_label: "Model: gpt-5.4-mini",
    normalized_prompt: { raw_text: "raw", sections: [] },
    action_log: ["Stored prompt version assignment-agent-prompt_v20260615120000000"],
    finding_groups: [
      group(
        "Structure",
        "finding-structure-dynamic-provider-facts",
        "AutoFixable",
        "Hardcoded clinic facts and provider schedules should be config-driven",
      ),
      group(
        "ToolGaps",
        "finding-tool-waitlist",
        "HumanReviewOnly",
        "Waitlist and note-taking promises lack supporting tools",
        "Move time-sensitive provider facts out of static prompt prose.",
      ),
      group(
        "ToolGaps",
        "finding-tool-interpreter",
        "AutoFixable",
        "Interpreter arrangement is promised without an operational tool",
        "Check provider availability before making changes.",
      ),
      group(
        "WorkflowOrder",
        "finding-workflow-reschedule-cancels-first",
        "AutoFixable",
        "Reschedule flow cancels the old visit before the new slot is secured",
      ),
      group(
        "SafetyPhi",
        "finding-safety-clinical-referral-transfer",
        "HumanReviewOnly",
        "Clinical and referral issues are over-bundled into a generic transfer",
      ),
      group("Verification", "finding-verification-scenarios-missing", "Backlog"),
    ],
  };
}

function patchResponse(): ApplySelectedFixesResponseFixture {
  return {
    result: {
      candidate: {
        updated_prompt_json: assignmentPrompt.replace("waitlist", "handoff"),
        applied_finding_ids: [
          "finding-structure-dynamic-provider-facts",
          "finding-tool-waitlist",
          "finding-workflow-reschedule-cancels-first",
          "finding-safety-family-member-verification",
        ],
        patch_summary: "Applied recommended prompt patch.",
      },
      model_label: "Model: gpt-5.4-mini",
    },
    updated_version_name: "updated_prompt_v20260615120000000",
    action_log: ["Stored updated prompt version updated_prompt_v20260615120000000"],
  };
}

function reverifyResult() {
  return {
    prompt_version_name: "updated_prompt_v20260615120000000",
    model_label: "Model: gpt-5.4-mini",
    finding_statuses: [
      {
        finding_id: "finding-tool-waitlist",
        status: "Fixed",
        status_label: "Fixed",
        rationale: "The updated prompt no longer promises a waitlist action.",
      },
      {
        finding_id: "finding-workflow-reschedule-cancels-first",
        status: "StillFailing",
        status_label: "Still failing",
        rationale: "The updated prompt still allows cancellation before replacement booking.",
      },
    ],
    action_log: ["Compared updated prompt against previous findings."],
  };
}

function group(
  section: string,
  findingId: string,
  fixMode = "AutoFixable",
  title = `Title for ${findingId}`,
  suggestedFix = "minimal fix",
) {
  return {
    section,
    findings: [
      {
        finding_id: findingId,
        title,
        severity: "High",
        prompt_evidence: "prompt evidence",
        impact: "real call impact",
        suggested_fix: suggestedFix,
        fix_mode: fixMode,
        verification_scenario: "scenario",
      },
    ],
  };
}

async function click(label: string) {
  button(label).click();
  await Promise.resolve();
  await Promise.resolve();
}

async function uploadPrompt(filename: string) {
  const fileInput = document.querySelector<HTMLInputElement>("#prompt-file");
  if (!fileInput) throw new Error("prompt input missing");
  Object.defineProperty(fileInput, "files", {
    value: [new File([assignmentPrompt], filename, { type: "application/json" })],
    configurable: true,
  });
  fileInput.dispatchEvent(new Event("change", { bubbles: true }));
  await Promise.resolve();
}

function button(label: string): HTMLButtonElement {
  const match = [...document.querySelectorAll("button")].find(
    (candidate) => candidate.textContent?.trim() === label,
  );
  if (!match) throw new Error(`button not found: ${label}`);
  return match as HTMLButtonElement;
}

function input(label: string): HTMLInputElement {
  const match = [...document.querySelectorAll("label")].find((candidate) =>
    candidate.textContent?.includes(label),
  );
  const control = match?.querySelector("input");
  if (!control) throw new Error(`input not found: ${label}`);
  return control;
}

function screenText() {
  return document.body.textContent ?? "";
}

function findingActionLabels() {
  return [...document.querySelectorAll(".finding-actions button")].map(
    (candidate) => candidate.textContent?.trim() ?? "",
  );
}

function expectPatchResultBeforeFindings() {
  const patchResult = document.querySelector(".updated-prompt");
  const findings = document.querySelector(".findings");

  expect(patchResult).not.toBeNull();
  expect(findings).not.toBeNull();
  expect(patchResult?.compareDocumentPosition(findings as Node)).toBe(
    Node.DOCUMENT_POSITION_FOLLOWING,
  );
}
