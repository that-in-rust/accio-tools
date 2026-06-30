# MVP Router Executable Specs

## Product Thesis

v0.0.1 proves a bidirectional tool router:

```text
query + recent context + tool catalog
  -> CPU router ranks top 5 candidates
  -> cheap LLM judge reviews only those top 5 cards
  -> top 1 tool, confidence, reason, or abstain
  -> evidence report compares route against the 50-query benchmark subset
```

The MVP is not a production MCP gateway or a workflow planner. It is a correctness-first ranking and judgment loop with enough evidence to show why the approach works.

## Inputs Parsed

| input | decision |
| --- | --- |
| Feature outcome | Select one best tool or abstain from a catalog, using CPU top-5 ranking plus required cheap LLM judgment. |
| Actors and boundaries | Reviewer/FDE uses evidence console or CLI. Rust cargo workspace owns routing, judging, and eval. OpenAI key remains session-only. |
| Failure modes | Missing required tool in CPU top 5, wrong LLM top 1, unsafe write exposure, abstention miss, malformed catalog, malformed query file, judge API failure, benchmark gold leakage into router scoring. |
| Performance and reliability limits | CPU ranking p95 <= 250 ms for 1,000 tools and 50 benchmark queries on a developer laptop; eval command exits non-zero on malformed fixtures. |
| Language/runtime constraints | Rust cargo workspace plus existing Tauri/Vite UI in `A02-routers-solutions/solution01`; bundled evaluation pack loads from `A00-raw-research/benchmarks/tool-routing-subset`; CPU-only tests run without network or OpenAI key. |

## Workspace Shape

The checked-in app currently starts from the Confido/PIE Tauri shell:

```text
solution01/
  Cargo.toml              # current package: pie-tauri-core; workspace members: "." and "src-tauri"
  ui/src/app.ts           # current exported app: createPieApp
  src-tauri/src/commands.rs
                           # current commands: validate_api_key, analyze_prompt, apply_selected_fixes, reverify_prompt
```

v0.0.1 SHALL migrate that shell into this target shape:

```text
solution01/
  Cargo.toml
  crates/
    catalog-router-core-engine/       # catalog model, lexical BM25, schema scoring, hybrid RRF, evidence cards
    candidate-judge-openai-adapter/   # judge trait, mock judge, OpenAI judge adapter
    benchmark-eval-metrics-runner/    # 50-query benchmark loader and metrics
    router-cli-command-surface/        # route and eval commands
```

New Rust functions and package names should follow four-word naming:

- `rank_lexical_tools_baseline`
- `score_schema_capability_signals`
- `fuse_hybrid_rankings_rrf`
- `judge_candidate_tools_top`
- `evaluate_routing_subset_metrics`
- `validate_catalog_schema_input`
- `load_bundled_evaluation_pack`
- `score_benchmark_route_outcome`
- `migrate_pie_workspace_shell`

Existing Tauri/Vite shell migration should use four-word names for new UI symbols:

- `createRouterWorkbenchApp`
- `RouterWorkbenchStateData`
- `loadCatalogInputSource`
- `selectRouterModeOption`
- `routeToolsForQuery`
- `runCpuPreviewOnly`
- `selectQuerySourceOption`
- `downloadEvaluationPackFiles`
- `renderCandidateEvidenceCards`
- `exportRouteEvidenceReport`
- `evaluateRoutingSubsetMetrics`

## Existing UI Base

The current `solution01` app already has a Confido/PIE-style Tauri shell:

```text
solution01/
  ui/src/app.ts          # current PIE workbench state and render loop
  ui/src/types.ts        # current prompt/finding/patch types
  ui/src/app.test.ts     # current PIE journey tests
  ui/src/styles.css      # current workbench styling
  src-tauri/             # Tauri wrapper and command bridge
  src/                   # current PIE Rust core
```

v0.0.1 SHALL migrate this shell instead of rebuilding the UI from scratch. The key setup card, progress strip, activity log, diagnostics export, and report download mechanics should remain; prompt-specific analysis, patching, and reverification surfaces should be removed or replaced.

