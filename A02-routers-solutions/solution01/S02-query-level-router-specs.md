# Spec 02 Query-Level Router Specs

## Product Thesis

Spec 02 turns the router console into a Shreyas-style MVP demo:

```text
one readable user query
  -> compare lexical, schema-aware, and hybrid CPU top-five shortlists
  -> show whether the expected tool survives top five
  -> optional cheap judge selects one tool or abstains
  -> user sees verdict first, internals second
```

The product is not a benchmark lab on the first screen. The primary job is to help a reviewer understand one routing decision quickly: what the user asked, what tool was expected, what each router did, whether the judge rescued or confirmed the route, and what failed when it failed.

## Inputs Parsed

| input | decision |
| --- | --- |
| Feature outcome | Compare routing behavior for one selected query and explain the pass/fail verdict in plain language. |
| Actors and boundaries | Reviewer/FDE uses Tauri UI first. Existing CLI and aggregate benchmark commands remain available but move behind advanced/debug UI. |
| Failure modes | Query label is meaningless, expected tool is hidden, CPU top-1 is wrong but top-5 survival is not surfaced, mass metrics distract from query story, debug exports/logs look like primary product features. |
| Performance and reliability limits | Query-level comparison SHALL complete in <= 2 seconds p95 for the bundled 947-tool catalog and one query on a developer laptop; UI render SHALL remain overflow-free at 390 px and 1200 px. |
| Language/runtime constraints | Existing Rust/Tauri/Vite workspace in `A02-routers-solutions/solution01`; reuse current router crates, Tauri commands, benchmark pack, and evidence export paths. |

## MVP Surface

### Primary Surface

- Human-readable query picker.
- Expected tool summary.
- Optional recent context.
- One primary action: `Run Routing Comparison`.
- Query-level comparison table across `Lexical BM25`, `Schema-aware BM25`, and `Hybrid RRF`.
- Optional judged route result when a judge key is accepted.
- One verdict panel: pass, judge rescued, CPU top-five miss, wrong judge pick, correct abstain, or abstention miss.

### Advanced Surface

- CPU top-five evidence cards.
- Aggregate benchmark metrics.
- Full mode-comparison report.
- Evaluation pack download.
- Preview/judged evidence export.
- Diagnostic logs.
- Raw tool ids, raw query ids, scores, matched terms, matched fields, and failure buckets.

Advanced surface may remain implemented, but SHALL be hidden behind an `Advanced Evidence` disclosure by default.

## Four-Word Implementation Names

New UI helpers should prefer four-word names:

- `createReadableQueryLabel`
- `renderRoutingComparisonPanel`
- `runQueryComparisonFlow`
- `createQueryVerdictSummary`
- `hideAdvancedBenchmarkControls`
- `renderAdvancedEvidenceDrawer`
- `createToolDisplayName`
- `renderExpectedToolSummary`
- `scoreQueryRouterOutcome`
- `compareQueryRouterModes`

Existing command names may remain for compatibility.

## Executable Requirements

### REQ-S02-001.0: Show readable query labels

**WHEN** the bundled benchmark query picker renders
**THEN** the system SHALL show readable query text as the primary option label
**AND** SHALL show raw query id and expected tool only as secondary metadata
**SHALL** never make `TRQ-* -> tool_id` the only visible selector label.

### REQ-S02-002.0: Surface expected tool

**WHEN** a labeled benchmark query is selected
**THEN** the system SHALL show `Expected tool` in human-readable form before route execution
**AND** SHALL include raw expected tool id only in advanced details
**SHALL** show `Expected outcome: abstain` for `should_route=false` rows.

### REQ-S02-003.0: Use one primary route action

**WHEN** the user has selected a query and catalog is loaded
**THEN** the primary UI SHALL expose one dominant action named `Run Routing Comparison`
**AND** SHALL run query-level CPU comparison across lexical, schema-aware, and hybrid modes
**SHALL** hide `Run Benchmark Eval`, `Compare All Modes`, `Export Logs`, and evaluation-pack download from the primary action row.

### REQ-S02-004.0: Compare routers per query

**WHEN** `Run Routing Comparison` completes
**THEN** the UI SHALL render one table row per CPU router mode
**AND** SHALL include top-1 tool, expected-tool-in-top-five status, expected-tool rank when present, judge pick when available, and query-level verdict
**SHALL** avoid showing aggregate Recall/MRR/nDCG metrics in the primary comparison table.

