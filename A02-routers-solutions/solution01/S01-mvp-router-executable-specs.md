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
| Failure modes | Missing required tool in CPU top 5, wrong LLM top 1, unsafe write exposure, abstention miss, malformed catalog, malformed query file, judge API failure. |
| Performance and reliability limits | CPU ranking p95 <= 250 ms for 1,000 tools and 50 benchmark queries on a developer laptop; eval command exits non-zero on malformed fixtures. |
| Language/runtime constraints | Rust cargo workspace in `A02-routers-solutions/solution01`; CPU-only tests run without network or OpenAI key. |

## Workspace Shape

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

## Executable Requirements

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
**THEN** the UI and CLI SHALL label the route as `cpu_only_debug_preview`
**AND** SHALL show CPU top 5 ranking without claiming production top-1 judgment
**SHALL** block production route export until the judge is available.

### REQ-MVP-008.0: Abstain safely

**WHEN** the CPU ranking has weak evidence or the LLM judge determines no candidate satisfies the query
**THEN** the system SHALL return an abstain decision with reason and confidence
**AND** SHALL count abstention decisions in benchmark metrics
**SHALL** prefer abstention over unsafe write exposure when write intent is ambiguous.

### REQ-MVP-009.0: Evaluate benchmark subset

**WHEN** a developer runs eval against `A00-raw-research/benchmarks/tool-routing-subset`
**THEN** the system SHALL report Recall@1, Recall@3, Recall@5, Recall@10, MRR, nDCG@10 where graded labels exist, abstention accuracy, average selected candidate count, and token reduction estimate
**AND** SHALL compare lexical BM25, schema-aware BM25, and hybrid RRF modes
**SHALL** write JSON and Markdown reports.

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

## Test Matrix

| req_id | test_id | type | assertion | target |
| --- | --- | --- | --- | --- |
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

## TDD Plan

### 1. STUB

- Create cargo workspace and crate shells.
- Add failing tests for catalog validation, lexical ranking, schema scoring, hybrid fusion, judge payload shape, eval metrics, and runtime mode list.
- Add fixture copies or relative fixture loading for `A00-raw-research/benchmarks/tool-routing-subset`.

### 2. RED

- Run `cargo test --workspace`.
- Expected failures: missing catalog types, missing router mode enum, missing scorer functions, missing judge trait, missing eval runner.
- Record the first failure per requirement ID before implementation.

### 3. GREEN

- Implement `ToolCatalogRecordData`, `RouteQueryInputData`, `CandidateEvidenceCardData`, `JudgeDecisionOutputData`, and `RouterTypedErrorKind`.
- Implement `rank_lexical_tools_baseline` to match the existing Python baseline behavior closely enough for fixture parity.
- Implement `score_schema_capability_signals` with deterministic weights and evidence output.
- Implement `fuse_hybrid_rankings_rrf` using lexical and schema ranks first; leave vector disabled unless configured.
- Implement mock judge and payload-shape enforcement before real OpenAI adapter.
- Implement eval metrics and report writing.

### 4. REFACTOR

- Normalize scorer interfaces behind a `RouteToolsForQuery` trait.
- Keep four-word naming for every new public implementation symbol.
- Split evidence-card construction from ranking math.
- Move benchmark fixture paths into CLI config.

### 5. VERIFY

- Run full workspace tests and quality gates.
- Run eval in CPU-only mode and compare lexical baseline against current Python baseline metrics.
- Run judge tests with mock judge; run real OpenAI smoke test only when key is explicitly supplied.

## Quality Gates

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p router-cli-command-surface -- evaluate-routing-subset-metrics --dataset ../../A00-raw-research/benchmarks/tool-routing-subset --mode lexical`
- `cargo run -p router-cli-command-surface -- evaluate-routing-subset-metrics --dataset ../../A00-raw-research/benchmarks/tool-routing-subset --mode schema-aware`
- `cargo run -p router-cli-command-surface -- evaluate-routing-subset-metrics --dataset ../../A00-raw-research/benchmarks/tool-routing-subset --mode hybrid`
- Every `REQ-MVP-*` ID has at least one linked test.
- No `TODO`, `STUB`, or `FIXME` introduced in implementation commits.
- No performance claim is accepted without an eval command or test.
- Runtime mode list contains only lexical BM25, schema-aware BM25, and hybrid RRF.
- Judge payload snapshot proves the full catalog is not sent to the LLM judge.

## Resolved Defaults For Build

| decision | default | verification |
| --- | --- | --- |
| Judge model | Read `OPENAI_ROUTER_JUDGE_MODEL`; use mock judge for tests; real adapter requires explicit key and model. | Judge tests run offline; real smoke test is opt-in. |
| Schema weights | Operation `+2.0`, object `+2.0`, required parameter `+1.5`, optional parameter `+0.5`, namespace/server `+0.75`, read/write alignment `+1.25`, unsafe write mismatch `-3.0`. | Unit fixtures prove feature boosts and penalties change rank as expected. |
| Hybrid RRF | Use `score += weight / (60 + rank)` with lexical `1.0`, schema `1.4`, vector `0.6` only when enabled. | Fusion tests prove deterministic ordering and per-signal evidence. |
| Vector search | Disabled by default; optional fixture/provider only. | Hybrid tests pass with no vector provider. |
| MVP success gate | Build passes when all modes emit metrics; product claim requires schema-aware Recall@5 to beat lexical or a failure bucket report. | Eval report compares lexical, schema-aware, and hybrid modes. |
| Lexical parity gate | Rust lexical Recall@5 within `0.02` of Python `0.6493`; MRR within `0.03` of Python `0.5223`. | CPU eval compares against checked-in baseline numbers. |
| UI boundary | Rust owns engine, CLI, judge adapter, and eval output; UI consumes CLI/library later. | `route-tools-for-query` and `evaluate-routing-subset-metrics` are stable enough for the evidence console. |

## Rubber Duck Debugging

The most important thing is that all runtime router modes produce **ranked CPU candidates**. Lexical BM25 naturally ranks. Schema-aware BM25 ranks because it is a weighted scoring extension of lexical. Hybrid RRF ranks because its job is to fuse ranked lists.

The LLM judge is not a mode. It is the second half of the product contract. If the judge is missing, the system can still show CPU evidence for debugging, but it cannot honestly claim a production top-1 decision.

Workflow planning is out of v0.0.1 because it changes the problem from ranking candidate tools to coordinating tool chains. That may become valuable later, but it would weaken the MVP by splitting evaluation between top-1 tool choice and workflow correctness.