## Executable Requirements

### REQ-MIG-001.0: Migrate existing shell

**WHEN** v0.0.1 implementation starts from the checked-in `solution01` app
**THEN** the system SHALL migrate the current `pie-tauri-core` workspace, `createPieApp` UI entrypoint, and PIE Tauri commands into router-named workspace, UI, and command surfaces
**AND** SHALL preserve reusable session-key, progress, activity-log, diagnostics, and download affordances
**SHALL** remove prompt analysis, patch, version-store, and reverify behavior from user-facing router flows.

### REQ-MVP-001.0: Build cargo workspace

**WHEN** a developer runs `cargo test --workspace`
**THEN** the system SHALL build and test a Rust cargo workspace with `catalog-router-core-engine`, `candidate-judge-openai-adapter`, `benchmark-eval-metrics-runner`, and `router-cli-command-surface`
**AND** SHALL run all CPU-only tests without an OpenAI key
**SHALL** expose typed errors for malformed catalog, malformed query file, missing dataset path, and judge configuration failure.

### REQ-MVP-002.0: Validate catalog metadata

**WHEN** the router loads `tools.json` or an uploaded catalog JSON
**THEN** the system SHALL validate every tool has a stable id, name, description, source/server metadata when present, and JSON-schema-compatible input metadata
**AND** SHALL reject duplicate tool ids with a typed validation error
**SHALL** preserve unknown metadata fields for evidence reporting.

### REQ-MVP-003.0: Rank lexical baseline candidates

**WHEN** lexical BM25 mode receives a query and valid catalog
**THEN** the system SHALL return a deterministic ranked candidate list ordered by lexical score
**AND** SHALL score name, description, server/source names, tags, and flattened input schema text
**SHALL** include `rank`, `score`, `tool_id`, `matched_terms`, and `why_matched` for each returned candidate.

### REQ-MVP-004.0: Rank schema-aware candidates

**WHEN** schema-aware BM25 mode receives a query and valid catalog
**THEN** the system SHALL combine lexical score with operation, object, parameter, namespace, and read/write capability signals
**AND** SHALL return top 5 candidates with `matched_fields`, `capability_match`, `risk`, and `why_matched`
**SHALL** avoid hardcoded domain/server routing rules in the scoring function.

### REQ-MVP-005.0: Rank hybrid fused candidates

**WHEN** hybrid RRF mode receives ranked lexical and schema-aware lists
**THEN** the system SHALL produce one deterministic fused ranking with weighted reciprocal rank fusion
**AND** SHALL include per-signal contributions in each candidate evidence card
**SHALL** run without vector search by default and only include vector contribution when a configured vector provider or fixture exists.

### REQ-MVP-006.0: Judge top five candidates

**WHEN** production route mode runs with a validated OpenAI API key
**THEN** the cheap LLM judge SHALL receive only the CPU top 5 compact candidate cards, the user query, and minimal routing instructions
**AND** SHALL return `selected_tool_id`, `confidence`, `reason`, `decision`, and `needs_more_metadata`
**SHALL** never receive the full catalog.

### REQ-MVP-007.0: Handle missing judge key

**WHEN** no validated OpenAI API key is available
**THEN** the UI and CLI SHALL allow `Run CPU Preview` and label the result as `cpu_only_debug_preview`
**AND** SHALL show CPU top 5 ranking without claiming production top-1 judgment
**SHALL** keep `Run Judged Route` and production route export disabled until the judge is available.

### REQ-MVP-008.0: Abstain safely

**WHEN** the CPU ranking has weak evidence or the LLM judge determines no candidate satisfies the query
**THEN** the system SHALL return an abstain decision with reason and confidence
**AND** SHALL count abstention decisions in benchmark metrics
**SHALL** prefer abstention over unsafe write exposure when write intent is ambiguous.

### REQ-MVP-009.0: Evaluate benchmark subset

