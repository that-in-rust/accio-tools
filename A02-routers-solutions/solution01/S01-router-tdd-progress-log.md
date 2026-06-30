# TDD Progress Journal

- Task: Implement router MVP executable specs
- Created: 2026-06-30 19:10:59Z
- Updated: 2026-06-30 20:04:27Z
- Current Phase: Refactor
- Status: active

## Sessions

### Session: 2026-06-30 19:11:15Z

#### Current Phase: Red

#### Tests Written:
- TEST-DATA-001: not written - default pack should load 947 tools and 50 queries
- TEST-LEX-001: not written - lexical baseline should rank exact name and description matches above distractors
- TEST-EVL-001: not written - eval should emit Recall@K, MRR, nDCG, abstention metrics

#### Implementation Progress:
- A02-routers-solutions/solution01: current workspace still pie-tauri-core with no router crates

#### Current Focus:
First router MVP slice: workspace crates, bundled evaluation pack loading, lexical ranking parity, and eval metrics.

#### Next Steps:
- Create router workspace crates and write failing Rust tests for data pack loading and lexical/eval parity.
- Port Python baseline tokenization, indexing, ranking, and metrics into catalog-router-core-engine plus benchmark-eval-metrics-runner.
- Run cargo test --workspace and record exact failures or green status.

#### Context Notes:
- Existing unrelated worktree deletions and Note-to-reviewer.md are not part of this implementation slice.

#### Performance/Metrics:
- Python baseline: Recall@5 0.6493, MRR 0.5223, abstention accuracy 0.0.

### Session: 2026-06-30 19:16:49Z

#### Current Phase: Red

#### Tests Written:
- cargo test --workspace: failing - src-tauri unresolved import benchmark_eval_metrics_runner

#### Implementation Progress:
- src-tauri/src/commands.rs now exposes router command names; src-tauri/Cargo.toml missing direct benchmark-eval-metrics-runner dependency.

#### Current Focus:
Fix first migration compile seam before behavior tests can run.

#### Next Steps:
- Add direct src-tauri dependency on benchmark-eval-metrics-runner.
- Re-run cargo test --workspace to reach behavioral red tests.
- Implement lexical ranking and metrics once compile seams are green.

#### Context Notes:
- This failure is expected from the package migration, not from router logic yet.

#### Performance/Metrics:
- Cargo failure: E0432 unresolved import benchmark_eval_metrics_runner.

### Session: 2026-06-30 19:17:14Z

#### Current Phase: Red

#### Tests Written:
- cargo test --workspace: failing - src-tauri/src/main.rs still references pie_tauri_app

#### Implementation Progress:
- src-tauri package renamed to tool-router-tauri-app; binary entrypoint not updated yet.

#### Current Focus:
Finish Rust package rename seam for Tauri binary.

#### Next Steps:
- Replace pie_tauri_app::run with tool_router_tauri_app::run.
- Re-run cargo test --workspace.
- Move to lexical ranking and metrics behavior failures.

#### Context Notes:
- Package rename is part of REQ-MIG-001.0 migration from PIE shell.

#### Performance/Metrics:
- Cargo failure: E0433 unresolved module pie_tauri_app.

### Session: 2026-06-30 19:20:00Z

#### Current Phase: Green

#### Tests Written:
- cargo test --workspace: passing - 3 benchmark tests, 2 core tests, 1 judge adapter test, and migrated workspace compile
- test_metrics_report_shape: passing - Recall@5 0.6493, MRR 0.5223, abstention accuracy 0.0

#### Implementation Progress:
- crates/catalog-router-core-engine: typed catalog/query models, validation, lexical ranking, schema scoring scaffold, hybrid RRF
- crates/benchmark-eval-metrics-runner: bundled pack loader and metrics parity implementation
- crates/candidate-judge-openai-adapter: five-card mock judge contract
- crates/router-cli-command-surface and src-tauri: router-shaped command surfaces compile

#### Current Focus:
First router MVP backbone slice is green: workspace crates, pack loading, lexical ranking, metrics, and Tauri command compile surface.

#### Next Steps:
- Run cargo fmt --all and cargo test --workspace again.
- Exercise router-cli-command-surface eval command for lexical/schema/hybrid modes.
- Start next TDD slice: frontend query source UI and typed invoke wrappers.

#### Context Notes:
- Old PIE modules remain on disk but are no longer compiled by the root Rust library; UI is still old and must be migrated next.

