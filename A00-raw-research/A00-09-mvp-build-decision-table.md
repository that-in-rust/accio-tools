# MVP Build Decision Table

## Decision Summary For Build

The v0.0.1 product should now move from research to implementation. The build target is a bidirectional tool router:

```text
query + recent context + tool catalog
  -> CPU router returns top 5 ranked candidates with evidence
  -> required cheap LLM judge reviews those five cards only
  -> judge returns top 1, abstain, or needs-more-metadata
  -> eval report compares modes on the 50-query subset
```

No blocking deep exploration is needed before starting the Rust workspace. The remaining unknowns can be resolved as implementation defaults, then tuned by the eval command.

## Evidence Used For Decisions

| evidence source | what it proves | build implication |
| --- | --- | --- |
| `A00-raw-research/scripts/run_tool_routing_baseline.py` | The repo already has a dependency-free lexical ranker and metric scorer. Fresh run: Recall@1 `0.413`, Recall@5 `0.6493`, MRR `0.5223`, abstention accuracy `0.0`. | Lexical mode is a baseline, not the product answer. Abstention needs an explicit judge/gate contract. |
| `A00-raw-research/benchmarks/tool-routing-subset` | The local subset has 947 tools, 50 queries, 46 route-required queries, and 4 abstention queries. | This is enough to prove routing quality without importing a full benchmark. |
| `A00-raw-research/benchmarks/tool-routing-subset/A00-06-routing-source-audit-notes.md` | All 22 local reference repos were inspected and primary/secondary sources were separated. | We can cite the local audit instead of re-reading every benchmark before build. |
| Codebase-memory scan for this repo | Found `rank_tools_for_query`, `score_predictions`, `schema_from_names`, `schema_from_votr_params`, `assign_query_ids`, and `create_query_record`. | Existing local code already contains ranking, scoring, query construction, and schema fixture helpers to port into Rust. |
| VOTR local README | Uses dense similarity, BM25, sparse expansion, weighted RRF, field-aware reranking, confidence handoff, dynamic candidate count, and abstention. | Weighted RRF and field-aware evidence are validated design references. |
| lazy-tool local README | Uses exact/name/FTS5 BM25/substring/vector candidates and returns `why_matched` style evidence. | Evidence cards should include matched fields, reason codes, and candidate scores. |
| n2-QLN local README | Uses trigger match, BM25, optional semantic search, `topK: 5`, and a confidence gate that avoids auto-execution under weak score. | Top 5 is a reasonable shortlist size; confidence gates are product features, not separate algorithms. |
| contextweaver local docs | Keeps compact choice cards and hydrates full schemas only after selection. | Candidate cards should stay compact; the judge should never see the full catalog. |
| ElBruno local README | Shows semantic top-K search plus optional LLM-assisted search and claims token savings through pre-routing. | Optional vectors are useful later; cheap LLM review fits after CPU shortlisting. |

## Resolved Build Decisions

| area | decision | known now | no-research reason | implementation default |
| --- | --- | --- | --- | --- |
| MVP product shape | Build CPU top-5 ranking plus required cheap LLM top-1 judging. | Known. | The assignment needs proof that a smaller selected tool set preserves correctness; the current baseline proves ranking alone is not enough. | Production route requires judge. CPU-only output is labeled debug preview. |
| Runtime modes | Use exactly three CPU modes: lexical BM25, schema-aware BM25, and hybrid RRF. | Known. | These are the highest PMF options already scored in `A00-07`; chain/path planning was removed from v0.0.1. | Runtime enum has only `lexical`, `schema_aware`, and `hybrid`. |
| Cargo workspace | Create `catalog-router-core-engine`, `candidate-judge-openai-adapter`, `benchmark-eval-metrics-runner`, and `router-cli-command-surface`. | Known. | The executable spec already maps responsibilities cleanly. | Core engine owns ranking; judge adapter owns judge payloads; metrics runner owns eval; command surface owns commands. |
| Catalog model | Normalize id, source id, server, name, description, tags, input schema, annotations, and unknown metadata. | Known. | Existing subset files and build scripts already use this shape. | Preserve unknown fields for evidence; reject duplicate ids. |
| Lexical scoring | Port the Python lexical TF-IDF overlap baseline into Rust first. | Known. | We need parity with the local baseline before claiming lift. | Flatten id, source id, server, name, description, tags, and schema text; return deterministic ranked scores. |
| Schema scoring | Add deterministic capability features over lexical score. | Known. | Reference repos agree that field-aware reranking matters; no domain-specific server maps are needed. | Feature boosts: operation match `+2.0`, object match `+2.0`, required parameter match `+1.5`, optional parameter match `+0.5`, namespace/server match `+0.75`, read/write alignment `+1.25`; unsafe write mismatch penalty `-3.0`. |
| Capability extraction | Infer operation, object, side effect, parameter names, and risk from metadata text and schema fields. | Known. | The assignment gives metadata, not private server knowledge. | Use token patterns from name/description/schema; never branch on benchmark source repo. |
| Hybrid fusion | Fuse lexical and schema-aware ranked lists with weighted reciprocal rank fusion. | Known. | VOTR validates weighted RRF; RRF is simple enough for five-hour delivery. | Use `score += weight / (60 + rank)`. Weights: lexical `1.0`, schema `1.4`, vector `0.6` only when enabled. |
| Vector signal | Keep vector search optional and disabled by default. | Known. | Pure vectors risk missing exact ids, params, and write/read distinctions. | Hybrid works with lexical + schema only; vector provider fixtures can be added later without changing the mode list. |
| Candidate count | Return exactly top 5 to the judge and reports. | Known. | The user wants CPU top 5 to cheap LLM top 1; n2-QLN defaults topK to 5; VOTR supports dynamic compact handoff. | CPU rankers can internally score more, but exported judge payload contains five cards maximum. |
| Judge contract | Judge is mandatory for production top-1 selection. | Known. | Without the judge, CPU rank 1 is only a ranker output, not the bidirectional architecture. | Judge output JSON: `decision`, `selected_tool_id`, `confidence`, `reason`, `needs_more_metadata`. |
| Judge model choice | Keep model configurable, not hardcoded. | Known enough. | Model availability and pricing change; tests should not depend on network. | Read `OPENAI_ROUTER_JUDGE_MODEL`; use a mock judge in tests; real adapter only runs when key and model are configured. |
| Judge payload | Send query, recent context summary, and five compact candidate cards only. | Known. | The core claim is that the LLM does not inspect the full tool universe. | Snapshot-test payload card count and absence of full catalog. |
| Abstention behavior | Treat abstention as a first-class judge decision with CPU weak-evidence flags. | Known. | Current lexical abstention accuracy is `0.0`; no-route cases exist in the subset. | Set weak-evidence flags when top score `< 2.0`, top margin `< 0.15`, no operation/object match, or write risk is ambiguous. |
| Benchmark gates | Separate build gates from product evidence targets. | Known. | Early implementation should not fail only because schema weights need tuning. | Build gate: metrics run for all modes. Evidence target: schema-aware Recall@5 should beat lexical or produce a failure bucket report. |
| Lexical parity gate | Rust lexical baseline should match Python baseline closely. | Known. | This prevents accidental metric drift. | Recall@5 within `0.02` of Python `0.6493`; MRR within `0.03` of Python `0.5223`. |
| Reporting format | Emit JSON and Markdown evidence reports. | Known. | The reviewer needs both machine-checkable metrics and presentation-ready artifacts. | Report query, mode, top 5, judge top 1, confidence, reasons, token estimate, gold match, and failure bucket. |
| Secret handling | API key is session-only and never exported. | Known. | This is required for a demo app and CLI. | Redact key-shaped values; do not persist environment values. |
| UI boundary | Rust engine first; UI can call CLI/library next. | Known. | The current task is executable spec and architecture, not full UI implementation. | `route-tools-for-query` and `evaluate-routing-subset-metrics` define the interface the UI consumes. |

