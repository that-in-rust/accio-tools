# Router Algorithm Selection Decision

## Governing Thought

For v0.0.1, the router should not try to be a full MCP gateway, a graph planner, or a universal executor. The highest-leverage slice is a **bidirectional tool-call router**:

```text
user query + recent context + tool metadata
  -> CPU candidate generator returns top 5 with scores and reasons
  -> cheap LLM judge reviews only those top 5 tool cards
  -> system returns top 1 selected tool, or abstains/escalates
```

This gives the reviewer the clearest proof: the expensive model never sees the full tool universe, the CPU layer does the scalable narrowing, and the cheap LLM only arbitrates a small evidence surface.

## V0.0.1 Decision

Pick no more than three CPU algorithms:

1. **Main router:** schema-aware BM25 / TF-IDF weighted scorer.
2. **Baseline router:** exact / alias / substring / lexical TF-IDF scorer.
3. **Optional upgrade:** hybrid weighted RRF fusion with an optional embedding signal.

Confidence gating, abstention, fallback, and evidence cards should be part of the output contract, but they should not count as separate CPU candidate-generation algorithms.

## Why This Is The Shreyas-Style V0.0.1

The L-task is not making the most sophisticated router. The L-task is proving that the approach works under time pressure. That means:

- high required-tool recall in the top 5;
- explainable scores and matched fields;
- visible lift over a plain lexical baseline;
- credible abstention behavior for no-route queries;
- a cheap LLM judge that only sees compact, scored candidates.