**WHEN** a developer runs eval against `A00-raw-research/benchmarks/tool-routing-subset`
**THEN** the system SHALL report Recall@1, Recall@3, Recall@5, Recall@10, MRR, nDCG@10 where graded labels exist, abstention accuracy, average selected candidate count, and token reduction estimate
**AND** SHALL compare lexical BM25, schema-aware BM25, and hybrid RRF modes
**AND** SHALL write JSON and Markdown reports
**SHALL** read `tools.json`, `queries.json`, and `manifest.json` without requiring any external benchmark checkout.

### REQ-MVP-010.0: Protect secrets and payloads

**WHEN** a judge request is executed or exported
**THEN** the system SHALL redact API keys and omit raw secret-bearing environment values from logs
**AND** SHALL store only route evidence, aggregate metrics, and sanitized judge reasons
**SHALL** keep the API key in session memory only.

### REQ-MVP-011.0: Export evidence report

**WHEN** the user exports a route or benchmark run
**THEN** the report SHALL include query, router mode, catalog stats, CPU top 5 candidates, judge top 1 decision, confidence, reasons, benchmark gold match status when present, metrics, and failure bucket
**AND** SHALL mark CPU-only output as debug preview when the judge was unavailable
**SHALL** be reproducible from the saved input fixture and router configuration.

### REQ-MVP-012.0: Restrict runtime modes

**WHEN** v0.0.1 router modes are listed in CLI, UI, docs, or tests
**THEN** the available CPU modes SHALL be exactly lexical BM25, schema-aware BM25, and hybrid RRF
**AND** SHALL reject any unsupported runtime mode with a typed validation error
**SHALL** keep the mode list identical across CLI, library, report metadata, and evidence console.

### REQ-DATA-001.0: Bundle evaluation pack

**WHEN** the app or CLI starts with no custom dataset supplied
**THEN** the system SHALL expose the bundled evaluation pack from `A00-raw-research/benchmarks/tool-routing-subset`
**AND** SHALL load `tools.json`, `queries.json`, and `manifest.json` as the default demo dataset
**SHALL** allow downloading the bundled pack as inspectable JSON files or one archive.

### REQ-DATA-002.0: Isolate benchmark labels

**WHEN** a bundled benchmark query is routed
**THEN** the router SHALL receive query text, optional recent context, selected router mode, and tool catalog metadata only
**AND** SHALL keep `required_tool_ids`, `should_route`, `graded_relevance`, `source_expected_tools`, and `failure_modes` inside evaluator/reporting code
**SHALL** fail a test if CPU ranking or LLM judge payloads contain benchmark gold labels.

### REQ-DATA-003.0: Normalize query records

**WHEN** `queries.json` is loaded
**THEN** the system SHALL parse `id`, `query`, `required_tool_ids`, `should_route`, `graded_relevance`, `source_expected_tools`, and `failure_modes`
**AND** SHALL treat `should_route=false` rows as abstention gold cases
**SHALL** return a typed validation error for missing `id`, missing `query`, or invalid `required_tool_ids` values.

### REQ-UI-001.0: Migrate product identity

**WHEN** the Tauri/Vite app renders its initial shell
**THEN** the UI SHALL identify itself as `Tool Router Evidence Console`
**AND** SHALL not show user-facing `PIE`, `Prompt Iteration Engine`, healthcare voice-agent, prompt patch, or reverify copy
**SHALL** keep the existing session-key, workbench, activity-log, and export affordances.

### REQ-UI-002.0: Validate judge session key

**WHEN** the user enters an OpenAI API key and clicks validate
**THEN** the UI SHALL call `validate_judge_api_key`
**AND** SHALL show validating, accepted, failed, and missing-key states without persisting the raw key
**SHALL** disable judged production route execution until the key is accepted.

### REQ-UI-003.0: Load evaluation pack

**WHEN** the user opens the router workbench
**THEN** the UI SHALL default to the bundled evaluation pack with 947 tools and 50 queries
**AND** SHALL offer `Download Evaluation Pack`, custom catalog upload, and custom query-file upload
**SHALL** display pack stats after validation: total tools, query count, source count, schema count, route-required count, abstention count, and duplicate-id status.