### REQ-S02-005.0: Explain route verdict

**WHEN** a query-level comparison is shown
**THEN** the UI SHALL show a plain-language verdict summary above raw evidence
**AND** SHALL map outcomes to `pass`, `judge rescued`, `cpu top-five miss`, `wrong judge pick`, `correct abstain`, or `abstention miss`
**SHALL** include one sentence explaining why the verdict matters.

### REQ-S02-006.0: Hide advanced evidence

**WHEN** the route screen first renders
**THEN** advanced debug controls SHALL be collapsed behind `Advanced Evidence`
**AND** SHALL include aggregate benchmark eval, full mode comparison, evidence exports, logs, raw ids, scores, matched fields, and matched terms only inside that disclosure
**SHALL** preserve existing debug functionality without letting it dominate the primary MVP flow.

### REQ-S02-007.0: Keep judged route optional but clear

**WHEN** no validated OpenAI key is available
**THEN** the query-level comparison SHALL still run CPU top-five comparison
**AND** SHALL label judge output as `Judge not run`
**SHALL** avoid production top-1 claims until a judged route result exists.

### REQ-S02-008.0: Show judge rescue

**WHEN** CPU top-1 is wrong but the expected tool is in top five and the judge selects the expected tool
**THEN** the verdict SHALL be `judge rescued`
**AND** SHALL show CPU rank and judge-selected tool in the comparison table
**SHALL** count this as a successful bidirectional routing demonstration for that query.

### REQ-S02-009.0: Prefer human tool names

**WHEN** a tool id appears in the primary UI
**THEN** the UI SHALL render a readable tool display name derived from server name, source tool id, name, or description
**AND** SHALL keep the raw tool id available in advanced evidence
**SHALL** not require users to parse strings like `votr.Slack::slack_post_message` to understand the result.

### REQ-S02-010.0: Preserve custom query escape hatch

**WHEN** the user switches to custom query mode
**THEN** the UI SHALL allow free-text query entry and optional recent context
**AND** SHALL run the same query-level comparison table
**SHALL** show `No benchmark label` instead of pass/fail gold verdict when expected labels are absent.

### REQ-S02-011.0: Keep performance bounded

**WHEN** query-level comparison runs on the bundled 947-tool catalog
**THEN** the CPU comparison SHALL complete in <= 2 seconds p95 across the 50 bundled queries on a developer laptop
**AND** SHALL not block the UI render loop during async Tauri command execution
**SHALL** retain the existing CPU ranking p95 <= 250 ms per mode budget.

### REQ-S02-012.0: Preserve responsive layout

**WHEN** the simplified UI renders at 390 px mobile width and 1200 px desktop width
**THEN** query labels, verdict cards, comparison table, and advanced disclosure SHALL not overflow or overlap
**AND** SHALL keep the primary route action visible without horizontal scrolling
**SHALL** keep raw evidence tables scroll-contained only inside `Advanced Evidence`.

## Test Matrix

| req_id | test_id | type | assertion | target |
| --- | --- | --- | --- | --- |
| REQ-S02-001.0 | TEST-S02-UI-001 | component | query picker labels prioritize query text over raw id/tool id | `ui/src/app.test.ts` |
| REQ-S02-002.0 | TEST-S02-UI-002 | component | selected benchmark query shows expected tool summary before execution | `ui/src/app.test.ts` |
| REQ-S02-003.0 | TEST-S02-UI-003 | component | primary action row exposes `Run Routing Comparison` and hides mass/debug actions | `ui/src/app.test.ts` |
| REQ-S02-004.0 | TEST-S02-UI-004 | integration | query comparison renders lexical/schema-aware/hybrid rows with top-1, top-five survival, rank, judge pick, verdict | `ui/src/app.test.ts` |
| REQ-S02-005.0 | TEST-S02-UI-005 | component | verdict summary maps outcomes to plain-language labels and explanation text | `ui/src/app.test.ts` |
| REQ-S02-006.0 | TEST-S02-UI-006 | component | aggregate metrics, exports, logs, raw scores, and raw ids are hidden until `Advanced Evidence` opens | `ui/src/app.test.ts` |
| REQ-S02-007.0 | TEST-S02-UI-007 | integration | no-key comparison shows CPU results and `Judge not run` without production top-1 claim | `ui/src/app.test.ts` |
| REQ-S02-008.0 | TEST-S02-UI-008 | unit | wrong CPU top-1 plus expected top-five plus correct judge pick yields `judge rescued` | `ui/src/app.test.ts` or Rust outcome test |
| REQ-S02-009.0 | TEST-S02-UI-009 | unit | readable tool names are derived while raw ids remain in advanced evidence | `ui/src/app.test.ts` |
| REQ-S02-010.0 | TEST-S02-UI-010 | integration | custom query mode reuses comparison table and shows `No benchmark label` | `ui/src/app.test.ts` |
| REQ-S02-011.0 | TEST-S02-PERF-001 | performance | query-level comparison p95 <= 2 seconds and CPU per-mode p95 <= 250 ms | Rust/CLI perf test |
| REQ-S02-012.0 | TEST-S02-VIS-001 | visual | 390 px and 1200 px screenshots have no overflow or overlap | Playwright visual check |

