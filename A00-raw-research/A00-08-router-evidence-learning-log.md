# Router Evidence Learning Log

## Purpose

This file captures the intermediate knowledge from the v0.0.1 algorithm selection pass. The final recommendation is in `A00-07-router-algorithm-selection-decision.md`; this log preserves the evidence trail so we can turn the decision into implementation or slides without re-reading every reference repo.

## Fixed Architecture Assumption

We assume a bidirectional tool-call router:

```text
CPU algorithm -> top 5 scored tool candidates -> cheap LLM judge -> top 1 tool or abstain
```

The CPU stage should maximize top-5 recall and provide deterministic evidence. The cheap LLM stage should resolve ambiguity, check the CPU's judgment, and decide whether to pick, abstain, or request more metadata.

## Current Local Baseline

The repo already has a dependency-free lexical baseline in `A00-raw-research/scripts/run_tool_routing_baseline.py`.

It flattens:

- tool id;
- source tool id;
- server name;
- tool name;
- description;
- tags;
- input schema.

It then tokenizes text, computes IDF-like weights, scores overlapping query terms, and reports recall/MRR/nDCG/abstention metrics.

Fresh run result:

```json
{
  "queries": 50,
  "route_required_queries": 46,
  "abstention_queries": 4,
  "recall_at_k": {
    "1": 0.413,
    "3": 0.5797,
    "5": 0.6493,
    "10": 0.679
  },
  "mrr": 0.5223,
  "ndcg_at_10": 0.5553,
  "abstention_accuracy": 0.0
}
```

Learning: lexical is a useful floor, but the top-5 result is not strong enough for a convincing demo, and abstention needs explicit design.

## Dataset Evidence

The local subset in `A00-raw-research/benchmarks/tool-routing-subset` contains:

- 947 tool metadata records;
- 50 queries;
- 46 route-required queries;
- 4 abstention queries;
- primary metric: required-tool recall@K;
- secondary metrics: abstention accuracy, nDCG@10, token reduction, latency.

Source contribution:

| Source | Queries | Tools | What it contributes |
| --- | ---: | ---: | --- |
| VOTR | 22 | 116 | Ambiguity, robustness, no-route, and multi-tool routing cases. |
| contextweaver | 8 | 83 | Routing gold, bounded card ideas, deterministic context packaging. |
| graph-tool-call | 8 | 158 | Tool-chain and OpenAPI selection examples. |
| mcpproxy-go | 6 | 45 | Frozen MCP corpus and graded retrieval relevance labels. |
| LiveMCPBench | 3 | 525 | Real MCP-style annotated tasks. |
| mcp-bench | 3 | 20 | Single-server and multi-server task-runner descriptions. |

Failure modes covered:

- ambiguous tool names;
- near-duplicate capability;
- noisy user language;
- typos and abbreviations;
- unsupported intent;
- abstention required;
- multi-tool single-turn;
- ordered subtasks;
- namespace breadth;
- exact route gold;
- read/write distinction;
- multi-server MCP tasks;
- graded relevance;
- hard negatives.

Learning: the benchmark slice is enough to evaluate pre-model routing quality. We should not spend v0.0.1 time trying to import a full external benchmark.

## Reference Repo Learnings

