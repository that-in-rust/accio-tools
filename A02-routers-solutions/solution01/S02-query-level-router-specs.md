# Spec 02 Query-Level Router Specs

## Product Thesis

Spec 02 turns the router console into a Shreyas-style MVP demo:

```text
one readable user query
  -> user chooses one CPU router
  -> selected CPU router returns one top-five shortlist
  -> cheap judge reviews only that selected shortlist
  -> user sees one route decision first, evidence second
```

The product is not a benchmark lab on the first screen. The primary job is to help a reviewer understand one routing decision quickly: what the user asked, which CPU router was selected, whether the expected tool survived the selected top five, what the judge picked, and what failed when it failed.

## Inputs Parsed

| input | decision |
| --- | --- |
| Feature outcome | Route one selected query through one selected CPU algorithm, then let the cheap judge pick the final tool or abstain. |
| Actors and boundaries | Reviewer/FDE uses Tauri UI first. Existing CLI and Tauri commands for aggregate benchmark, exports, pack download, and mode comparison remain available for tests/developer evidence but SHALL have no visible MVP buttons. |
| Failure modes | Query label is meaningless, expected tool is hidden, CPU top-1 is mistaken for final answer, judge scope is unclear, mass metrics distract from query story, debug exports/logs look like primary product features, JSON upload controls imply the reviewer must understand benchmark internals. |
| Performance and reliability limits | Selected CPU top-five routing SHALL complete in <= 250 ms p95 for the bundled 947-tool catalog and one query on a developer laptop; mocked/local judged route SHALL complete in <= 2 seconds p95 excluding external API latency; UI render SHALL remain overflow-free at 390 px and 1200 px. |
| Language/runtime constraints | Existing Rust/Tauri/Vite workspace in `A02-routers-solutions/solution01`; reuse current router crates, Tauri commands, benchmark pack, and evidence export paths. |

## MVP Surface

### Primary Surface

- Human-readable query picker.
- Expected tool summary.
- Optional recent context.
- CPU router selector with exactly one selected mode: `Lexical BM25`, `Schema-aware BM25`, or `Hybrid RRF`.
- One primary action: `Run Selected Route Decision`.
- One selected CPU top-five shortlist.
- One judged route result when a judge key is accepted.
- One verdict panel: pass, judge rescued, CPU top-five miss, wrong judge pick, correct abstain, or abstention miss.
- Optional custom query free-text entry for "try your own" usage.

### Hidden Developer Surface

- Aggregate benchmark metrics.
- Full mode-comparison report across all router modes.
- Evaluation pack download.
- Preview/judged evidence export.
- Diagnostic logs.
- Custom catalog JSON import.
- Custom query JSON import.
- Raw tool ids, raw query ids, scores, matched terms, matched fields, and failure buckets.

Hidden developer surface may remain implemented in commands, tests, or CLI hooks, but SHALL not be exposed as visible MVP buttons or upload fields.

## Four-Word Implementation Names

New UI helpers should prefer four-word names:

- `createReadableQueryLabel`
- `renderRouteDecisionPanel`
- `runSelectedRouterFlow`
- `createRouteDecisionSummary`
- `hideDebugControlButtons`
- `renderRouterChoicePanel`
- `createToolDisplayName`
- `renderExpectedToolSummary`
- `scoreSelectedRouteOutcome`
- `selectVisibleRouterMode`
- `removeJsonUploadSurfaces`
- `createJudgeReviewPayload`

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
**AND** SHALL avoid showing raw expected tool id in the primary surface
**SHALL** show `Expected outcome: abstain` for `should_route=false` rows.

### REQ-S02-003.0: Select one router mode

**WHEN** the user has selected a query and catalog is loaded
**THEN** the primary UI SHALL show exactly one active CPU router mode at a time
**AND** SHALL allow the user to choose `Lexical BM25`, `Schema-aware BM25`, or `Hybrid RRF`
**SHALL** default to one recommended mode without running any hidden all-mode comparison.

### REQ-S02-004.0: Use one route action

**WHEN** the user has selected a query, a catalog is loaded, and judge readiness is valid
**THEN** the primary UI SHALL expose one dominant action named `Run Selected Route Decision`
**AND** SHALL invoke routing only for the currently selected CPU router mode
**SHALL** not invoke lexical, schema-aware, and hybrid modes as a default batch.

### REQ-S02-005.0: Judge selected shortlist

**WHEN** `Run Selected Route Decision` runs
**THEN** the CPU router SHALL produce one top-five candidate list for the selected mode
**AND** the LLM judge SHALL review only that selected top-five candidate list
**SHALL** not judge merged candidates from multiple CPU router modes in Spec 02.

