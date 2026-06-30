# Why This Folder Exists

This folder is the local, ignored reference shelf for tool-routing research. It holds cloned benchmark repos, router implementations, MCP gateway projects, and prior take-home/product references that are useful for analysis but should not be committed into the main repo.

The important subfolder is:

- `git-ref-repo/`: shallow or local clones of external reference repos used to study routing algorithms, benchmark design, failure modes, and evidence-console UX.

## Evidence Posture

These repos are references, not dependencies. They should inform the shape of the router, but the product implementation should remain small, testable, and grounded in our own executable specs.

On 2026-07-01, this folder was indexed with `codebase-memory-mcp` as project:

```text
Users-amuldotexe-Desktop-personal-repos-lane-accio-tools-ignore-references
```

The graph index reported `148371` nodes and `472154` edges across the ignored reference shelf. Treat that graph as a discovery accelerator only. Important claims below were checked against local repo files with direct reads and `rg`.

## Working Thesis

The best v0.0.1 direction is not a production MCP gateway or a graph workflow planner. It is a correctness-first router proof:

```text
query + recent context + tool catalog
  -> CPU candidate generator returns top 5 with scores and reasons
  -> cheap LLM judge reviews only those 5 compact cards
  -> system returns top 1 selected tool, abstain, or needs-more-metadata
  -> eval report proves recall, abstention, token reduction, and failure buckets
```

The CPU layer should optimize top-5 recall and explainability. The judge can only choose among what the CPU layer surfaced.

## Reference Repos In `git-ref-repo`

| repo | role in our study | why it was shortlisted | most useful local evidence |
| --- | --- | --- | --- |
| `VOTR` | Primary router algorithm reference | Strongest match for the assignment: MCP-scale tool retrieval, hybrid retrieval, field-aware reranking, confidence handoff, abstention/null-route guards, and ablations. | `README.md` highlights dense + BM25 + SPLADE-lite with weighted RRF, field-aware reranking, dynamic k, and robustness suites. |
| `lazy-tool` | Local-first implementation reference | Shows how to build a practical local router with exact/name search, FTS5 BM25, optional vector search, score weights, `why_matched`, and passthrough fallback. | `README.md` search algorithm section and `internal/search/weights.go`. |
| `contextweaver` | Choice-card and bounded-context reference | Shows how to turn a large tool catalog into compact `ChoiceCard`s without exposing full schemas, plus deterministic routing, bounded graphs, and benchmark framing. | `docs/tool_router.md`, `docs/benchmarks.md`, `src/contextweaver/routing/`. |
| `graph-tool-call` | Graph/path routing reference | Useful for multi-tool workflows where the answer is a chain, not one tool. Demonstrates BM25 + graph traversal + embeddings + MCP annotations via weighted RRF. | `README.md` sections on workflow retrieval, weighted RRF, graph traversal, reranking, and history-aware retrieval. |
| `mcpproxy-go` | Production gateway and eval reference | Shows the product future: one endpoint in front of many MCP servers, `retrieve_tools`, security/quarantine, UI, and graded retrieval metrics. Too broad for v0.0.1, but useful for evaluation and proxy UX. | `README.md` and `bench/metrics.go` for Recall@K, MRR, nDCG. |
| `n2-QLN` | One-meta-tool router reference | Shows a universal MCP facade with `n2_qln_call`, trigger match, BM25, optional semantic search, confidence gating, fallback chain, usage/success ranking, and boost keywords. | `README.md` 3-stage engine and `src/lib/schema.ts` search text/trigger extraction. |
| `ToolRoute` | Classifier and telemetry/product reference | Routes tasks to MCP servers and LLM models with LLM classification, value scoring, alternatives, fallback, telemetry, and optional pgvector task matching. Useful but routes services/models more than raw tools. | `README.md`, `docs/phase0-routing-inventory.md`, `src/lib/task-matcher.ts`. |
| `RAG-MCP-example` | Minimal semantic retrieval skeleton | Simple baseline for "embed tools, rank top-k, inject only relevant tools." Useful as a teaching skeleton, not enough as the final proof. | `README.md` and `src/main.py` embedding/cosine retrieval. |
| `dmcp` | Pure vector meta-tool reference | Demonstrates `search_tools` as the only exposed MCP tool, using ToolRet-trained E5 embeddings, Redis HNSW, cosine top-k, and list-changed notifications. Useful as vector-only contrast. | `README.md` architecture and search sections. |
| `ElBruno.ModelContextProtocol` | Local embedding and LLM-distillation reference | Shows local ONNX embeddings for MCP tool routing, plus a second mode where a local LLM distills verbose prompts into action phrases before search. | `README.md` two-mode router and technical details. |
| `benchmarking-tool-retrieval` | ToolRet benchmark/model reference | Provides the research backing for tool-specific retrieval models and rerankers. Useful for later embedding/reranking choices, not a v0.0.1 dependency. | `README.md` benchmark description, dense model list, reranker list, and training notes. |
| `semantic-router` | Static semantic route reference | Shows dense/sparse/hybrid route layers and thresholds. Useful for threshold/no-route ideas, but less aligned with arbitrary unknown tool catalogs. | `README.md`, `CLAUDE.md`, `semantic_router/schema.py`. |
| `MCP-Zero` | Active tool discovery and MCP dataset reference | Useful for server/tool embedding data, hierarchical discovery framing, and active tool-chain construction. More research/backlog than immediate build. | `README.md` method and MCP-tools dataset structure. |
| `ToolBench` | Tool-use benchmark and retriever reference | Provides single-tool and multi-tool data, retriever training/eval examples, and a top-k retrieval path. Better for benchmark coverage than direct router design. | `README.md`, `toolbench/inference/LLM/retriever.py`. |
| `StableToolBench` | Stable tool-use benchmark reference | Useful for stable evaluation thinking, virtual API simulation, solvable query filtering, and reduced benchmark flakiness. Not a pre-model router algorithm. | `README.md` features and StableToolEval sections. |
| `LiveMCPBench` | Real MCP benchmark reference | Provides realistic MCP tasks, annotated data, server/tool retrieval config, and top-server/top-tool settings. Useful for external validation and query/task shapes. | `README.md` env config and MCP Copilot sections. |
| `mcp-bench` | Real-world MCP task benchmark reference | Provides single-server and multi-server MCP tasks, tool usage evaluation, and LLM-judged task completion. Useful for benchmark-derived queries and multi-server cases. | `README.md` overview and task-runner commands. |
| `ToolSandbox` | Stateful tool-use benchmark reference | Useful for failure taxonomy: state dependency, canonicalization, insufficient information, hidden dependencies, and milestone evaluation. Not a router implementation. | `README.md` benchmark overview and state/tool sections. |
| `tau2-bench` | Agent/customer-service benchmark reference | Useful for multi-turn tasks, policy/tool/task separation, knowledge retrieval domains, and interaction evaluation. Not a router implementation. | `README.md` overview and knowledge retrieval sections. |
| `appworld` | API-world/MCP environment reference | Useful for API docs compression, function-calling formats, MCP server/client support, and stateful API environments. Not a routing algorithm. | `README.md` API-doc compression and MCP sections. |
| `gorilla` | Function-calling and API selection benchmark reference | Useful historical baseline for API invocation, BFCL, APIBench, relevance detection, and large API catalogs. Not a shortlist router by itself. | `README.md` Gorilla/BFCL/APIBench overview. |
| `confido-exploration-01` | Prior take-home and evidence-console UX reference | Useful as product-process evidence: Tauri app packaging, API-key-in-UI pattern, reviewer note, and "progressive disclosure" framing. Not a tool-router algorithm. | `ReadMe.md`, `NoteToReviewer.md`, `Sol-variant-01/`. |