### REQ-UI-004.0: Select query source

**WHEN** the router workbench is ready
**THEN** the UI SHALL default to a searchable bundled benchmark query picker and SHALL allow switching to custom query entry
**AND** SHALL show optional recent context input for both benchmark and custom query modes
**SHALL** show benchmark pass/fail metrics only when a bundled benchmark query or custom query file with labels is selected.

### REQ-UI-005.0: Select router mode

**WHEN** the user chooses a CPU router mode
**THEN** the UI SHALL expose exactly lexical BM25, schema-aware BM25, and hybrid RRF
**AND** SHALL send the selected mode to `route_tools_for_query`
**SHALL** reject or ignore any unsupported mode value in UI state.

### REQ-UI-006.0: Run judged route

**WHEN** the user clicks `Run Judged Route`
**THEN** the UI SHALL call `route_tools_for_query` with catalog source, query, recent context, router mode, and session judge key
**AND** SHALL show progress stages for catalog validation, CPU ranking, judge review, and evidence compilation
**SHALL** render a failed state if the judge API, catalog parser, or router engine returns a typed error.

### REQ-UI-013.0: Run CPU preview

**WHEN** catalog, query, and router mode are valid but no judge key is accepted
**THEN** the UI SHALL enable `Run CPU Preview` and call `run_cpu_preview_only`
**AND** SHALL show exactly five CPU candidate evidence cards with `cpu_only_debug_preview` labeling
**SHALL** disable judged top-1 claims and production export until `Run Judged Route` succeeds.

### REQ-UI-007.0: Render route evidence

**WHEN** a route result is returned
**THEN** the UI SHALL show judge decision, selected tool id, confidence, reason, and abstention state
**AND** SHALL show exactly five CPU candidate evidence cards with rank, score, matched fields, risk, and `why_matched`
**SHALL** mark CPU-only output as debug preview when no accepted judge key was used.

### REQ-UI-008.0: Render benchmark health

**WHEN** the active query has benchmark gold labels or the user runs subset evaluation
**THEN** the UI SHALL show Recall@1, Recall@3, Recall@5, Recall@10, MRR, abstention accuracy, token reduction estimate, and failure bucket when available
**AND** SHALL compare lexical BM25, schema-aware BM25, and hybrid RRF in an advanced/developer panel
**SHALL** score `should_route=true` as required-tool survival plus judged selection, and score `should_route=false` as abstention correctness.

### REQ-UI-009.0: Export route evidence

**WHEN** the user clicks export after a route or eval run
**THEN** the UI SHALL call `export_route_evidence_report`
**AND** SHALL download Markdown or JSON containing query, catalog stats, router mode, CPU top 5, judge top 1, confidence, reasons, metrics, and failure bucket
**SHALL** redact API keys and omit raw secret-bearing environment values from exported content.

### REQ-UI-010.0: Remove prompt patch flows

**WHEN** the migrated router UI is built or tested
**THEN** the UI SHALL not expose `Apply Recommended Patch`, `Updated Prompt`, `Re-verify Updated Prompt`, prompt findings, finding groups, selected finding ids, or prompt version downloads
**AND** SHALL remove or replace invoke paths for `analyze_prompt`, `apply_selected_fixes`, `reverify_prompt`, and `verify_and_export_update`
**SHALL** fail UI tests if prompt-patch controls reappear in user-facing markup.

### REQ-UI-011.0: Preserve activity diagnostics

**WHEN** readiness, catalog load, route execution, export, or failures occur
**THEN** the UI SHALL append concise activity log entries in chronological order
**AND** SHALL keep diagnostic log export available
**SHALL** include command names without raw API keys or full judge payload secrets.

### REQ-UI-012.0: Maintain operational layout

**WHEN** the UI is rendered at 390 px mobile width and 1200 px desktop width
**THEN** controls, cards, tables, progress indicators, and activity log text SHALL not overlap or overflow their containers
**AND** SHALL keep repeated candidate cards at stable heights with responsive wrapping
**SHALL** use a restrained operational palette without decorative radial backgrounds or marketing hero layout.