| Repo | Method signal | Learning for our router |
| --- | --- | --- |
| VOTR | Hybrid retrieval: dense similarity, BM25, SPLADE-lite, weighted RRF, field-aware reranking, dynamic K, abstention. | Best technical inspiration for a correctness-first router. It validates hybrid scoring and field-aware ranking. |
| lazy-tool | Local-first exact/name/FTS5 BM25/substring/vector scoring, `why_matched`, passthrough fallback. | Copy the explainability style. `why_matched` is the easiest way to make reviewer trust the CPU layer. |
| n2-QLN | Trigger match, BM25, optional semantic search, usage/success feedback, confidence gate, fallback chain. | Confidence gating should be in v0.0.1, but usage learning should wait. |
| graph-tool-call | BM25 plus graph traversal plus optional embeddings and annotations; returns workflow chains. | Valuable roadmap for multi-step tasks, not the first top-1 selection demo. |
| DMCP | Pure vector search with ToolRet-trained E5 model and Redis HNSW. | Strong semantic retrieval reference, but too infra-heavy and too vector-only for v0.0.1. |
| ElBruno.ModelContextProtocol | Local embedding router and optional local LLM query distillation. | Use the idea of a cheap LLM as reviewer, but avoid adding LLM distillation before the CPU layer in v0.0.1. |
| RAG-MCP-example | Simple embed, rank, inject, generate loop. | Good educational skeleton, too thin for assignment proof. |
| MCP-Zero | Active tool discovery dataset and server/tool embeddings. | Good framing for active metadata discovery, not necessary for initial implementation. |
| semantic-router | Static semantic route layer with utterance examples and thresholding. | Useful threshold/abstention concept, poor fit for arbitrary unknown tool catalogs. |
| contextweaver | ChoiceCards, deterministic beam-search, context firewall, bounded tool surface. | Borrow compact cards and traceability, but not as the main retrieval algorithm. |
| mcpproxy-go | One `retrieve_tools` function, proxy UI, security and quarantine. | Product future, not v0.0.1. The proxy can obscure the routing proof. |
| ToolRoute | LLM classifier plus value score over quality, reliability, efficiency, cost, trust. | Good product scoring idea, but it routes server/model choices rather than top-5 tool metadata. |
| ToolRet / benchmarking-tool-retrieval | Retrieval benchmark and specialized embedding/reranker models. | If we add embeddings later, use tool-specialized models or rerankers. Do not start here. |
| LiveMCPBench | Large-scale MCP benchmark with server/tool retrieval config. | Useful for later evaluation breadth. Local subset already captured enough for now. |
| ToolBench / StableToolBench | Large API/tool-use and stable simulated API benchmarks. | Better for end-to-end agents than this pre-model routing slice. |
| ToolSandbox | Stateful tool-use evaluation with milestone DAGs and hidden dependencies. | Great failure taxonomy for later chain evaluation. |
| AppWorld | API docs, app descriptions, on-need doc access, MCP server support. | Supports future active metadata discovery and doc lookup ideas. |
| mcp-bench | Real MCP servers and single/multi-server evaluation tasks. | Good realistic task flavor; use as benchmark evidence, not routing algorithm inspiration. |

## Algorithm Families Found

1. Full catalog / no routing baseline.
2. Exact / alias / substring / lexical TF-IDF or BM25.
3. Schema/capability-aware deterministic scoring.
4. Pure dense embedding / vector retrieval.
5. Hybrid lexical + schema + embedding fusion.
6. Trigger + BM25 + semantic + confidence gates.
7. Graph/path retrieval.
8. ChoiceGraph / bounded ChoiceCards.
9. LLM query distillation before search.
10. Static semantic route layer.
11. Active metadata discovery.
12. Universal proxy / one meta-tool.
13. Telemetry or learning-weighted routing.

Learning: many repos combine methods, but for our v0.0.1 we should separate **algorithm slots** from **router features**. Confidence gating and reason cards are features. Schema-aware BM25 and hybrid RRF are candidate-generation algorithms.

## Decision Criteria Used

| Criterion | Why it matters |
| --- | --- |
| Top-5 recall potential | Missing the right tool is fatal; extra candidates are cheap. |
| Explainability | The reviewer needs to see why the CPU router made the shortlist. |
| Metadata-only fit | The assignment emphasizes unknown servers and tool metadata. |
| Five-hour feasibility | A small working proof beats an impressive half-built proxy. |
| Cheap LLM compatibility | CPU output must be compact enough for low-cost review. |
| Abstention support | No-route cases are part of the local subset and current baseline fails them. |
| Implementation lift over baseline | We need a visible delta beyond lexical overlap. |

## Final Learning

The best v0.0.1 is not "BM25 vs embeddings vs graph." It is:

```text
schema-aware lexical scoring as the main engine
  + lexical baseline for proof
  + optional hybrid RRF for stretch
  + confidence gate
  + cheap LLM judge
  + evidence cards
```

This is enough to present as a real router architecture while staying grounded in something that can be implemented and evaluated quickly.

