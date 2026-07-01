# Spec 03 Router Explainer Model Specs

## Product Thesis

Spec 03 makes the selected-router MVP self-explanatory for a reviewer.

```text
validate judge key
  -> show the exact judge model being used
  -> show CPU router shortlist flow
  -> show LLM judge top-one decision
  -> compare judge output against benchmark gold
  -> cite benchmark source provenance
```

The product should not require the reviewer to infer the architecture from raw route cards. The UI should say, plainly and visually, that the CPU router returns five candidates, the LLM judge chooses one or abstains, and the app compares that decision against the bundled benchmark label.

## Inputs Parsed

| input | decision |
| --- | --- |
| Feature outcome | Add a small explanatory section and model label to the existing selected-router UI. |
| Actors and boundaries | Reviewer/FDE uses the Tauri UI. Existing Rust/Tauri command contracts remain unchanged unless the current `model_label` is insufficient. |
| Failure modes | Reviewer cannot tell which LLM model is judging, cannot understand why top five appears, cannot see how benchmark gold is used, or thinks the benchmark is invented rather than sourced from OSS/reference corpora. |
| Performance and reliability limits | New explanatory UI SHALL add no additional Tauri command call and SHALL not affect route latency. Responsive layout SHALL remain overflow-free at 390 px and 1200 px. |
| Language/runtime constraints | Existing Rust/Tauri/Vite workspace in `A02-routers-solutions/solution01`; reuse `RouterAppReadinessData.model_label`, bundled benchmark metadata, and current selected-route flow. |

## MVP Surface

### Judge Model Placement

- The judge session card SHALL display the model next to the key validation controls.
- The label SHALL read `Judge model: <model_label>`.
- Before validation, `<model_label>` SHALL come from current readiness state.
- After validation, `<model_label>` SHALL update from the validated readiness response.
- The model label SHALL not be hidden only in header metadata.

### Flow Explainer Section

Add a section immediately below the judge session card and above the training benchmark card.

Recommended display:

```text
Selected CPU router
        |
        v
Top five shortlist
        |
        v
LLM judge chooses top 1 or abstains
        |
        v
Compare against benchmark gold
```

The section should be simple visual cards or a horizontal/vertical stepper, not a paragraph-heavy explanation.

### Benchmark Provenance Copy

The flow explainer or training benchmark card SHALL state:

```text
Benchmark source: curated local subset from OSS/reference routing corpora.
Primary sources: VOTR, contextweaver, graph-tool-call, mcpproxy-go, LiveMCPBench, mcp-bench.
Pack: 947 tools, 50 queries, 46 route-required labels, 4 abstention labels.
```

The UI may link or reference the local evidence path:

```text
A00-raw-research/benchmarks/tool-routing-subset
```

## Four-Word Implementation Names

New UI helpers should prefer four-word names:

- `renderJudgeModelLabel`
- `renderRouterFlowExplainer`
- `createBenchmarkSourceSummary`
- `renderBenchmarkSourceBadge`
- `renderFlowStageCards`
- `formatJudgeModelText`
- `createFlowStageItems`
- `renderBenchmarkGoldStep`

Existing command and data type names may remain for compatibility.

## Executable Requirements

### REQ-S03-001.0: Show judge model

**WHEN** the judge session card renders
**THEN** the UI SHALL show `Judge model: <model_label>` adjacent to the API key validation controls
**AND** SHALL use `RouterAppReadinessData.model_label` as the source of truth
**SHALL** show the current default readiness model before validation.

### REQ-S03-002.0: Update validated model

**WHEN** `validate_judge_api_key` returns a readiness response with a new `model_label`
**THEN** the judge session card SHALL update the visible `Judge model` value
**AND** SHALL keep the validate button state unchanged except for the existing readiness behavior
**SHALL** not require a page refresh to show the validated model.

### REQ-S03-003.0: Add flow explainer

**WHEN** the app renders the selected-router MVP surface
**THEN** the UI SHALL show a route-flow explainer directly below the judge session card
**AND** SHALL include exactly four visible stages: selected CPU router, top-five shortlist, LLM judge top-one or abstain, benchmark comparison
**SHALL** avoid raw implementation labels as the primary text for those stages.

### REQ-S03-004.0: Bind selected CPU stage

**WHEN** the user changes the selected CPU router
**THEN** the flow explainer SHALL update the CPU stage to the selected mode label
**AND** SHALL not imply that all three CPU routers are judged
**SHALL** keep the selected mode consistent with the route decision action.

### REQ-S03-005.0: Explain benchmark comparison

**WHEN** a benchmark query is active
**THEN** the explainer SHALL state that the judge result is compared against the benchmark expected tool or abstention label
**AND** SHALL show `No benchmark label` language only for custom free-text queries
**SHALL** not expose raw `required_tool_ids` as the main explanation.

### REQ-S03-006.0: Cite benchmark provenance

