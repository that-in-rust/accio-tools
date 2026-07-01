import { spawn } from "node:child_process";
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const scriptFilePath = fileURLToPath(import.meta.url);
const scriptsDirectoryPath = path.dirname(scriptFilePath);
const uiRootDirectoryPath = path.resolve(scriptsDirectoryPath, "..");
const solutionRootDirectoryPath = path.resolve(uiRootDirectoryPath, "..");
const outputDirectoryPath = path.join(
  solutionRootDirectoryPath,
  "target",
  "layout-check",
);
const harnessFileName = ".responsive-layout-check.html";
const harnessFilePath = path.join(uiRootDirectoryPath, harnessFileName);
const serverPortNumber = 1441;
const serverBaseUrl = `http://127.0.0.1:${serverPortNumber}`;

async function runResponsiveLayoutCheck() {
  await fs.mkdir(outputDirectoryPath, { recursive: true });
  await fs.writeFile(harnessFilePath, createHarnessHtmlText(), "utf8");

  const serverProcess = spawn(
    process.execPath,
    [
      path.join(uiRootDirectoryPath, "node_modules", "vite", "bin", "vite.js"),
      "--host",
      "127.0.0.1",
      "--port",
      String(serverPortNumber),
    ],
    {
      cwd: uiRootDirectoryPath,
      stdio: ["ignore", "pipe", "pipe"],
    },
  );

  try {
    await waitForServerReady(serverProcess);
    const results = await captureViewportProofs();
    const reportPath = path.join(outputDirectoryPath, "responsive-layout-report.json");
    await fs.writeFile(reportPath, JSON.stringify(results, null, 2), "utf8");
    console.log(`Responsive layout report: ${reportPath}`);
    for (const result of results) {
      console.log(
        `${result.viewport.name}: width=${result.viewport.width}, screenshot=${result.screenshotPath}, overflow=${result.audit.overflowCount}, overlaps=${result.audit.overlapCount}, comparisonRows=${result.audit.comparisonRowCount}, advancedOpen=${result.audit.advancedEvidenceOpen}`,
      );
    }

    const failed = results.some(
      (result) =>
        result.audit.documentScrollWidth > result.viewport.width + 1 ||
        result.audit.bodyScrollWidth > result.viewport.width + 1 ||
        result.audit.overflowCount > 0 ||
        result.audit.overlapCount > 0 ||
        result.audit.comparisonRowCount !== 3 ||
        result.audit.candidateCount !== 5,
    );
    if (failed) {
      throw new Error(`Responsive layout check failed. See ${reportPath}`);
    }
  } finally {
    serverProcess.kill("SIGTERM");
    await fs.rm(harnessFilePath, { force: true });
  }
}