#### Performance/Metrics:
- cargo test --workspace passed in 12.24s. Lexical metrics match Python baseline: Recall@5 0.6493, MRR 0.5223.

### Session: 2026-06-30 19:24:16Z

#### Current Phase: Refactor

#### Tests Written:
- cargo fmt --all --check: passing - Rust formatting gate clean
- cargo test --workspace: passing - workspace tests and doctests pass
- cargo clippy --workspace --all-targets -- -D warnings: passing - Rust lint gate clean
- router-cli lexical/schema/hybrid: passing - all three eval modes emit metrics for bundled pack

#### Implementation Progress:
- CodeGraphContext: rank_lexical_tools_baseline found in catalog-router-core-engine; route_tools_for_query and run_cpu_preview_only found in root plus Tauri command surfaces.
- Schema-aware mode currently does not beat lexical Recall@5; metrics make this visible for future tuning/failure bucket reporting.

#### Current Focus:
Rust backbone slice verified; next slice is Tauri/Vite UI migration from PIE to Tool Router Evidence Console.

#### Next Steps:
- Replace ui/src/types.ts with router workbench types and typed command names.
- Replace ui/src/app.ts and app.test.ts with benchmark picker, custom query, CPU preview, judged route, metrics, and export flows.
- Run npm --prefix ui test and npm --prefix ui run build.

#### Context Notes:
- Old PIE source files remain on disk but are not compiled by Rust root; frontend still exposes PIE and must be migrated next.

#### Performance/Metrics:
- CLI lexical Recall@5 0.6493/MRR 0.5223; schema Recall@5 0.6275/MRR 0.5258; hybrid Recall@5 0.6493/MRR 0.5229.

### Session: 2026-06-30 19:36:18Z

#### Current Phase: Refactor

#### Tests Written:
- npm test: passing - 6 router workbench UI tests
- npm run build: passing - TypeScript and Vite production build
- cargo fmt --all --check: passing - Rust formatting gate clean
- cargo test --workspace: passing - 6 Rust unit tests plus doctests and Tauri compile surfaces
- cargo clippy --workspace --all-targets -- -D warnings: passing - Rust lint gate clean
- router-cli lexical smoke: passing - bundled 50-query metrics emitted

#### Implementation Progress:
- ui/src/app.ts now exposes router workbench flows instead of PIE prompt analysis.
- UI supports benchmark dropdown, custom query, recent context, three CPU modes, CPU preview, judged route, metrics, pack download, evidence export, and log export.
- src-tauri command surface and UI invoke names now agree on router commands.

#### Current Focus:
Commit and push coherent router MVP implementation checkpoint.

#### Next Steps:
- Stage router implementation paths only, leaving unrelated pre-existing worktree changes untouched.
- Commit with detailed commentary and push to origin/main.

#### Context Notes:
- Unrelated journal edits, raw-ref deletions, prompt deletions, Note-to-reviewer.md, and generated .hermes/ are not part of this checkpoint.

#### Performance/Metrics:
- CLI lexical Recall@5 0.6493, MRR 0.5223, nDCG@10 0.5553, abstention accuracy 0.0.

### Session: 2026-06-30 19:49:44Z

#### Current Phase: Green

#### Tests Written:
- mode_router_adds_schema: passing - shared mode router dispatches schema-aware mode and records schema contribution.
- hybrid_fusion_keeps_signals: passing - hybrid RRF evidence includes lexical_rrf, schema_rrf, and schema signals.
- judge_abstains_without_candidates: passing - mock judge returns abstain when CPU shortlist is empty.
- judge_payload_excludes_labels: passing - serialized judge payload excludes benchmark gold-label fields.
- preview_uses_router_mode: passing - run_cpu_preview_only honors request.router_mode instead of always using lexical.
- router workbench identity test: passing - UI renders Tool Router Evidence Console and keeps old PIE copy absent.

#### Implementation Progress:
- crates/catalog-router-core-engine/src/lib.rs: added rank_tools_for_mode as shared lexical/schema/hybrid dispatcher; hybrid RRF now keeps per-signal evidence; schema scoring now flattens schema keys and values while lexical baseline parity stays unchanged.
- crates/benchmark-eval-metrics-runner/src/lib.rs: eval now calls rank_tools_for_mode so CLI/library mode behavior shares the same dispatch path.
- src/lib.rs: CPU preview now uses request.router_mode for live Tauri/UI routing.
- crates/candidate-judge-openai-adapter/src/lib.rs: empty shortlist now returns an abstain decision with reason, confidence, and needs_more_metadata.
- ui/src/app.ts and ui/src/app.test.ts: product identity now matches Tool Router Evidence Console.

