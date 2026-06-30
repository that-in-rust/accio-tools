# TDD Progress Journal

- Task: Select v0.0.1 CPU router algorithms for bidirectional tool routing
- Created: 2026-06-30 14:44:39Z
- Updated: 2026-06-30 14:50:47Z
- Current Phase: Refactor
- Status: active

## Sessions

### Session: 2026-06-30 14:44:48Z

#### Current Phase: Red

#### Tests Written:
- algorithm_selection_table: pending - single verbose table must score all discovered algorithms and choose at most three

#### Implementation Progress:
- A00-raw-research and ignore-references scan not yet complete

#### Current Focus:
Research all local evidence to pick no more than three v0.0.1 CPU algorithms for a CPU top-5 plus cheap LLM top-1 bidirectional router

#### Next Steps:
- Run codebase-memory scan on current repo, then verify A00 research docs and reference repos directly

#### Context Notes:
- Assume bidirectional flow is fixed: CPU candidate generator returns top 5 with reasons; cheap LLM judge scores top 5 and selects top 1 or abstains/escalates.

#### Performance/Metrics:
- Target v0.0.1: highest reviewer signal in limited time, not production MCP gateway completeness

### Session: 2026-06-30 14:45:41Z

#### Current Phase: Red

#### Tests Written:
- codebase_memory_scan: passing - Indexed current repo at /tmp/codex-code-intel/codebase-memory/accio-tools-20260630-201457 with 1000 nodes and found local ranking functions

#### Implementation Progress:
- A00-raw-research/scripts/run_tool_routing_baseline.py: build_tool_index, rank_tools_for_query, score_predictions identified by graph and to be verified by direct reads

#### Current Focus:
Extract algorithm candidates and evidence from local repo and reference shelf

#### Next Steps:
- Read A00 numbered research docs and reference repo README/docs for all algorithm families

#### Context Notes:
- Local baseline evidence: lexical token overlap with IDF exists and currently reports Recall@5 0.6493, Recall@10 0.679, MRR 0.5223, nDCG@10 0.5553, abstention 0.0 on the subset.

#### Performance/Metrics:
- Graph scan confirms current code surface; direct file reads still required for final claims.

### Session: 2026-06-30 14:49:35Z

#### Current Phase: Green

#### Tests Written:
- reference_repo_inventory: passing - A00 docs plus 21 ignore-reference repos inspected for CPU routing methods and benchmark-only repos separated

#### Implementation Progress:
- Identified algorithm families: lexical/BM25, field/schema scoring, dense vector, hybrid fusion, trigger+confidence, graph/path, ChoiceGraph, static semantic routes, active metadata discovery, universal proxy, telemetry learning, LLM query distillation.

#### Current Focus:
Score all discovered algorithms and select at most three CPU candidates for v0.0.1

#### Next Steps:
- Synthesize a single table with PMF/fit scores, sources, mechanics, failure modes, and chosen/not-chosen status

#### Context Notes:
- Benchmark repos are evidence sources, but v0.0.1 should not chase full benchmark harness breadth. CPU top-5 needs high recall plus explainable reasons before cheap LLM top-1 judgment.

#### Performance/Metrics:
- Lexical baseline: Recall@1 0.413, Recall@3 0.5797, Recall@5 0.6493, Recall@10 0.679, MRR 0.5223, abstention 0.0.

### Session: 2026-06-30 14:50:47Z

#### Current Phase: Refactor

#### Tests Written:
- algorithm_selection_table: passing - Final synthesis will score every discovered method family and marks exactly three v0.0.1 picks: schema-aware BM25 scorer, lexical baseline, hybrid RRF optional upgrade

#### Implementation Progress:
- Decision: v0.0.1 main router should be schema-aware BM25/TF-IDF weighted scoring over name, description, server, tags, JSON schema, required params, and read/write/capability hints; cheap LLM judges only the CPU top-5 cards.
- Keep lexical baseline because it is already runnable and exposes lift over Recall@5 0.6493; keep hybrid RRF as optional upgrade after schema scoring, not before.
- Do not pick graph/path, universal proxy, static semantic routes, or active metadata discovery for v0.0.1 because they either solve a different product surface or need more than the five-hour proof window.

#### Current Focus:
(not set)

#### Next Steps:
- Use this table to implement three router modes only if coding starts: lexical_baseline, schema_weighted_bm25, hybrid_rrf_optional_embedding.

#### Context Notes:
- Confidence gating should be folded into the selected router output contract, but it is not counted as one of the three CPU candidate-generation algorithms.

#### Performance/Metrics:
- Selection count: 3 chosen algorithms maximum; all other discovered method families scored as not chosen or future/backlog.