The current local baseline proves why this matters. The dependency-free lexical baseline over the 50-query subset reports:

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
  "abstention_accuracy": 0.0,
  "baseline": {
    "name": "lexical_tfidf_overlap",
    "threshold": 2.0,
    "max_k": 10
  }
}
```

So lexical retrieval is good enough as a floor, but not good enough as the product answer.

## Algorithm Score Table

PMF score is a 1-100 product judgment score for this assignment. It blends reviewer signal, five-hour feasibility, expected routing lift, explainability, and fit with the CPU-top-5 plus cheap-LLM-top-1 architecture.

| PMF score | Algorithm / method | Evidence in local refs | How it works | V0.0.1 call |
| ---: | --- | --- | --- | --- |
| **94** | **PICK #1: Schema-aware BM25 / TF-IDF weighted scorer** | `A00-04-tool-reader-architecture-options.md`, VOTR, graph-tool-call, lazy-tool, n2-QLN | CPU indexes `server_name`, `tool.name`, description, tags, schema fields, required params, and hints like read/write/destructive. It scores operation match, object match, parameter overlap, exact IDs, namespace/server evidence, and side-effect alignment, then returns top 5 with reason codes. Cheap LLM only judges those 5 cards. | **Use as the main router.** This is the most assignment-shaped algorithm: metadata-only, fast, explainable, no hardcoded domain knowledge, and directly better than pure lexical. |
| **84** | **PICK #2: Exact / alias / substring / BM25 lexical baseline** | `scripts/run_tool_routing_baseline.py`, lazy-tool, n2-QLN, VOTR ablations | CPU tokenizes query and flattened tool metadata, uses exact/name hits plus IDF/BM25-like scoring, and ranks tools. It handles IDs, names, obvious nouns, schema words, and abbreviations better than embeddings alone. | **Keep as baseline mode.** It proves lift and gives a fallback. It is too weak alone because it misses paraphrase, vague intent, and no-route abstention. |
| **91** | **PICK #3: Hybrid weighted RRF fusion with optional embedding signal** | VOTR, lazy-tool, DMCP, ElBruno, graph-tool-call | CPU gets separate ranked candidate lists from lexical search, schema/capability scoring, optional vector similarity, and optional sparse expansion. It fuses ranks with weighted reciprocal rank fusion, then returns top 5 with per-signal explanations. | **Use as optional upgraded mode if time permits.** It is the strongest retrieval story, but only after schema-aware scoring exists. Do not let embeddings become the whole demo. |
| 88 | Confidence gate / abstention / fallback ranking | n2-QLN, VOTR, lazy-tool, contextweaver | If the top score or top-vs-second margin is weak, return `abstain`, ask for clarification, or give cheap LLM a no-tool option. If execution later fails, try the next candidate. | **Fold into the router output contract.** It fixes the local baseline's biggest gap: abstention accuracy is currently 0.0. Do not count it as an algorithm slot. |
| 78 | Pure dense embedding / vector retrieval | RAG-MCP-example, DMCP, ElBruno Mode 1, MCP-Zero, ToolRet | Embed query and tool text; rank by cosine/HNSW nearest neighbors. Strong for paraphrase and fuzzy intent. | **Use only as a signal inside hybrid.** Risky as the main v0.0.1 algorithm because it can miss exact IDs, required params, namespaces, and read/write differences. |
| 77 | Graph / path retrieval | graph-tool-call, ToolSandbox, ToolBench multi-tool traces | Tools are graph nodes. Edges represent prerequisite, complementary, follows, requires, shared parameter, or workflow relationships. Retrieval returns a chain instead of independent top-K tools. | **Roadmap, not v0.0.1.** Very relevant for multi-tool tasks, but too much if the first demo is top-1 tool selection. |
| 75 | LLM query distillation / multi-query expansion | ElBruno Mode 2, ToolRoute | Cheap/local LLM rewrites a verbose prompt into short action phrases. CPU searches original query plus phrase queries, then merges results. | **Later feature.** Our architecture already uses a cheap LLM after CPU top 5. Adding an LLM before CPU search increases moving parts. |
| 72 | Contextweaver ChoiceGraph / beam-search ChoiceCards | contextweaver | Build a bounded graph from the catalog, beam-search candidates, and return compact `ChoiceCard`s instead of full schemas. | **Borrow the card format, not the routing core.** Good presentation and context packing, but not the strongest retrieval algorithm for our proof. |
| 70 | Active metadata discovery / on-demand docs | MCP-Zero, AppWorld, LiveMCPBench | Start with shallow metadata, then fetch server summaries, API docs, examples, or extra schema information when first-pass confidence is low. | **v0.0.2 active reader.** Conceptually strong but too broad for a five-hour v0.0.1 unless mocked. |
| 64 | Telemetry / learning-weighted routing | n2-QLN, ToolRoute, lazy-tool | Add usage count, success rate, reliability, cost, trust, favorites, or user feedback into future scores. | **Log now, learn later.** Good product loop, weak first proof because we do not yet have real outcomes. |
| 58 | Universal proxy / one meta-tool router | mcpproxy-go, DMCP, n2-QLN, lazy-tool | Final agent sees one tool like `search_tools`, `retrieve_tools`, or `n2_qln_call`; proxy hides retrieval and execution. | **Not the v0.0.1 proof.** Productizable, but it obscures candidate selection and evaluation. |
| 54 | Static semantic route layer | semantic-router | Define known routes with example utterances and route by semantic similarity. | **Wrong abstraction for unknown catalogs.** Useful for fixed intents, but this assignment needs arbitrary tool metadata. |
| 45 | Full catalog / no routing baseline | contextweaver, mcpproxy-go, assignment anti-pattern | Inject every schema into the final model and let it search, rank, and reason in one context. | **Use only as the before picture.** It proves why routing exists. |

## Recommended V0.0.1 Output Shape

Every CPU router should return a structured top-5 packet:

```json
{
  "query": "Checkout started failing after release 1242",
  "router": "schema_weighted_bm25",
  "candidates": [
    {
      "rank": 1,
      "tool_id": "github.get_release",
      "score": 0.87,
      "matched_fields": ["name", "description", "input_schema.release_id"],
      "capability_match": {
        "operation": "get",
        "object": "release",
        "side_effect": "read"
      },
      "why_matched": [
        "release matched query term",
        "required parameter release_id aligns with 1242",
        "read-only tool matches investigation intent"
      ],
      "risk": "low"
    }
  ],
  "abstention": {
    "should_abstain": false,
    "reason": null
  }
}
```

The cheap LLM then receives only these cards and returns:

```json
{
  "selected_tool_id": "github.get_release",
  "confidence": 0.82,
  "reason": "Best match for inspecting release 1242 before tracing downstream failures.",
  "needs_more_metadata": false
}
```

## Build Order

1. Keep the existing lexical baseline runnable.
2. Add schema/capability feature extraction.
3. Add weighted scoring and reason codes.
4. Add cheap LLM top-5 judge behind an API key.
5. Add confidence gate and no-tool candidate.
6. Add optional hybrid RRF mode if there is still time.

## What Not To Build First

- Do not build a production MCP gateway.
- Do not build graph workflow planning first.
- Do not make embeddings the only retrieval signal.
- Do not make the cheap LLM read all tools.
- Do not optimize telemetry before we have outcomes.