#### Current Focus:
Close spec gaps where controls existed but behavior was not yet real: selected CPU mode, hybrid evidence, abstain path, and benchmark-label isolation.

#### Next Steps:
- Expand eval output with token reduction estimate, mode comparison report, JSON/Markdown report writing from CLI, and failure buckets.
- Strengthen route evidence export to include query, router mode, catalog stats, candidate cards, judge decision, benchmark match status when available, metrics, and redaction tests.
- Add UI coverage for exactly five candidate cards, progress stages, custom catalog/query-file upload controls, and benchmark health comparison across all three modes.

#### Context Notes:
- CodeGraphContext refreshed at /tmp/codex-code-intel/codegraphcontext/solution01-direct-20260701-0112 and found rank_tools_for_mode in catalog-router-core-engine/src/lib.rs.
- codebase-memory refreshed at /tmp/codex-code-intel/codebase-memory/accio-tools-20260701-011909 and found rank_tools_for_mode with in-degree 4.
- Old PIE source modules still exist under solution01/src and are visible to graph tools, but root lib and Tauri command registration use router surfaces.

#### Performance/Metrics:
- cargo fmt --all --check: passing.
- cargo test --workspace: passing - 11 Rust tests plus doctests.
- cargo clippy --workspace --all-targets -- -D warnings: passing.
- npm test: passing - 6 UI tests.
- npm run build: passing.
- CLI lexical: Recall@5 0.6493, MRR 0.5223, nDCG@10 0.5553.
- CLI schema-aware: Recall@5 0.6275, MRR 0.5258, nDCG@10 0.5589.
- CLI hybrid: Recall@5 0.6275, MRR 0.5146, nDCG@10 0.5508.

### Session: 2026-06-30 19:52:49Z

#### Current Phase: Green

#### Tests Written:
- markdown_includes_proof_fields: passing - Markdown report includes Recall@1, Recall@10, and token_reduction_estimate.
- cli_writes_report_files: passing - CLI --report-dir writes routing-metrics-report.json and routing-metrics-report.md.
- Existing metrics tests: passing - lexical Recall@5 remains 0.6493 and token_reduction_estimate is 0.9894.

#### Implementation Progress:
- crates/benchmark-eval-metrics-runner/src/lib.rs: MetricReportOutputData now includes token_reduction_estimate; Markdown report now includes router mode, Recall@1/3/5/10, MRR, nDCG@10, abstention accuracy, average candidate count, and token reduction.
- crates/router-cli-command-surface/src/main.rs: evaluate-routing-subset-metrics now accepts --report-dir and writes JSON/Markdown reports through write_evaluation_reports_files.
- ui/src/types.ts and ui/src/app.ts: metric type and benchmark health panel now surface token reduction.

#### Current Focus:
Make benchmark evaluation produce reviewer-ready proof artifacts rather than terminal-only output.

#### Next Steps:
- Add route evidence export context fields: query, router mode, catalog stats, benchmark match status, metrics, and failure bucket.
- Add mode comparison output across lexical, schema-aware, and hybrid in one eval command or UI developer panel.
- Add UI upload controls for custom catalog and custom labeled query files, with parser/validation errors surfaced in activity log.

#### Context Notes:
- CLI report-dir smoke wrote /tmp/tool-router-report-smoke/routing-metrics-report.json and routing-metrics-report.md.
- The smoke Markdown contains token_reduction_estimate: 0.9894.

#### Performance/Metrics:
- cargo fmt --all --check: passing.
- cargo test --workspace: passing - 13 Rust tests plus doctests.
- cargo clippy --workspace --all-targets -- -D warnings: passing.
- npm test: passing - 6 UI tests.
- npm run build: passing.
- CLI report-dir lexical smoke: passing - wrote JSON and Markdown reports with Recall@5 0.6493 and token_reduction_estimate 0.9894.

### Session: 2026-06-30 20:00:04Z

#### Current Phase: Green