## Test Matrix

| req_id | test_id | type | assertion | target |
| --- | --- | --- | --- | --- |
| REQ-MIG-001.0 | TEST-MIG-001 | integration | current `pie-tauri-core` shell is migrated to router workspace and command names | workspace |
| REQ-MIG-001.0 | TEST-MIG-002 | component | reusable key, progress, activity, diagnostics, and download affordances remain visible | ui/src/app.test.ts |
| REQ-MVP-001.0 | TEST-WORK-001 | integration | `cargo test --workspace` succeeds without OpenAI key | workspace |
| REQ-MVP-001.0 | TEST-WORK-002 | unit | malformed fixture paths return typed errors | router-cli-command-surface |
| REQ-MVP-002.0 | TEST-CAT-001 | unit | duplicate tool ids are rejected | catalog-router-core-engine |
| REQ-MVP-002.0 | TEST-CAT-002 | unit | unknown catalog metadata is preserved | catalog-router-core-engine |
| REQ-MVP-003.0 | TEST-LEX-001 | unit | exact name and description matches rank above distractors | catalog-router-core-engine |
| REQ-MVP-003.0 | TEST-LEX-002 | unit | lexical candidates include matched terms and scores | catalog-router-core-engine |
| REQ-MVP-004.0 | TEST-SCH-001 | unit | required parameter match boosts ranking | catalog-router-core-engine |
| REQ-MVP-004.0 | TEST-SCH-002 | unit | read/write mismatch raises risk flag | catalog-router-core-engine |
| REQ-MVP-004.0 | TEST-SCH-003 | unit | scorer contains no hardcoded benchmark server switch | catalog-router-core-engine |
| REQ-MVP-005.0 | TEST-HYB-001 | unit | RRF produces deterministic fused ranking | catalog-router-core-engine |
| REQ-MVP-005.0 | TEST-HYB-002 | unit | vector contribution is absent when provider disabled | catalog-router-core-engine |
| REQ-MVP-006.0 | TEST-JDG-001 | integration | judge prompt includes only five candidate cards | candidate-judge-openai-adapter |
| REQ-MVP-006.0 | TEST-JDG-002 | unit | mock judge returns selected tool and confidence | candidate-judge-openai-adapter |
| REQ-MVP-007.0 | TEST-KEY-001 | integration | missing key produces `cpu_only_debug_preview` | router-cli-command-surface |
| REQ-MVP-008.0 | TEST-ABS-001 | unit | unsupported query can return abstain decision | catalog-router-core-engine |
| REQ-MVP-008.0 | TEST-ABS-002 | integration | abstention accuracy appears in eval report | benchmark-eval-metrics-runner |
| REQ-MVP-009.0 | TEST-EVL-001 | integration | eval emits Recall@K, MRR, nDCG, abstention metrics | benchmark-eval-metrics-runner |
| REQ-MVP-009.0 | TEST-EVL-002 | integration | eval compares lexical, schema-aware, and hybrid modes | benchmark-eval-metrics-runner |
| REQ-MVP-010.0 | TEST-SEC-001 | unit | exported report redacts API key shaped values | candidate-judge-openai-adapter |
| REQ-MVP-011.0 | TEST-REP-001 | integration | export contains CPU top 5, judge top 1, metrics, and failure bucket | router-cli-command-surface |
| REQ-MVP-012.0 | TEST-MODE-001 | unit | runtime mode enum contains exactly lexical, schema-aware, hybrid | catalog-router-core-engine |
| REQ-DATA-001.0 | TEST-DATA-001 | integration | default pack loads 947 tools, 50 queries, and manifest metadata | benchmark-eval-metrics-runner |
| REQ-DATA-001.0 | TEST-DATA-002 | component | Download Evaluation Pack emits inspectable JSON files or archive | ui/src/app.test.ts |
| REQ-DATA-002.0 | TEST-DATA-003 | unit | router and judge payloads exclude benchmark gold fields | catalog-router-core-engine |
| REQ-DATA-003.0 | TEST-DATA-004 | unit | query loader parses `required_tool_ids`, `should_route`, `graded_relevance`, `source_expected_tools`, and `failure_modes` | benchmark-eval-metrics-runner |
| REQ-DATA-003.0 | TEST-DATA-005 | unit | `should_route=false` rows are scored as abstention gold cases | benchmark-eval-metrics-runner |
| REQ-UI-001.0 | TEST-UI-001 | component | shell renders Tool Router Evidence Console and no PIE/prompt-patch copy | ui/src/app.test.ts |
| REQ-UI-002.0 | TEST-UI-002 | component | key validation calls `validate_judge_api_key` and gates route execution | ui/src/app.test.ts |
| REQ-UI-003.0 | TEST-UI-003 | component | bundled pack defaults, custom uploads, download action, and pack stats render | ui/src/app.test.ts |
| REQ-UI-004.0 | TEST-UI-004 | component | benchmark picker is default and custom query entry is available | ui/src/app.test.ts |
| REQ-UI-005.0 | TEST-UI-005 | component | router mode selector exposes exactly lexical, schema-aware, hybrid | ui/src/app.test.ts |
| REQ-UI-006.0 | TEST-UI-006 | integration | Run Judged Route calls `route_tools_for_query` and shows progress stages | ui/src/app.test.ts |
| REQ-UI-013.0 | TEST-UI-013 | integration | Run CPU Preview calls `run_cpu_preview_only` without judge key and marks debug output | ui/src/app.test.ts |
| REQ-UI-007.0 | TEST-UI-007 | component | result renders judge decision and exactly five evidence cards | ui/src/app.test.ts |
| REQ-UI-008.0 | TEST-UI-008 | component | benchmark health scores `should_route=true` and `should_route=false` cases correctly | ui/src/app.test.ts |
| REQ-UI-009.0 | TEST-UI-009 | integration | export calls `export_route_evidence_report` and downloads redacted report | ui/src/app.test.ts |
| REQ-UI-010.0 | TEST-UI-010 | component | patch/reverify controls and invoke paths are absent from router UI | ui/src/app.test.ts |
| REQ-UI-011.0 | TEST-UI-011 | component | activity log records readiness, catalog, route, export, and failure events | ui/src/app.test.ts |
| REQ-UI-012.0 | TEST-UI-012 | visual | mobile and desktop layout snapshots have no overflow or overlap | Playwright/visual check |