**WHEN** the flow explainer or training benchmark section renders
**THEN** the UI SHALL show benchmark provenance text naming the curated local subset
**AND** SHALL list the six primary OSS/reference sources: VOTR, contextweaver, graph-tool-call, mcpproxy-go, LiveMCPBench, and mcp-bench
**SHALL** show the bundled pack counts: 947 tools, 50 queries, 46 route-required labels, and 4 abstention labels.

### REQ-S03-007.0: Preserve compact MVP

**WHEN** the explainer is added
**THEN** the UI SHALL remain a compact workbench surface without reintroducing hidden lab buttons, JSON upload controls, aggregate metrics, or compare-all modes
**AND** SHALL keep the primary action as `Run Selected Route Decision`
**SHALL** keep visible text free of `Run Benchmark Eval`, `Compare All Modes`, `Custom catalog JSON`, and `Custom query JSON`.

### REQ-S03-008.0: Preserve responsive layout

**WHEN** the updated UI renders at 390 px and 1200 px widths
**THEN** the model label, flow explainer, and benchmark provenance copy SHALL not overflow or overlap
**AND** SHALL keep the route action visible without horizontal scrolling
**SHALL** pass the responsive layout verifier with `overflow=0` and `overlaps=0`.

## Test Matrix

| req_id | test_id | type | assertion | target |
| --- | --- | --- | --- | --- |
| REQ-S03-001.0 | TEST-S03-UI-001 | component | judge card shows `Judge model: mock-router-judge` before validation | `ui/src/app.test.ts` |
| REQ-S03-002.0 | TEST-S03-UI-002 | integration | validated readiness response updates visible judge model label | `ui/src/app.test.ts` |
| REQ-S03-003.0 | TEST-S03-UI-003 | component | flow explainer renders four stage cards below judge session | `ui/src/app.test.ts` |
| REQ-S03-004.0 | TEST-S03-UI-004 | integration | changing router mode updates CPU stage without invoking all modes | `ui/src/app.test.ts` |
| REQ-S03-005.0 | TEST-S03-UI-005 | component | benchmark query says compare against expected tool; custom query says no benchmark label | `ui/src/app.test.ts` |
| REQ-S03-006.0 | TEST-S03-UI-006 | component | provenance text lists local subset, six primary sources, and pack counts | `ui/src/app.test.ts` |
| REQ-S03-007.0 | TEST-S03-UI-007 | regression | removed lab/upload labels remain absent after explainer is added | `ui/src/app.test.ts` |
| REQ-S03-008.0 | TEST-S03-VIS-001 | visual | mobile and desktop layout have no overflow/overlap with explainer present | `ui/scripts/verify-responsive-layout-viewports.mjs` |

## TDD Plan

### 1. STUB

- Add UI tests for judge model placement, validated model update, four-stage explainer, selected CPU stage update, benchmark provenance, and no lab-label regressions.
- Update the responsive verifier audit to include the explainer and model label.

### 2. RED

- Run `npm --prefix ui test`.
- Expected failures: model label is only in header/status metadata, route-flow explainer is missing, benchmark provenance copy is missing.
- Run responsive verifier after UI tests are red to confirm the visual gate needs the new elements.

### 3. GREEN

- Implement `renderJudgeModelLabel`.
- Implement `renderRouterFlowExplainer`.
- Implement `createBenchmarkSourceSummary`.
- Insert the explainer immediately below `renderJudgeKeyCard`.
- Reuse existing readiness and catalog summary state; do not add a new Tauri command.

### 4. REFACTOR

- Keep copy short enough for mobile.
- Keep helper names four-word shaped.
- Avoid duplicating benchmark counts by deriving counts from `createCatalogStatsSummary` where practical.
- Keep provenance source names deterministic and testable.

### 5. VERIFY

- Run `npm --prefix ui test`.
- Run `npm --prefix ui run build`.
- Run `npm --prefix ui run test:responsive-layout-viewports`.
- Run Rust workspace checks only if Tauri command or Rust payload shape changes.

## Quality Gates

- `npm --prefix ui test`
- `npm --prefix ui run build`
- `npm --prefix ui run test:responsive-layout-viewports`
- `cargo test --workspace` only if Rust/Tauri command contracts change
- `cargo clippy --workspace --all-targets -- -D warnings` only if Rust/Tauri command contracts change
- Every `REQ-S03-*` ID has at least one linked test.
- No visible reintroduction of benchmark lab buttons or JSON upload labels.
- No new API key persistence, logging, or export behavior.
- Responsive verifier reports `overflow=0` and `overlaps=0` at 390 px and 1200 px.

## Open Questions

| question | current stance | when to resolve |
| --- | --- | --- |
| Should the model label show provider as well as model? | Use the existing `model_label` field first; add provider only if backend already exposes it. | During implementation. |
| Should provenance link open the local file path? | Mention the local path in text; do not add file-opening behavior in S03. | After UI copy is accepted. |
| Should the explainer show live candidate names after route execution? | Not in S03; keep it conceptual and simple. Candidate details stay in route decision evidence. | Later polish. |
| Should benchmark source names be abbreviated? | No; list all six primary sources so the reviewer trusts the pack. | Implement immediately. |