#### Tests Written:
- export_report_includes_context: passing - route export includes query, router mode, catalog stats, CPU candidate, judge decision, benchmark gold status, failure bucket, and token reduction metrics.
- export_report_redacts_secret: passing - exported Markdown redacts sk-* shaped secrets and marks CPU-only output as debug preview.
- runs metrics and downloads evidence artifacts: passing - UI export calls export_route_evidence_report with RouteEvidencePayloadData including sanitized route request, catalog stats, benchmark gold match, and metrics report.

#### Implementation Progress:
- src/lib.rs: added CatalogStatsSummaryData, BenchmarkGoldMatchData, RouteEvidencePayloadData, build_route_evidence_report, and redact_secret_values_text.
- src-tauri/src/commands.rs: export_route_evidence_report now accepts RouteEvidencePayloadData instead of a bare RouteToolsResponseData.
- ui/src/types.ts: added typed evidence payload, catalog stats, and benchmark gold match interfaces.
- ui/src/app.ts: route runs now store a sanitized lastRouteRequest; export builds a reproducible evidence payload with catalog stats, benchmark labels when selected, metrics, and no session key.
- ui/src/app.test.ts: export test now asserts query/mode/catalog/gold/metrics payload shape.

#### Current Focus:
Make route evidence export satisfy REQ-MVP-010, REQ-MVP-011, and REQ-UI-009 instead of exporting only route_label and candidate_count.

#### Next Steps:
- Add all-mode comparison output for lexical, schema-aware, and hybrid in one eval command or UI developer panel.
- Add UI custom catalog and custom labeled query file upload controls with validation errors.
- Add visual/responsive verification for 390 px and 1200 px layouts.

#### Context Notes:
- codebase-memory refreshed at /tmp/codex-code-intel/codebase-memory/accio-tools-20260701-012935 and found build_route_evidence_report in src/lib.rs.
- CodeGraphContext refreshed at /tmp/codex-code-intel/codegraphcontext/solution01-direct-20260701-0112 and found build_route_evidence_report plus createRouteEvidencePayload.
- Old PIE source modules still appear in graph unresolved-call summaries because they remain on disk; router root/Tauri/UI paths are the active compiled surfaces.

#### Performance/Metrics:
- cargo fmt --all --check: passing.
- cargo test --workspace: passing - 15 Rust tests plus doctests.
- cargo clippy --workspace --all-targets -- -D warnings: passing.
- npm test: passing - 6 UI tests.
- npm run build: passing.

### Session: 2026-06-30 20:04:27Z

#### Current Phase: Green

#### Tests Written:
- comparison_includes_all_modes: passing - benchmark comparison returns lexical, schema-aware, and hybrid reports in the expected order.
- cli_writes_comparison_files: passing - CLI compare-routing-modes-metrics writes routing-mode-comparison-report.json and routing-mode-comparison-report.md.

#### Implementation Progress:
- crates/benchmark-eval-metrics-runner/src/lib.rs: added compare_routing_modes_metrics and write_comparison_reports_files; comparison Markdown includes Recall@5, MRR, nDCG@10, abstention accuracy, and token reduction per mode.
- crates/router-cli-command-surface/src/main.rs: added compare-routing-modes-metrics subcommand while keeping runtime mode values limited to lexical, schema-aware, and hybrid.

#### Current Focus:
Make REQ-MVP-009 mode comparison provable through a single reviewer-facing CLI command.

#### Next Steps:
- Add UI developer panel for comparing all three mode reports in the evidence console.
- Add UI custom catalog and custom labeled query file upload controls with validation errors.
- Add visual/responsive verification for 390 px and 1200 px layouts.

#### Context Notes:
- codebase-memory refreshed at /tmp/codex-code-intel/codebase-memory/accio-tools-20260701-013400 and found compare_routing_modes_metrics.
- CodeGraphContext refreshed at /tmp/codex-code-intel/codegraphcontext/solution01-direct-20260701-0112 and found compare_routing_modes_metrics.
- CLI comparison smoke wrote /tmp/tool-router-comparison-smoke/routing-mode-comparison-report.json and routing-mode-comparison-report.md.

#### Performance/Metrics:
- cargo fmt --all --check: passing.
- cargo test --workspace: passing - 17 Rust tests plus doctests.
- cargo clippy --workspace --all-targets -- -D warnings: passing.
- npm test: passing - 6 UI tests.
- npm run build: passing.
- CLI compare-routing-modes-metrics smoke: passing - lexical Recall@5 0.6493, schema-aware Recall@5 0.6275, hybrid Recall@5 0.6275.