## TDD Plan

### 1. STUB

- Create cargo workspace and crate shells.
- Add migration tests proving the current PIE shell is intentionally replaced, not ignored.
- Add failing tests for catalog validation, lexical ranking, schema scoring, hybrid fusion, judge payload shape, eval metrics, and runtime mode list.
- Replace PIE UI test stubs with router workbench tests for identity, key readiness, bundled evaluation pack, query source selection, CPU preview, judged route, export, and removed patch flows.
- Add fixture loading tests for `tools.json`, `queries.json`, `manifest.json`, `required_tool_ids`, `should_route`, `graded_relevance`, `source_expected_tools`, and `failure_modes`.

### 2. RED

- Run `cargo test --workspace`.
- Run `npm --prefix ui test`.
- Expected failures: missing catalog types, missing router mode enum, missing scorer functions, missing judge trait, missing eval runner.
- Expected UI failures: old PIE copy still visible, prompt-finding types still referenced, query picker missing, CPU preview action missing, route commands missing, candidate evidence cards missing.
- Record the first failure per requirement ID before implementation.

### 3. GREEN

- Implement `ToolCatalogRecordData`, `RouteQueryInputData`, `CandidateEvidenceCardData`, `JudgeDecisionOutputData`, and `RouterTypedErrorKind`.
- Implement `rank_lexical_tools_baseline` to match the existing Python baseline behavior closely enough for fixture parity.
- Implement `score_schema_capability_signals` with deterministic weights and evidence output.
- Implement `fuse_hybrid_rankings_rrf` using lexical and schema ranks first; leave vector disabled unless configured.
- Implement mock judge and payload-shape enforcement before real OpenAI adapter.
- Implement eval metrics and report writing.
- Implement `load_bundled_evaluation_pack`, `score_benchmark_route_outcome`, and gold-label isolation before UI wiring.
- Implement `createRouterWorkbenchApp`, `RouterWorkbenchStateData`, `loadCatalogInputSource`, `selectQuerySourceOption`, `selectRouterModeOption`, `runCpuPreviewOnly`, `routeToolsForQuery`, `renderCandidateEvidenceCards`, `downloadEvaluationPackFiles`, `exportRouteEvidenceReport`, and `evaluateRoutingSubsetMetrics` in the Tauri/Vite shell.
- Replace prompt/finding/patch TypeScript types with catalog, query, candidate, judge, metric, and export types.
- Replace Tauri commands with `validate_judge_api_key`, `run_cpu_preview_only`, `route_tools_for_query`, `evaluate_routing_subset_metrics`, `download_evaluation_pack_files`, `export_route_evidence_report`, and `export_diagnostic_logs_text`.

