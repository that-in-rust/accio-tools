# TDD Progress Journal

- Task: Implement router MVP executable specs
- Created: 2026-06-30 19:10:59Z
- Updated: 2026-06-30 19:36:18Z
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