## Tool-Routing Algorithm Landscape

| algorithm or design option | what it does | reference repos in this shelf | idea came from where | fit for our v0.0.1 |
| --- | --- | --- | --- | --- |
| Full catalog / no routing baseline | Send all tool schemas to the model and let the final LLM choose. | `contextweaver`, `mcpproxy-go`, `n2-QLN`, `VOTR` | Mostly appears as the "before" picture: large catalogs blow context budgets and degrade selection quality. | Use only as the negative control. It proves why routing exists. |
| Exact, alias, substring, and trigger matching | Fast deterministic retrieval by exact tool names, aliases, trigger words, IDs, and substring hits. | `lazy-tool`, `n2-QLN`, `contextweaver` | `lazy-tool` uses exact/name/substr fallback; `n2-QLN` starts with trigger match; ContextWeaver supports fuzzy/lexical retrieval. | Good baseline and fallback. Weak alone for paraphrase and sparse descriptions. |
| Lexical TF-IDF / BM25 ranking | Flatten tool metadata into searchable text and rank by term evidence. | `VOTR`, `lazy-tool`, `contextweaver`, `graph-tool-call`, `n2-QLN`, `mcpproxy-go` | VOTR uses BM25 in its hybrid stack; lazy-tool uses FTS5 BM25; ContextWeaver uses TF-IDF/BM25 backends; graph-tool-call uses BM25; n2-QLN uses BM25 with boost keywords; mcpproxy-go scores BM25 retrieval. | Required baseline. Transparent, deterministic, and fast, but not strong enough as the final product answer. |
| Schema-aware / capability-aware scoring | Add structured boosts or penalties for operation, object, namespace, parameters, required fields, read/write safety, and risk. | `VOTR`, `lazy-tool`, `n2-QLN`, `contextweaver`, `graph-tool-call` | VOTR field-aware reranking uses server/tool/description/parameter signals; lazy-tool normalizes capability records and search text; n2-QLN duplicates boost keywords and extracts triggers; graph-tool-call uses MCP annotations. | Main v0.0.1 CPU router. It is metadata-only, explainable, and should lift recall over lexical. |
| Pure dense embedding retrieval | Embed query and tool descriptions, then return cosine-similar top-k tools. | `RAG-MCP-example`, `dmcp`, `ElBruno.ModelContextProtocol`, `ToolBench`, `MCP-Zero`, `semantic-router` | RAG-MCP uses sentence-transformers; DMCP uses ToolRet-trained E5 + Redis HNSW; ElBruno uses local ONNX embeddings; ToolBench has a SentenceTransformer retriever; MCP-Zero ships server/tool embeddings. | Useful comparison and optional signal. Risky as the only router because it can miss exact IDs, params, namespaces, and read/write distinctions. |
| Tool-specialized dense retrieval and reranking | Use models trained/evaluated specifically for tool retrieval, plus cross-encoder rerankers. | `benchmarking-tool-retrieval`, `dmcp`, `ToolBench` | ToolRet argues general IR models are weak for tools and evaluates dense models plus rerankers; DMCP uses a ToolRet-trained E5 model; ToolBench has retriever data. | Good v0.0.2 or research direction. Too heavy for the initial local Rust proof. |
| Hybrid lexical + dense + sparse fusion | Generate several ranked lists, then fuse them with weighted reciprocal rank fusion or weighted scoring. | `VOTR`, `graph-tool-call`, `lazy-tool`, `semantic-router`, `contextweaver` | VOTR uses dense + BM25 + SPLADE-lite fused by weighted RRF; graph-tool-call uses weighted RRF across BM25, graph, embeddings, and annotations; lazy-tool merges lexical/vector candidates; semantic-router has hybrid dense/sparse routing. | Strong stretch mode. Keep vectors optional and disabled by default. |
| Field-aware reranking | Reorder candidate tools using structured overlap: name, description, server, parameters, examples, annotations, and side-effect hints. | `VOTR`, `graph-tool-call`, `contextweaver`, `n2-QLN`, `ToolRoute` | VOTR explicitly calls out field-aware reranking; graph-tool-call uses annotations; ContextWeaver reranks with history/dependencies; n2-QLN uses boosted search text; ToolRoute uses task priors and skill relevance. | Build into schema-aware scoring and evidence cards, not as a separate heavy model. |
| Confidence gates, abstention, and no-route handling | Avoid unsafe or low-confidence selections by returning no tool, more metadata, or a shortlist instead of executing. | `VOTR`, `n2-QLN`, `ToolRoute`, `semantic-router`, `lazy-tool` | VOTR has dynamic k and optional abstention; n2-QLN avoids auto-exec below score threshold; ToolRoute defers on low confidence or ambiguous matches; semantic-router stores thresholds; lazy-tool has passthrough fallback. | First-class output contract. Current lexical-only abstention is weak, so this is mandatory. |
| Cheap LLM judge or classifier after retrieval | Use a small/cheap model to judge a narrowed candidate set, classify task intent, or resolve ambiguity. | `ToolRoute`, `ElBruno.ModelContextProtocol`, `VOTR`, `contextweaver`, `mcp-bench` | ToolRoute uses a cheap classifier; ElBruno Mode 2 uses local LLM distillation; VOTR has confidence handoff; ContextWeaver lets the LLM choose among cards; MCP-Bench uses an LLM judge for eval. | Core v0.0.1 thesis: the LLM sees only top 5 compact cards, never the full catalog. |
| LLM query distillation / query expansion before search | Rewrite verbose prompts into short action phrases, then search original plus distilled phrases. | `ElBruno.ModelContextProtocol`, `ToolRoute`, `ToolBench` | ElBruno extracts 2-5 word action phrases; ToolRoute classifies tasks; ToolBench retriever corpus uses task/tool retrieval text. | Later feature. It adds an LLM before CPU retrieval, which increases moving parts. |
| Bounded ChoiceGraph / ChoiceCards | Convert a large catalog into a bounded graph, navigate via beam search, and pack compact cards without full schemas. | `contextweaver`, `graph-tool-call` | ContextWeaver builds `ChoiceGraph` and emits `ChoiceCard`s; graph-tool-call builds relationship graphs and returns chains. | Borrow card format and compact evidence. Do not make ChoiceGraph the first ranking core. |
| Graph or workflow-chain retrieval | Retrieve a chain of tools using relations like precedes, requires, complementary, history, and dependencies. | `graph-tool-call`, `ToolSandbox`, `tau2-bench`, `mcp-bench`, `LiveMCPBench` | graph-tool-call is the concrete algorithm source; the benchmark repos show why multi-step stateful tasks exist. | Roadmap. Valuable once top-1/top-5 routing is proven. |
| Active metadata discovery / server-first matching | First identify likely servers or metadata sources, then fetch or hydrate tool details on demand. | `MCP-Zero`, `LiveMCPBench`, `contextweaver`, `appworld`, `mcpproxy-go` | MCP-Zero frames active tool discovery and ships server/tool embeddings; LiveMCPBench has top-server/top-tool config; ContextWeaver hydrates schemas on selection; AppWorld compresses API docs. | v0.0.2. Useful when catalog metadata is sparse or confidence is low. |
| Universal meta-tool / proxy router | Expose one tool such as `search_tools`, `retrieve_tools`, or `n2_qln_call`, hiding the full tool catalog behind a gateway. | `dmcp`, `mcpproxy-go`, `n2-QLN`, `lazy-tool`, `contextweaver` | DMCP exposes `search_tools`; mcpproxy-go exposes one endpoint/retrieve function; n2-QLN exposes one MCP tool; lazy-tool and ContextWeaver support gateway-like surfaces. | Product future. For the take-home proof, it can obscure the selection evidence. |
| Fallback execution chain / passthrough | If the top candidate fails or retrieval returns nothing, try lower-ranked tools or expose the full catalog as a safety net. | `n2-QLN`, `lazy-tool`, `ToolRoute`, `mcpproxy-go` | n2-QLN tries up to three ranked candidates; lazy-tool has passthrough fallback; ToolRoute returns alternatives/fallback; mcpproxy-go handles gateway resilience. | Do not execute in v0.0.1. Capture as future product behavior and failure-bucket analysis. |
| Telemetry / learning-weighted ranking | Use usage count, success rate, quality, reliability, cost, trust, favorites, or user feedback to adjust future rankings. | `ToolRoute`, `n2-QLN`, `lazy-tool`, `mcpproxy-go` | ToolRoute updates value scores from outcomes; n2-QLN uses usage and success rate; lazy-tool boosts favorites/history; mcpproxy-go has activity/security surfaces. | Log now, learn later. Not useful before we have real outcomes. |
| Benchmark-derived router evaluation | Evaluate whether the required tool appears in top-k, not whether a final agent answer is perfect. | `benchmarking-tool-retrieval`, `mcpproxy-go`, `VOTR`, `contextweaver`, `LiveMCPBench`, `mcp-bench`, `ToolBench`, `StableToolBench`, `gorilla`, `ToolSandbox`, `tau2-bench`, `appworld` | ToolRet and mcpproxy-go provide retrieval metrics; VOTR/contextweaver provide router ablations; broad benchmarks contribute task/failure shapes. | Mandatory. Primary proof should be Recall@5, plus abstention accuracy, MRR/nDCG, token reduction, latency, and failure buckets. |
| Evidence-console UX | Give reviewers a local UI to see route inputs, top-5 cards, matched fields, judge decision, metrics, and exportable reports. | `confido-exploration-01`, `contextweaver`, `mcpproxy-go`, `lazy-tool` | Confido provides prior Tauri/reviewer UX pattern; ContextWeaver has ChoiceCards; mcpproxy-go has proxy UI; lazy-tool has explainable search output. | Build after the Rust/CLI evidence loop is stable. It is an N-task around the L-task. |

## Shreyas-Style LNO Cut

| category | belongs here | rationale |
| --- | --- | --- |
| L | Benchmark subset, Recall@5, schema-aware scoring, top-5 evidence cards, cheap LLM judge, abstention/failure buckets. | This proves the product thesis under time pressure. |
| N | Tauri evidence console, report export, session API key handling, UI polish. | Helps the reviewer understand the proof, but depends on engine outputs. |
| O | Production MCP gateway, OAuth, server quarantine, graph workflow planner, telemetry learning, active metadata lookup, vector infrastructure. | Important product future, but dangerous scope for v0.0.1. |

## Practical Build Implication

Start with the boring proof:

1. Port lexical BM25/TF-IDF baseline.
2. Add schema/capability-aware signals.
3. Return top 5 with `why_matched`, matched fields, risk, and scores.
4. Add the cheap LLM judge over only those five cards.
5. Run the 50-query/947-tool benchmark and report Recall@K, abstention accuracy, MRR, nDCG@10, token reduction, latency, and failure buckets.

Everything else is roadmap until this evidence loop works.