## TDD Plan

### 1. STUB

- Add UI test stubs for readable query labels, expected-tool summary, primary action row, query comparison table, verdict summary, and advanced evidence collapse.
- Add unit fixtures for query-level outcomes: CPU pass, judge rescued, CPU top-five miss, wrong judge pick, correct abstain, and unlabeled custom query.
- Add performance test stub for `compareQueryRouterModes` p95.

### 2. RED

- Run `npm --prefix ui test`.
- Expected failures: old query selector still displays `TRQ-* -> tool_id`, primary action row still exposes benchmark/eval/export/log actions, comparison table does not exist, and advanced evidence is not collapsed.
- Run targeted Rust/CLI performance test if query-level comparison helper lands in Rust.

### 3. GREEN

- Implement `createReadableQueryLabel` and `createToolDisplayName`.
- Replace primary action row with `Run Routing Comparison`.
- Implement `compareQueryRouterModes` using existing router commands and bundled labels.
- Implement `createQueryVerdictSummary` and `renderRoutingComparisonPanel`.
- Move mass benchmark eval, full mode comparison, evidence export, pack download, logs, raw score cards, and matched-term details into `renderAdvancedEvidenceDrawer`.

### 4. REFACTOR

- Keep existing Rust commands stable for CLI compatibility.
- Keep UI helpers four-word named where new helpers are introduced.
- Remove duplicate verdict logic between single-route result and comparison table.
- Keep raw evidence rendering reusable inside advanced disclosure.

### 5. VERIFY

- Run UI tests, Rust workspace tests, clippy, UI build, responsive layout verifier, and query-level performance test.
- Confirm primary UI text contains no `Run Benchmark Eval`, `Compare All Modes`, `Export Logs`, or raw `TRQ-* -> tool_id` labels outside advanced evidence.
- Confirm advanced evidence can still export route evidence and show aggregate metrics.

## Quality Gates

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `npm --prefix ui test`
- `npm --prefix ui run build`
- `npm --prefix ui run test:responsive-layout-viewports`
- Query-level comparison p95 <= 2 seconds over bundled 50-query pack.
- CPU per-mode ranking p95 <= 250 ms remains passing.
- Every `REQ-S02-*` ID has at least one linked test.
- No `TODO`, `STUB`, or `FIXME` introduced in implementation code.
- Primary UI screenshot shows one-query comparison before any aggregate benchmark metric.
- Advanced evidence screenshot shows hidden debug controls available after disclosure.

## Open Questions

| question | current stance | when to resolve |
| --- | --- | --- |
| Should the judge run for all three CPU modes or only the selected best CPU mode? | Run judge for all three in Spec 02 only if cost is acceptable; otherwise judge the selected/default mode and show CPU-only comparison for the others. | Before implementation starts. |
| Should aggregate benchmark metrics disappear entirely or stay in advanced evidence? | Keep in advanced evidence so assignment proof remains available without confusing the primary demo. | During UI refactor. |
| Should query labels be curated manually? | Prefer deterministic readable labels from query text first; add curated labels only for the best demo queries if time allows. | During fixture polishing. |
| Should custom query support expected-tool input? | Not for Spec 02 MVP; custom query shows no benchmark label unless uploaded labels exist. | After query-level comparison is working. |