### 4. REFACTOR

- Normalize scorer interfaces behind a `RouteToolsForQuery` trait.
- Keep four-word naming for every new public implementation symbol.
- Split evidence-card construction from ranking math.
- Move benchmark fixture paths into CLI config and UI evaluation-pack config.
- Split UI rendering into four-word helpers for readiness, catalog, query, candidate cards, benchmark health, activity log, and export status.
- Remove prompt patch/reverify modules after equivalent router tests are green.

### 5. VERIFY

- Run full workspace tests and quality gates.
- Run UI unit tests and responsive layout checks.
- Run eval in CPU-only mode and compare lexical baseline against current Python baseline metrics.
- Verify benchmark dropdown, custom query mode, and CPU preview mode in UI tests.
- Run judge tests with mock judge; run real OpenAI smoke test only when key is explicitly supplied.

## Quality Gates

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `npm --prefix ui test`
- `npm --prefix ui run build`
- `npm --prefix ui run test:responsive-layout-viewports`
- `cargo run -p router-cli-command-surface -- evaluate-routing-subset-metrics --dataset ../../A00-raw-research/benchmarks/tool-routing-subset --mode lexical`
- `cargo run -p router-cli-command-surface -- evaluate-routing-subset-metrics --dataset ../../A00-raw-research/benchmarks/tool-routing-subset --mode schema-aware`
- `cargo run -p router-cli-command-surface -- evaluate-routing-subset-metrics --dataset ../../A00-raw-research/benchmarks/tool-routing-subset --mode hybrid`
- Every `REQ-*` ID has at least one linked test.
- No `TODO`, `STUB`, or `FIXME` introduced in implementation commits.
- No performance claim is accepted without an eval command or test.
- Runtime mode list contains only lexical BM25, schema-aware BM25, and hybrid RRF.
- Bundled evaluation pack loads 947 tools, 50 queries, 46 route-required cases, and 4 abstention cases.
- Router and judge payload tests prove benchmark gold labels are not used before evaluation.
- Judge payload snapshot proves the full catalog is not sent to the LLM judge.
- User-facing UI copy contains no `PIE`, `Prompt Iteration Engine`, prompt patch, updated prompt, or reverify controls.
- Responsive screenshots at 390 px and 1200 px show no text overlap, control overflow, or incoherent table/card wrapping.

## Resolved Defaults For Build