## Deep Exploration Remaining Items

| question | status | build blocker | decision now | later exploration |
| --- | --- | --- | --- | --- |
| Which OpenAI judge model is cheapest and best today? | Unstable external choice. | No. | Configurable model plus mock judge. | Compare live model cost/quality before final demo polish. |
| Do the first schema weights maximize Recall@5? | Unknown until eval. | No. | Start with deterministic weights above. | Tune weights after the first schema-aware eval report. |
| Should vector retrieval ship in v0.0.1? | Optional. | No. | Disabled by default. | Add fixture-backed embeddings only after lexical and schema modes pass. |
| What frontend should expose the router? | Partly known from Confido. | No. | Use CLI/library as stable boundary. | Build the evidence console after engine outputs are stable. |
| Should active metadata lookup exist? | Valuable later. | No. | Do not build before top-5 ranking works. | Add metadata fetching after confidence gates expose low-evidence cases. |
| Should execution fallback invoke second-best tools? | Product future. | No. | v0.0.1 selects or abstains only. | Add execution fallback after the assignment proof is accepted. |

## Implementation Steps Known Now

| step | four-word target name | acceptance check |
| --- | --- | --- |
| 1 | `validate_catalog_schema_input` | Duplicate ids fail with typed errors; unknown metadata survives. |
| 2 | `rank_lexical_tools_baseline` | Rust metrics match Python lexical baseline within parity thresholds. |
| 3 | `score_schema_capability_signals` | Param/object/operation/read-write fixtures rank above distractors. |
| 4 | `fuse_hybrid_rankings_rrf` | Fusion is deterministic and exposes per-signal contributions. |
| 5 | `judge_candidate_tools_top` | Judge payload snapshot contains five cards maximum and no full catalog. |
| 6 | `evaluate_routing_subset_metrics` | Eval emits Recall@K, MRR, nDCG, abstention accuracy, and token reduction estimate. |
| 7 | `export_route_evidence_report` | Markdown and JSON reports include CPU top 5, judge decision, and failure bucket. |

## Rubber Duck Logic Check

The product needs a final top-1 answer, but CPU algorithms only rank candidates. Therefore the CPU layer should optimize shortlist recall and evidence quality, while the cheap LLM judge performs the final selection or abstention. Making the judge optional for production would collapse the architecture into a plain ranker, which is not the thesis.

The three remaining algorithms all naturally produce rankings. Lexical BM25 ranks by term evidence. Schema-aware BM25 ranks by lexical score plus capability features. Hybrid RRF ranks by fusing already-ranked lists. That means the build does not need a new retrieval family to return a top result.

The most important product truth is that missing the correct tool from the CPU top 5 is fatal. Choosing among five candidates is cheap and reviewable. So v0.0.1 should bias toward Recall@5, compact evidence cards, and abstention safety before optimizing final top-1 elegance.

## Final Build Recommendation

Start implementation immediately from `A02-routers-solutions/solution01/S01-mvp-router-executable-specs.md`. Do not spend more time collecting benchmarks or designing new retrieval families before the Rust workspace exists. The correct next artifact is a working cargo workspace with eval parity against the local Python baseline, then schema-aware lift or a clear failure bucket report.