### REQ-S02-006.0: Explain route verdict

**WHEN** a selected route decision is shown
**THEN** the UI SHALL show a plain-language verdict summary above evidence
**AND** SHALL map outcomes to `pass`, `judge rescued`, `cpu top-five miss`, `wrong judge pick`, `correct abstain`, `abstention miss`, or `no benchmark label`
**SHALL** include one sentence explaining why the verdict matters.

### REQ-S02-007.0: Remove debug buttons

**WHEN** the route screen first renders
**THEN** the UI SHALL not render buttons named `Run CPU Preview`, `Run Judged Route`, `Run Benchmark Eval`, `Compare All Modes`, `Download Evaluation Pack`, `Export Judged Route Evidence`, `Export Preview Route Evidence`, or `Export Logs`
**AND** SHALL keep the underlying commands available for tests/developer usage
**SHALL** avoid presenting debug/evidence plumbing as product workflow.

### REQ-S02-008.0: Remove JSON upload surfaces

**WHEN** the MVP route screen renders
**THEN** the UI SHALL not render `Custom catalog JSON` or `Custom query JSON` upload inputs
**AND** SHALL use the bundled benchmark pack as the default catalog and labeled query source
**SHALL** keep custom JSON import functionality outside the visible MVP surface.

### REQ-S02-009.0: Require judge for final decision

**WHEN** no validated OpenAI key is available
**THEN** the UI SHALL not present a CPU top-1 as the final routed tool
**AND** SHALL explain that a judge key is required for the final selected route decision
**SHALL** keep any CPU-only preview behavior out of the visible MVP action set.

### REQ-S02-010.0: Show judge rescue

**WHEN** CPU top-1 is wrong but the expected tool is in top five and the judge selects the expected tool
**THEN** the verdict SHALL be `judge rescued`
**AND** SHALL show CPU rank and judge-selected tool in the decision panel
**SHALL** count this as a successful bidirectional routing demonstration for that query.

### REQ-S02-011.0: Prefer human tool names

**WHEN** a tool id appears in the primary UI
**THEN** the UI SHALL render a readable tool display name derived from server name, source tool id, name, or description
**AND** SHALL keep the raw tool id out of the primary decision headline
**SHALL** not require users to parse strings like `votr.Slack::slack_post_message` to understand the result.

### REQ-S02-012.0: Preserve custom query escape hatch

**WHEN** the user switches to custom query mode
**THEN** the UI SHALL allow free-text query entry and optional recent context
**AND** SHALL run the same selected-router judged route flow
**SHALL** show `No benchmark label` instead of pass/fail gold verdict when expected labels are absent.

### REQ-S02-013.0: Keep performance bounded

**WHEN** selected-router CPU routing runs on the bundled 947-tool catalog
**THEN** the selected CPU top-five ranking SHALL complete in <= 250 ms p95 across the 50 bundled queries on a developer laptop
**AND** SHALL not block the UI render loop during async Tauri command execution
**SHALL** keep mocked/local judged route completion <= 2 seconds p95 excluding external API latency.

### REQ-S02-014.0: Preserve responsive layout

**WHEN** the simplified UI renders at 390 px mobile width and 1200 px desktop width
**THEN** query labels, router selector, verdict card, and top-five evidence SHALL not overflow or overlap
**AND** SHALL keep the primary route action visible without horizontal scrolling
**SHALL** avoid showing wide raw evidence tables in the primary surface.

## Test Matrix

| req_id | test_id | type | assertion | target |
| --- | --- | --- | --- | --- |
| REQ-S02-001.0 | TEST-S02-UI-001 | component | query picker labels prioritize query text over raw id/tool id | `ui/src/app.test.ts` |
| REQ-S02-002.0 | TEST-S02-UI-002 | component | selected benchmark query shows expected tool summary before execution | `ui/src/app.test.ts` |
| REQ-S02-003.0 | TEST-S02-UI-003 | component | router selector has one active mode and does not auto-run all modes | `ui/src/app.test.ts` |
| REQ-S02-004.0 | TEST-S02-UI-004 | integration | route action invokes exactly one selected `router_mode` | `ui/src/app.test.ts` |
| REQ-S02-005.0 | TEST-S02-UI-005 | integration | judge request is built from only the selected top-five shortlist | `ui/src/app.test.ts` |
| REQ-S02-006.0 | TEST-S02-UI-006 | component | verdict summary maps outcomes to plain-language labels and explanation text | `ui/src/app.test.ts` |
| REQ-S02-007.0 | TEST-S02-UI-007 | component | debug/eval/export/download buttons are absent from visible UI | `ui/src/app.test.ts` |
| REQ-S02-008.0 | TEST-S02-UI-008 | component | custom catalog/query JSON upload labels are absent from visible UI | `ui/src/app.test.ts` |
| REQ-S02-009.0 | TEST-S02-UI-009 | integration | no-key state blocks final decision and avoids CPU top-1 final claim | `ui/src/app.test.ts` |
| REQ-S02-010.0 | TEST-S02-UI-010 | unit | wrong CPU top-1 plus expected top-five plus correct judge pick yields `judge rescued` | `ui/src/app.test.ts` or Rust outcome test |
| REQ-S02-011.0 | TEST-S02-UI-011 | unit | readable tool names are derived while raw ids stay out of primary headline | `ui/src/app.test.ts` |
| REQ-S02-012.0 | TEST-S02-UI-012 | integration | custom free-text query reuses selected-router judged route and shows `No benchmark label` | `ui/src/app.test.ts` |
| REQ-S02-013.0 | TEST-S02-PERF-001 | performance | selected CPU routing p95 <= 250 ms and mocked judged route p95 <= 2 seconds | Rust/CLI perf test |
| REQ-S02-014.0 | TEST-S02-VIS-001 | visual | 390 px and 1200 px screenshots have no overflow or overlap | Playwright visual check |