| decision | default | verification |
| --- | --- | --- |
| Judge model | Read `OPENAI_ROUTER_JUDGE_MODEL`; use mock judge for tests; real adapter requires explicit key and model. | Judge tests run offline; real smoke test is opt-in. |
| Schema weights | Operation `+2.0`, object `+2.0`, required parameter `+1.5`, optional parameter `+0.5`, namespace/server `+0.75`, read/write alignment `+1.25`, unsafe write mismatch `-3.0`. | Unit fixtures prove feature boosts and penalties change rank as expected. |
| Hybrid RRF | Use `score += weight / (60 + rank)` with lexical `1.0`, schema `1.4`, vector `0.6` only when enabled. | Fusion tests prove deterministic ordering and per-signal evidence. |
| Vector search | Disabled by default; optional fixture/provider only. | Hybrid tests pass with no vector provider. |
| MVP success gate | Build passes when all modes emit metrics; product claim requires schema-aware Recall@5 to beat lexical or a failure bucket report. | Eval report compares lexical, schema-aware, and hybrid modes. |
| Lexical parity gate | Rust lexical Recall@5 within `0.02` of Python `0.6493`; MRR within `0.03` of Python `0.5223`. | CPU eval compares against checked-in baseline numbers. |
| Bundled evaluation pack | Default UI and CLI dataset is `A00-raw-research/benchmarks/tool-routing-subset`, containing 947 tools, 50 queries, 46 route-required cases, and 4 abstention cases. | Loader tests verify counts and required query fields. |
| Query source default | Benchmark query picker is the happy path; custom query entry is the escape hatch. | UI tests prove benchmark picker appears first and custom mode is available. |
| No-key route action | Missing judge key enables `Run CPU Preview` but disables `Run Judged Route`. | UI and CLI tests prove CPU preview never claims production top 1. |
| UI boundary | Rust owns engine, CLI, judge adapter, and eval output; UI consumes CLI/library later. | `route-tools-for-query` and `evaluate-routing-subset-metrics` are stable enough for the evidence console. |
| UI migration base | Keep the Tauri/Vite shell but replace PIE prompt surfaces with router workbench surfaces. | UI tests prove no prompt patch/reverify controls remain. |

## Open Questions

| question | current stance | when to resolve |
| --- | --- | --- |
| Should the advanced benchmark comparison panel be always visible? | Keep it collapsed by default so the primary journey focuses on one judged route. | During UI implementation after first screenshot pass. |
| Should catalog upload accept JSON arrays and wrapper objects? | Support both if validation can normalize them into `ToolCatalogRecordData`. | During catalog parser implementation. |
| Should Download Evaluation Pack be separate JSON files or one archive? | Support either; prefer one archive if Tauri download mechanics make it simple. | During UI export implementation. |
| Should real OpenAI judge smoke tests run in CI? | No; keep mock judge in CI and make real smoke tests explicit local commands. | Before final demo packaging. |

## Rubber Duck Debugging

The core MVP is captured well: CPU ranks top 5, cheap LLM judge selects top 1 or abstains, and the benchmark subset proves the claim. The three CPU modes are correctly bounded: lexical BM25, schema-aware BM25, and hybrid RRF.

The main risk is target/current mismatch. Codebase-memory and source reads show the current app is still the PIE shell: `pie-tauri-core` workspace, `createPieApp`, `validate_api_key`, `analyze_prompt`, `apply_selected_fixes`, and `reverify_prompt`. Router symbols such as `route_tools_for_query`, `validate_judge_api_key`, `evaluate_routing_subset_metrics`, and `RouterWorkbench` are not implemented yet. REQ-MIG-001.0 exists to make that migration explicit.

The benchmark side is good. The bundled subset contains 947 tools, 50 queries, 46 route-required cases, and 4 abstention cases. The baseline is Recall@5 `0.6493`, MRR `0.5223`, and abstention accuracy `0.0`.

The biggest UX correction is query source. The happy path is a bundled benchmark query picker because it gives reviewers instant reproducible proof. Custom query entry remains available as an escape hatch, but it cannot show benchmark pass/fail unless labels are supplied.

The biggest route-state correction is key gating. No key means `Run CPU Preview` is allowed and clearly labeled `cpu_only_debug_preview`. A validated judge key unlocks `Run Judged Route`, which is the only path that may claim production top 1 or abstain.

Workflow planning is out of v0.0.1 because it changes the problem from ranking candidate tools to coordinating tool chains. That may become valuable later, but it would weaken the MVP by splitting evaluation between top-1 tool choice and workflow correctness.