async function waitForServerReady(serverProcess) {
  let stderrText = "";
  serverProcess.stderr.on("data", (chunk) => {
    stderrText += chunk.toString();
  });

  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    if (serverProcess.exitCode !== null) {
      throw new Error(`Vite exited early: ${stderrText}`);
    }
    try {
      const response = await fetch(`${serverBaseUrl}/${harnessFileName}`);
      if (response.ok) return;
    } catch {
      // Keep polling until Vite starts listening.
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(`Timed out waiting for Vite on ${serverBaseUrl}: ${stderrText}`);
}

async function captureViewportProofs() {
  const browser = await chromium.launch({ headless: true });
  const viewports = [
    { width: 390, height: 1200, name: "mobile" },
    { width: 1200, height: 1000, name: "desktop" },
  ];
  const results = [];

  try {
    for (const viewport of viewports) {
      const page = await browser.newPage({ viewport });
      await page.goto(`${serverBaseUrl}/${harnessFileName}`, {
        waitUntil: "domcontentloaded",
      });
      await page.waitForFunction(() =>
        document.body.textContent?.includes("Tool Router Evidence Console"),
      );
      await page.getByRole("button", { name: /Run Routing Comparison/ }).click();
      await page.waitForFunction(
        () => document.querySelectorAll(".query-comparison-table tbody tr").length === 3,
      );
      const screenshotPath = path.join(
        outputDirectoryPath,
        `${viewport.name}-${viewport.width}.png`,
      );
      await page.screenshot({ path: screenshotPath, fullPage: true });
      results.push({
        viewport,
        screenshotPath,
        audit: await auditViewportLayoutState(page),
      });
      await page.close();
    }
  } finally {
    await browser.close();
  }

  return results;
}

async function auditViewportLayoutState(page) {
  return page.evaluate(() => {
    function rectangleOverlapArea(left, right) {
      return (
        Math.max(0, Math.min(left.right, right.right) - Math.max(left.left, right.left)) *
        Math.max(0, Math.min(left.bottom, right.bottom) - Math.max(left.top, right.top))
      );
    }

    function isHiddenByClosedDetails(element) {
      const details = element.closest("details");
      if (!details || details.open) return false;
      return element.tagName !== "SUMMARY" && !element.closest("summary");
    }

    const viewportWidth = window.innerWidth;
    const allElements = [...document.querySelectorAll("body *")].filter(
      (element) => !isHiddenByClosedDetails(element),
    );
    const overflowElements = allElements.flatMap((element) => {
      if (element.closest(".comparison-table-wrap")) return [];
      const style = getComputedStyle(element);
      const rect = element.getBoundingClientRect();
      const horizontalScrollAllowed =
        style.overflowX === "auto" || style.overflowX === "scroll";
      const boxOutsideViewport =
        rect.width > 0 && (rect.left < -1 || rect.right > viewportWidth + 1);
      const ownOverflow =
        !horizontalScrollAllowed && element.scrollWidth > element.clientWidth + 1;
      if (!boxOutsideViewport && !ownOverflow) return [];
      return [
        {
          tag: element.tagName,
          className: String(element.className),
          id: element.id,
          text: (element.textContent || "").trim().slice(0, 80),
          left: rect.left,
          right: rect.right,
          scrollWidth: element.scrollWidth,
          clientWidth: element.clientWidth,
          overflowX: style.overflowX,
          boxOutsideViewport,
          ownOverflow,
        },
      ];
    });

    const layoutTargets = allElements
      .filter((element) =>
        element.matches(
          "button,input,select,textarea,.candidate-card,.route-progress-stage,.metric-grid > div,.activity-log li",
        ),
      )
      .map((element, index) => ({
        index,
        tag: element.tagName,
        className: String(element.className),
        id: element.id,
        text: (element.textContent || "").trim().slice(0, 60),
        rect: element.getBoundingClientRect(),
      }))
      .filter((item) => item.rect.width > 0 && item.rect.height > 0);

    const overlaps = [];
    for (let leftIndex = 0; leftIndex < layoutTargets.length; leftIndex += 1) {
      for (
        let rightIndex = leftIndex + 1;
        rightIndex < layoutTargets.length;
        rightIndex += 1
      ) {
        const left = layoutTargets[leftIndex];
        const right = layoutTargets[rightIndex];
        const overlap = rectangleOverlapArea(left.rect, right.rect);
        const smallerArea = Math.min(
          left.rect.width * left.rect.height,
          right.rect.width * right.rect.height,
        );
        if (overlap > 1 && overlap / smallerArea > 0.2) {
          overlaps.push({ left, right, overlap });
        }
      }
    }

    return {
      viewportWidth,
      bodyScrollWidth: document.body.scrollWidth,
      documentScrollWidth: document.documentElement.scrollWidth,
      overflowCount: overflowElements.length,
      overflowElements: overflowElements.slice(0, 20),
      overlapCount: overlaps.length,
      overlaps: overlaps.slice(0, 10),
      comparisonRowCount: document.querySelectorAll(".query-comparison-table tbody tr").length,
      advancedEvidenceOpen:
        document.querySelector('[data-testid="advanced-evidence"]')?.hasAttribute("open") ??
        false,
      candidateCount: document.querySelectorAll('[data-testid="candidate-card"]').length,
    };
  });
}

function createHarnessHtmlText() {
  return `<!doctype html>
<html>
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Router Layout Check</title>
  <link rel="stylesheet" href="/src/styles.css" />
</head>
<body>
  <main id="app"></main>
  <script type="module">
    import { createRouterWorkbenchApp } from "/src/app.ts";
    const tools = Array.from({ length: 947 }, (_, index) => createFixtureToolRecord(index));
    const queries = Array.from({ length: 50 }, (_, index) => createFixtureQueryRecord(index));
    const packFiles = [
      { filename: "tools.json", content: JSON.stringify(tools) },
      { filename: "queries.json", content: JSON.stringify(queries) },
      { filename: "manifest.json", content: JSON.stringify({ name: "layout-check", tool_count: 947, query_count: 50 }) },
    ];
    const invoke = async (command, args) => {
      if (command === "download_evaluation_pack_files") return packFiles;
      if (command === "run_cpu_preview_only") return createRouteResponseData("cpu_only_debug_preview", false);
      if (command === "validate_judge_api_key") return createReadyStateData();
      if (command === "route_tools_for_query") return createRouteResponseData(args?.request?.api_key ? "judged_route" : "cpu_only_debug_preview", true);
      if (command === "evaluate_routing_subset_metrics") return createMetricsReportData("lexical");
      if (command === "compare_routing_modes_metrics") return [
        createMetricsReportData("lexical"),
        { ...createMetricsReportData("schema_aware"), recall_at_k: { 1: 0.44, 3: 0.6, 5: 0.66, 10: 0.79 } },
        { ...createMetricsReportData("hybrid"), recall_at_k: { 1: 0.46, 3: 0.62, 5: 0.68, 10: 0.8 } },
      ];
      if (command === "export_route_evidence_report") return "# Tool Router Evidence Report";
      if (command === "export_diagnostic_logs_text") return "# Tool Router Diagnostic Log";
      throw new Error(command);
    };
    createRouterWorkbenchApp(document.querySelector("#app"), invoke, { downloadText: () => {} });

    function createFixtureToolRecord(index) {
      return {
        id: index === 0 ? "calendar.search_availability" : \`tool.\${index}\`,
        source_tool_id: \`source.tool.\${index}\`,
        server_id: \`server.\${index % 7}\`,
        server_name: index % 2 === 0 ? "Calendar Operations" : "Reference Catalog",
        name: index === 0 ? "calendar_search_availability" : \`tool_\${index}\`,
        description: index === 0 ? "Find calendar availability for follow-up visits and scheduling." : "Reference tool used as a distractor.",
        input_schema: { type: "object", properties: { input: { type: "string" } } },
        tags: ["calendar", "routing"],
        source: { repo: "fixture", path: "visual-check" },
        metadata: {},
      };
    }

    function createFixtureQueryRecord(index) {
      return {
        id: \`query-\${String(index).padStart(2, "0")}\`,
        query: \`Find nearby calendar availability for a follow-up visit \${index}\`,
        required_tool_ids: index < 46 ? ["calendar.search_availability"] : [],
        should_route: index < 46,
        graded_relevance: [{ tool_id: "calendar.search_availability", relevance: 3 }],
        source_expected_tools: ["calendar.search_availability"],
        failure_modes: ["confuse search with booking"],
      };
    }

    function createRouteResponseData(routeLabel, judged) {
      return {
        route_label: routeLabel,
        candidates: Array.from({ length: 5 }, (_, index) => createCandidateCardData(index)),
        judge_decision: judged
          ? {
              decision: "select_tool",
              selected_tool_id: "calendar.search_availability",
              confidence: 0.91,
              reason: "mock judge selected the strongest candidate with safe read-only evidence",
              needs_more_metadata: false,
            }
          : null,
      };
    }

    function createCandidateCardData(index) {
      return {
        rank: index + 1,
        score: 12.3456 - index,
        tool_id: index === 0 ? "calendar.search_availability" : \`calendar.candidate_\${index}\`,
        matched_terms: ["calendar", "availability", "follow", "visit", "nearby", "search", "routing", "evidence"],
        matched_fields: ["name", "description", "input_schema", "server_name"],
        capability_match: ["calendar", "read", "parameter"],
        risk: index === 0 ? "low" : "medium ambiguity around write exposure",
        why_matched: "Matched calendar availability language and compact schema evidence without using benchmark labels.",
        signal_contributions: { lexical: 12 - index, schema: 4 - index / 10 },
      };
    }

    function createMetricsReportData(routerMode) {
      return {
        queries: 50,
        route_required_queries: 46,
        abstention_queries: 4,
        recall_at_k: { 1: 0.42, 3: 0.58, 5: 0.6493, 10: 0.78 },
        mrr: 0.5223,
        ndcg_at_10: 0.5553,
        abstention_accuracy: 0.25,
        judged_route_accuracy: 0.5,
        failure_bucket_counts: {
          none: 19,
          wrong_llm_top1: 27,
          abstention_miss: 4,
        },
        average_selected_candidate_count: 5,
        token_reduction_estimate: 0.9894,
        router_mode: routerMode,
      };
    }

    function createReadyStateData() {
      return {
        judge_key_ready: true,
        route_preview_enabled: true,
        judged_route_enabled: true,
        model_label: "mock-router-judge",
        readiness_message: "Judge key accepted.",
      };
    }
  </script>
</body>
</html>`;
}

runResponsiveLayoutCheck().catch((error) => {
  console.error(error);
  process.exit(1);
});