## TDD Plan

### 1. STUB

- Add UI test stubs for readable query labels, expected-tool summary, one-selected-router behavior, one route action, route decision panel, hidden debug buttons, and removed JSON upload labels.
- Add unit fixtures for query-level outcomes: CPU pass, judge rescued, CPU top-five miss, wrong judge pick, correct abstain, and unlabeled custom query.
- Add performance test stub for `runSelectedRouterFlow` p95.

### 2. RED

- Run `npm --prefix ui test`.
- Expected failures: old query selector still displays `TRQ-* -> tool_id`, primary UI still exposes benchmark/eval/export/log buttons, custom JSON labels still render, and routing still compares all modes by default.
- Run targeted Rust/CLI performance test if selected-router helper lands in Rust.

### 3. GREEN

- Implement `createReadableQueryLabel` and `createToolDisplayName`.
- Replace primary action row with `Run Selected Route Decision`.
- Implement `runSelectedRouterFlow` using the currently selected router mode only.
- Implement `createRouteDecisionSummary` and `renderRouteDecisionPanel`.
- Remove visible benchmark eval, full mode comparison, evidence export, pack download, logs, custom catalog JSON, custom query JSON, raw score cards, and matched-term controls from the MVP UI.

### 4. REFACTOR

- Keep existing Rust commands stable for CLI compatibility.
- Keep UI helpers four-word named where new helpers are introduced.
- Remove duplicate verdict logic between CPU candidate evidence and route decision summary.
- Keep retained debug/evaluation commands testable without visible MVP controls.

### 5. VERIFY

- Run UI tests, Rust workspace tests, clippy, UI build, responsive layout verifier, and query-level performance test.
- Confirm visible UI text contains no `Run Benchmark Eval`, `Compare All Modes`, `Download Evaluation Pack`, `Export Logs`, `Custom catalog JSON`, `Custom query JSON`, or raw `TRQ-* -> tool_id` labels.
- Confirm judged route calls one selected router mode and does not loop through all router modes.

## Quality Gates

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `npm --prefix ui test`
- `npm --prefix ui run build`
- `npm --prefix ui run test:responsive-layout-viewports`
- Selected CPU routing p95 <= 250 ms over bundled 50-query pack.
- Mocked/local judged route p95 <= 2 seconds excluding external API latency.
- Every `REQ-S02-*` ID has at least one linked test.
- No `TODO`, `STUB`, or `FIXME` introduced in implementation code.
- Primary UI screenshot shows one selected router decision and no aggregate benchmark metric.
- Visible UI screenshot shows no debug/eval/export/download buttons and no custom JSON upload fields.

## Open Questions

| question | current stance | when to resolve |
| --- | --- | --- |
| Should the judge run for all three CPU modes or only the selected best CPU mode? | Resolved for Spec 02: judge only the selected CPU router's top-five shortlist. | Implement immediately. |
| Should aggregate benchmark metrics disappear entirely or stay in advanced evidence? | Remove from visible MVP UI; keep commands/tests for assignment proof. | Implement immediately. |
| Should query labels be curated manually? | Prefer deterministic readable labels from query text first; add curated labels only for the best demo queries if time allows. | During fixture polishing. |
| Should custom JSON uploads remain visible? | Resolved for Spec 02: remove `Custom catalog JSON` and `Custom query JSON` from the visible MVP UI. | Implement immediately. |
| Should custom query support expected-tool input? | Not for Spec 02 MVP; custom free-text query shows no benchmark label. | After selected-route decision is working. |
