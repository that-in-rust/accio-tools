# Tool Routing Deep Exploration

Premise check: the raw assignment notes say connected MCP
servers are unknown in advance. The router gets only name,
description, and schema metadata; hardcoded categories are
disallowed; the harness should stay minimal; and evaluation
quality is the highest-leverage work. I read the repos through
that lens, using CodeGraphContext where it indexed cleanly and
then verifying claims against source files directly.

## 1. Tool Routing Philosophy And Architecture Across The Repos

**VOTR**

```text
User query
    |
    v
+------------------+
| RouterEngine     |
+------------------+
    | embed server and tool intent text
    | run dense + BM25 + SPLADE-lite rankers
    v
+------------------+
| Weighted RRF     |
+------------------+
    | field-aware rerank
    | intent guardrails
    | adaptive K
    v
+------------------+
| Candidate tools  |
| or abstention    |
+------------------+
```

- Philosophy: VOTR treats tool routing as information retrieval,
  not as a gateway UX problem. The model should see a compact
  candidate set only after a hybrid ranker has used dense
  similarity, lexical match, sparse expansion, field weights,
  and uncertainty gates.
- Architecture: `src/mcp_router/retrieval/engine.py` builds the
  route from server/tool embeddings, BM25, SPLADE-lite, weighted
  reciprocal-rank fusion, field-aware reranking, adaptive K,
  ambiguity checks, confidence downgrades, and null-route or
  abstention behavior.
- The core unit is still a candidate tool, but VOTR knows the
  difference between "top tool is strong" and "many tools are
  close." `adaptive_k.py` grows or shrinks K from the score
  head, while the engine downgrades confidence when candidates
  overlap too much.
- `retrieval/hybrid.py` keeps the lexical corpus simple: server
  context plus tool name and description. That makes it close to
  the assignment constraint: route from metadata instead of live
  tool calls or hardcoded product logic.
- `field_scoring.py` is an important idea for unknown MCP
  servers: server name, server summary, tool name, description,
  parameters, and explicit server hints should not be treated as
  equal fields.
- Assignment inference: VOTR is the closest architectural
  reference for the router itself. The useful wedge is "hybrid
  retrieval with an honest uncertainty policy," not the FastAPI
  service wrapper.

**MCP-Zero**

```text
User task
    |
    v
+-------------------+
| LLM extracts      |
| desired server +  |
| tool descriptions |
+-------------------+
    |
    v
+-------------------+
| Server embedding  |
| match             |
+-------------------+
    |
    v
+-------------------+
| Tool embedding    |
| match in servers  |
+-------------------+
    |
    v
+-------------------+
| Best tool         |
+-------------------+
```

- Philosophy: MCP-Zero makes the model actively describe the
  server/tool it wants, then uses embedding search over a large
  tool corpus. It shifts the search query from the raw user task
  to an LLM-produced "tool need" summary.
- Architecture: `MCP-zero/matcher.py` extracts
  `<tool_assistant>` server and tool descriptions, embeds them,
  ranks servers first, then ranks tools only inside the server
  shortlist.
- The scoring formula in `match_tools` combines server score and
  tool score, which encodes a useful belief: a tool is less
  plausible if it comes from an irrelevant server, even when its
  own description has local overlap.
- `reformatter.py` turns tools into function definitions named
  `mcp_{server}_{tool}`. That keeps the server namespace visible
  in the model-facing function name when the same operation name
  appears across servers.
- The repo is research-shaped. Its README and experiment scripts
  emphasize a dataset of hundreds of servers and thousands of
  tools, plus a top-server and top-tool matching experiment,
  rather than a production proxy.
- Assignment inference: the best idea is hierarchical retrieval
  server -> tool. The risky idea is relying on an LLM to write
  the search query before any retrieval; that adds cost and a
  new failure mode unless it is an optional rerank/decomposition
  step.

**graph-tool-call**

```text
OpenAPI / MCP / code tools
    |
    v
+------------------+
| Tool graph       |
| categories       |
| edges            |
+------------------+
    |
    v
+------------------+
| Retrieval        |
| BM25             |
| embeddings       |
| annotations      |
| graph traversal  |
+------------------+
    |
    v
+------------------+
| wRRF + graph     |
| candidate inject |
+------------------+
    |
    v
+------------------+
| Tool chain       |
| and prerequisites|
+------------------+
```

- Philosophy: graph-tool-call argues that a single nearest tool
  is often the wrong abstraction. The relevant object can be a
  workflow: prerequisites, complementary tools, read/write
  sequencing, or a chain of operations.
- Architecture: `docs/architecture/overview.md` describes
  retrieval tiers: BM25 plus graph plus annotations, then query
  expansion, then intent decomposition. `retrieval/engine.py`
  fuses keyword, embedding, annotation, and graph channels and
  injects graph candidates so relationship evidence does not get
  erased by vector ranking.
- `graph_search.py` builds category and reverse-token indexes
  from the graph, then scores tools by resource/category matches
  plus intent signals such as read/write/delete. This is not
  merely a semantic search over descriptions.
- `workflow.py` plans chains by finding a target tool, adding
  same-category prerequisites, topologically sorting, and
  optionally asking an LLM to fill gaps.
- MCP ingestion keeps annotations, and the graph can link across
  sources. That matters for assignment cases where "look up
  issue, then create comment" is a more realistic success
  condition than "retrieve create_comment somewhere."
- Assignment inference: graph-tool-call is high leverage for
  evaluation design. Even if the assignment implementation stays
  simple, the eval should include tasks whose success requires a
  viable path, not only one gold tool.

**mcpproxy-go**

```text
Many upstream MCP servers
    |
    v
+------------------+
| Discovery        |
| permissions      |
| quarantine       |
+------------------+
    |
    v
+------------------+
| Bleve BM25 index |
| names, desc,     |
| params, tags     |
+------------------+
    |
    v
+------------------+
| retrieve_tools   |
+------------------+
    |
    v
+------------------------------+
| call_tool_read / write /     |
| destructive / code_execution |
+------------------------------+
```

- Philosophy: mcpproxy-go is an operational proxy. Its wedge is
  reducing model tool overload while adding control surfaces:
  quarantine, permissions, scanner behavior, desktop/tray UX,
  and explicit call modes.
- Architecture: `internal/server/mcp_routing.go` has direct
  mode, call-tool mode, and code-execution mode. Direct mode
  exposes upstream tools. Routed modes expose `retrieve_tools`
  and controlled invocation tools such as `call_tool_read`,
  `call_tool_write`, and `call_tool_destructive`.
- `internal/index/bleve.go` builds a search document from tool
  name, full tool name, server name, description, params,
  schema, hash, tags, and searchable text. The search strategy
  boosts exact tool names, exact full names, prefixes,
  wildcards, and full-text matches.
- `internal/upstream/manager.go` filters disconnected, disabled,
  and quarantined servers and parses calls through a
  `server:tool` naming convention.
- This repo thinks about "who is allowed to call what" and "how
  dangerous is this call" more than most retrieval-first repos.
  That is product-relevant but not central to the assignment.
- Assignment inference: borrow the idea of preserving
  read/write/destructive metadata in returned candidates. Avoid
  building a real multi-server proxy, tray app, scanners,
  quarantine, or code-execution sandbox for the take-home.

**n2-QLN**

```text
Tool definitions
    |
    v
+------------------+
| SQLite registry  |
| triggers         |
| embeddings       |
| usage stats      |
+------------------+
    |
    v
+------------------+
| n2_qln_call      |
| one MCP tool     |
+------------------+
    |
    v
+-----------------------------+
| trigger match -> BM25 ->    |
| optional semantic vector    |
+-----------------------------+
    |
    v
+-----------------------------+
| confidence gate -> execute  |
| or show candidates/fallback |
+-----------------------------+
```

- Philosophy: n2-QLN compresses a large tool universe behind one
  MCP tool. It optimizes the agent interface more aggressively
  than the assignment asks: the model calls `n2_qln_call`, which
  may search, execute, validate, create, or inspect tools.
- Architecture: `src/lib/router.ts` runs three retrieval stages:
  trigger match, BM25 keyword scoring, and optional semantic
  vector scoring, then merges with usage, success, recency, and
  source weights. It also injects exploration to avoid starving
  rarely used tools.
- `qln-call.ts` exposes the one-tool surface and has a low-
  confidence branch: below threshold it returns search results
  instead of immediately executing. It also has a fallback chain
  that tries additional candidates.
- `registry.ts`, `vector-index.ts`, and `executor.ts` show a
  product-minded loop: register tools, precompute embeddings,
  update usage/success, and use a circuit breaker around
  execution.
- The main assignment caveat is `schema.ts` and `validator.ts`:
  categories and naming assumptions are hardcoded. The raw
  assignment notes explicitly say hardcoded categories are
  disallowed.
- Assignment inference: borrow the simple multi-stage ranker and
  confidence gate. Do not borrow the one universal executor as
  the primary shape, and do not bake in category lists.

**lazy-tool**

```text
Existing MCP configs
    |
    v
+------------------+
| Local catalog    |
| tools, prompts,  |
| resources        |
+------------------+
    |
    v
+------------------+
| SQLite FTS5      |
| optional vector  |
| exact/substring  |
| usage boosts     |
+------------------+
    |
    v
+-----------------------------+
| search_tools returns hits   |
| with why_matched and        |
| next tool/input hint        |
+-----------------------------+
    |
    v
+-----------------------------+
| inspect_capability /        |
| invoke_proxy_tool           |
+-----------------------------+
```

- Philosophy: lazy-tool is local-first search-before-invoke. It
  wants the fewest moving parts: one Go binary, local SQLite, no
  cloud, no Docker, no vector service unless configured.
- Architecture: `internal/catalog/normalizer.go` normalizes
  tools, prompts, resources, and templates into a common
  `CapabilityRecord` with canonical names, source IDs, tags,
  generated/user summaries, schema-derived arg names, and search
  text.
- `internal/search/service.go` scores exact canonical name,
  original name, substring, lexical match, vector match, user
  summary, favorite status, and invocation history, then returns
  why-matched explanations and breakdowns.
- `internal/mcpserver/tools_search.go` returns not just a ranked
  hit, but also the exact `proxy_tool_name`, required fields,
  next tool to call, and example input. That is a reviewer-
  friendly pattern because the route is inspectable.
- `internal/storage/fts.go` uses SQLite FTS5/BM25, while
  optional vectors can supplement lexical matching.
- Assignment inference: lazy-tool is one of the best
  implementation taste references. Borrow explainable scoring,
  stable capability records, local harness simplicity, and exact
  next-step hints. Avoid the larger proxy mode matrix unless
  needed after evaluation.

**dmcp**

```text
Agent Gateway / MCP servers
    |
    v
+------------------+
| Indexer          |
| discover tools   |
| embed name+desc  |
+------------------+
    |
    v
+------------------+
| Redis JSON +     |
| HNSW vector idx  |
+------------------+
    |
    v
+------------------+
| search_tools     |
+------------------+
    |
    v
+-----------------------------+
| list_changed exposes only   |
| returned tools in session   |
+-----------------------------+
    |
    v
+------------------+
| Lazy backend call|
+------------------+
```

- Philosophy: dmcp is the cleanest "search unlocks tools"
  implementation. The model initially sees only `search_tools`;
  after search, matching tools become callable in the session
  through MCP list-changed notifications.
- Architecture: `server/src/dmcp-server.ts` exposes only
  `search_tools`, keeps per-session exposed tools, lazy-connects
  backend SSE clients, and forwards calls only for tools that
  have been exposed to that session.
- `server/src/redis-vss.ts` indexes each tool as `name:
  description`, stores metadata in Redis JSON, and uses HNSW
  vector KNN with a minimum-score filter.
- `indexer/src/index.ts` discovers tools from Agent Gateway
  config, detects adds/removes/updates, syncs fingerprints,
  embeds in chunks, and writes Redis.
- The README explicitly frames the route as pure vector search
  with a ToolRet style embedding model and no heuristic
  filtering.
- Assignment inference: dmcp is useful for dynamic exposure
  semantics and session-local state, but pure vector retrieval
  alone is too risky for exact IDs, issue numbers, side effects,
  and parameter-schema constraints. Redis, SSE, and gateway
  discovery are overhead for a minimal take-home.

**ElBruno.ModelContextProtocol**

```text
MCP tool definitions
    |
    v
+-----------------------------+
| Format embedding text       |
| name, description, params,  |
| input schema                |
+-----------------------------+
    |
    v
+------------------+
| ToolIndex        |
| local embeddings |
| cosine search    |
+------------------+
    ^
    |
+-----------------------------+
| Optional PromptDistiller    |
| original prompt + action    |
| phrases, merged results     |
+-----------------------------+
```

- Philosophy: this repo is a library-level router for .NET
  users. It focuses on a direct API: index MCP tool definitions,
  route a prompt to top tools, optionally use an LLM to distill
  verbose prompts into compact action phrases.
- Architecture: `ToolIndex.cs` formats embedding text from name,
  description, parameters, and input schema, embeds tools,
  caches query embeddings, computes cosine similarity, and
  returns topK over a minScore threshold.
- `PromptDistiller.cs` asks an LLM for comma-separated 2-5 word
  action phrases, deduplicates them, and falls back to the
  original prompt when distillation fails.
- `ToolRouter.cs` merges the original-prompt baseline search
  with phrase searches. Baseline keeps full score, phrase-only
  results are discounted.
- This is a good example of query decomposition as a light add-
  on rather than a whole agent loop. It is not a proxy and does
  not solve evaluation by itself.
- Assignment inference: borrow schema-aware embedding text and
  optional multi-query merge. Keep distillation optional so the
  baseline remains local and deterministic.

**RAG-MCP-example**

```text
MCP tools/list
    |
    v
+------------------+
| Extract name +   |
| description      |
+------------------+
    |
    v
+------------------+
| Sentence         |
| transformer      |
| embeddings       |
+------------------+
    |
    v
+------------------+
| Cosine top-K     |
+------------------+
    |
    v
+-----------------------------+
| Inject selected tools into  |
| an LLM prompt               |
+-----------------------------+
```

- Philosophy: RAG-MCP-example is the educational skeleton:
  index, rank, inject, generate. It shows the minimal shape of
  the idea without product or research complexity.
- Architecture: `src/main.py` fetches MCP tools through JSON-RPC
  `tools/list`, embeds `name: description` with a sentence
  transformer, ranks by cosine similarity, and builds a compact
  prompt containing only top-K tools.
- `src/utils.py` contains lightweight helper parsing and
  recommendation thresholds, but the parsing is fragile and the
  generation step is effectively stubbed.
- It has no serious baseline, no field-aware scoring, no side
  effect handling, no multi-step/path eval, and no robust schema
  feature extraction.
- Assignment inference: useful as a minimal runnable harness
  outline, not as enough router quality for the grading rubric.

**semantic-router**

```text
Route definitions
utterances
function schemas
    |
    v
+------------------+
| Encoder + index  |
| dense or hybrid  |
+------------------+
    |
    v
+------------------+
| Query scores     |
| route groups     |
+------------------+
    |
    v
+-----------------------------+
| Threshold pass/fail         |
| RouteChoice or no route     |
+-----------------------------+
    |
    v
+-----------------------------+
| Optional dynamic route      |
| extracts function inputs    |
+-----------------------------+
```

- Philosophy: semantic-router is a generic decision layer, not
  an MCP metadata router. It assumes routes with example
  utterances can be defined ahead of time, then optimizes fast
  route choice and thresholding.
- Architecture: `routers/base.py` encodes the query, queries an
  index, groups scores by route, aggregates, and checks
  route/router thresholds before returning a `RouteChoice` or no
  route.
- `routers/hybrid.py` combines dense and sparse embeddings with
  an alpha parameter, including a default BM25 sparse encoder.
- `route.py` supports dynamic routes where a route has function
  schemas and an LLM can extract function input values after the
  route is selected.
- The docs include threshold optimization and evaluation with
  labeled data, which is directly relevant to the assignment's
  quality bar.
- Assignment inference: borrow calibrated thresholds, no-route
  behavior, and eval discipline. Do not predefine a static
  taxonomy of domains, because the assignment expects arbitrary
  unknown tool metadata.

**ToolRoute**

```text
Task text
    |
    v
+-----------------------------+
| LLM classifier              |
| do-vs-explain               |
| named tool extraction       |
+-----------------------------+
    |
    +--------+
             v
        +------------------+
        | Semantic task    |
        | matcher          |
        | pgvector priors  |
        +------------------+
             |
             v
+-----------------------------+
| Skill candidates            |
| relevance or workflow       |
| plus quality scores         |
+-----------------------------+
    |
    v
+-----------------------------+
| Named-tool guard            |
| unresolved beats wrong tool |
+-----------------------------+
    |
    v
+-----------------------------+
| Skill + model recommendation|
| telemetry/reporting loop    |
+-----------------------------+
```

- Philosophy: ToolRoute routes business tasks to the best MCP
  server and LLM model, backed by outcome telemetry. It is a
  marketplace/business router more than a per-call schema subset
  selector.
- Architecture: `src/lib/task-classifier.ts` uses an LLM to
  decide whether an external tool is needed, extract a named
  tool, classify task type/complexity, identify tool categories,
  and detect multi-tool tasks. It also has a keyword fallback
  and cost-aware overrides.
- `src/lib/task-matcher.ts` is a later precision layer: embed
  the query, call a pgvector `match_tasks` RPC, confidence-gate
  low scores, detect ambiguity by top-1/top-2 margin, then rank
  skill priors by relevance score.
- `src/app/api/route/route.ts` shows the product judgments:
  semantic-task matching only adds precision when confident,
  uncertain matches fall back, a top-50 global-score trap was
  explicitly removed for unresolved tool tasks, and named-tool
  gaps return unresolved instead of substituting a different
  brand.
- `src/lib/scoring.ts` encodes the value score as quality,
  reliability, efficiency, cost, and trust. ToolRoute thinks
  about feedback loops and telemetry as first-class routing
  inputs.
- `tests/routing-benchmark-v2.md` is especially useful because
  it keeps known issues visible instead of editing the battery
  to pass. That is strong evaluation culture.
- Assignment inference: borrow named-tool guardrails,
  ambiguity/unresolved behavior, and honest regression
  batteries. Avoid marketplace scoring, Supabase, LLM/model
  routing, credits, missions, and hardcoded category maps for
  the assignment.

**contextweaver**

```text
MCP tool catalog
    |
    v
+------------------+
| SelectableItems  |
| stable ids       |
| tags, namespace  |
+------------------+
    |
    v
+-----------------------------+
| TreeBuilder -> ChoiceGraph  |
+-----------------------------+
    |
    v
+-----------------------------+
| retrieve -> rerank ->       |
| navigate -> pack            |
+-----------------------------+
    |
    v
+-----------------------------+
| ChoiceCards                 |
| no full schemas             |
+-----------------------------+
    |
    v
+-----------------------------+
| constrained selection       |
| validate / repair / reject  |
| hydrate schema on demand    |
+-----------------------------+
```

- Philosophy: contextweaver treats routing as one piece of
  context engineering. The goal is bounded, deterministic
  model-visible context: short choice cards for route selection,
  lazy schema hydration for execution, and a separate firewall
  for large tool outputs.
- Architecture: the README and `docs/tool_router.md` describe a
  bounded-choice router that turns 50-500+ tools into K
  `ChoiceCard`s, never includes full schemas, and hydrates the
  selected schema only after the model commits.
- `src/contextweaver/routing/router.py` implements a beam-search
  router over a `ChoiceGraph` with pluggable retrievers: TF-IDF,
  BM25, fuzzy, and embedding hybrid modes. It also includes
  confidence-gap ambiguity detection, negative routing,
  toolset gating, history-aware score adjustment, pinned items,
  namespace quotas, and deterministic tie-breaking.
- `src/contextweaver/routing/pipeline.py` splits the router into
  retrieve, rerank, navigate, and pack stages. That is a clean
  seam for the assignment: BM25/embedding retrieval and light
  rerank can be swapped without rewriting the harness.
- `src/contextweaver/routing/cards.py` and
  `docs/gateway_spec.md` enforce the key contract: cards include
  IDs, names, descriptions, tags, kind, namespace, safety, and
  has_schema, but not input schemas. Per-card token budgets and
  deterministic order support prompt caching.
- `src/contextweaver/routing/selection.py` constrains selection
  before model generation with an enum of candidate IDs and
  validates after generation with accept/repair/reject
  semantics. Ambiguous repairs are rejected, never guessed.
- `src/contextweaver/eval/routing.py` ships a routing eval
  harness with recall@1/3/5, MRR, average candidates, confidence
  gap, and beam steps. `docs/benchmarks.md` is honest that
  lexical recall drops at large catalog sizes and presents those
  numbers as a baseline floor, not a universal claim.
- Assignment inference: contextweaver is the strongest
  architecture reference for the model-visible interface:
  compact choice cards, no full schemas in the browsing path,
  lazy hydration, deterministic selection validation, and honest
  benchmark framing.

## 2. Assignment Usefulness Table

<table>
<thead>
<tr>
<th>Repo</th>
<th>Useful routing philosophy</th>
<th>LNO</th>
<th>Borrow</th>
<th>Avoid</th>
</tr>
</thead>
<tbody>
<tr>
<td>VOTR</td>
<td>Hybrid retrieval with field-aware rerank and uncertainty-aware K.</td>
<td>Leverage</td>
<td>BM25, dense/sparse fusion, parameter weights, adaptive K.</td>
<td>Shipping a web service before eval quality is proven.</td>
</tr>
<tr>
<td>MCP-Zero</td>
<td>Hierarchical server to tool matching from metadata.</td>
<td>Leverage</td>
<td>Server-first narrowing, dataset framing, namespaced tools.</td>
<td>Mandatory LLM query generation before retrieval.</td>
</tr>
<tr>
<td>graph-tool-call</td>
<td>Some tasks need tool paths, not one nearest tool.</td>
<td>Leverage</td>
<td>Path recall, prerequisite edges, read-before-write cases.</td>
<td>A full workflow graph if subset routing is enough.</td>
</tr>
<tr>
<td>mcpproxy-go</td>
<td>Operational proxy with retrieve-then-call safety surfaces.</td>
<td>Neutral / Overhead</td>
<td>Read/write/destructive metadata and server:tool identity.</td>
<td>UI, scanners, quarantine, tray app, code execution.</td>
</tr>
<tr>
<td>n2-QLN</td>
<td>One-tool facade with staged search and confidence fallback.</td>
<td>Neutral</td>
<td>Trigger/BM25/semantic stages and fallback tests.</td>
<td>Hardcoded categories and auto-execute-first UX.</td>
</tr>
<tr>
<td>lazy-tool</td>
<td>Local-first explainable search-before-invoke.</td>
<td>Leverage</td>
<td>Capability records, why_matched, exact next-step hints.</td>
<td>Full proxy modes before router eval is strong.</td>
</tr>
<tr>
<td>dmcp</td>
<td>Search unlocks tools dynamically in the MCP session.</td>
<td>Neutral</td>
<td>Session-local exposure and min-score gates.</td>
<td>Redis/HNSW/SSE stack and pure vector-only ranking.</td>
</tr>
<tr>
<td>ElBruno.ModelContextProtocol</td>
<td>Schema-aware embedding text plus optional distillation.</td>
<td>Leverage</td>
<td>Params/schema in embeddings and phrase-query merge.</td>
<td>.NET/Azure packaging or mandatory LLM distiller.</td>
</tr>
<tr>
<td>RAG-MCP-example</td>
<td>Minimal index-rank-inject skeleton.</td>
<td>Neutral</td>
<td>Simple runnable harness and top-K injection flow.</td>
<td>Cosine over name and description as enough evidence.</td>
</tr>
<tr>
<td>semantic-router</td>
<td>Calibrated thresholds and no-route behavior.</td>
<td>Leverage</td>
<td>Threshold tuning, labeled eval, dense/sparse scoring.</td>
<td>Static route taxonomies for unknown MCP tools.</td>
</tr>
<tr>
<td>ToolRoute</td>
<td>Business router that refuses wrong named-tool substitutes.</td>
<td>Neutral / Overhead</td>
<td>Named-tool guard, unresolved, ambiguity margins.</td>
<td>Marketplace scoring, model routing, Supabase, credits.</td>
</tr>
<tr>
<td>contextweaver</td>
<td>Choice cards with lazy hydration and selection validation.</td>
<td>Leverage</td>
<td>ChoiceCard contract, selection enum, eval metrics.</td>
<td>Context firewall and full gateway runtime.</td>
</tr>
</tbody>
</table>

The Shreyas-style decision is clear: the L-work is not "build a
better proxy." It is "prove a small router preserves task
success under unknown toolsets."

<table>
<thead>
<tr>
<th>Assignment need</th>
<th>Highest-leverage repo patterns</th>
<th>Practical implementation choice</th>
</tr>
</thead>
<tbody>
<tr>
<td>Unknown MCP servers</td>
<td>VOTR fields, lazy-tool records, contextweaver IDs.</td>
<td>Normalize name, desc, schema, namespace, operation, params.</td>
</tr>
<tr>
<td>Metadata-only routing</td>
<td>VOTR, ElBruno, lazy-tool, RAG-MCP-example.</td>
<td>Build docs from names, params, schema keys, annotations.</td>
</tr>
<tr>
<td>No hardcoded categories</td>
<td>VOTR, lazy-tool, contextweaver are safer references.</td>
<td>Derive capabilities from text/schema signals; no fixed maps.</td>
</tr>
<tr>
<td>Preserve task success</td>
<td>graph-tool-call, ToolRoute, contextweaver validation.</td>
<td>Measure required-tool recall@K and path recall@K.</td>
</tr>
<tr>
<td>Reduce visible tools</td>
<td>contextweaver cards, VOTR adaptive K, dmcp exposure.</td>
<td>Return compact cards and schemas only for the shortlist.</td>
</tr>
<tr>
<td>Evaluation quality</td>
<td>contextweaver harness, ToolRoute battery, MCP-Zero data.</td>
<td>Ship gold set, baselines, ablations, failure buckets.</td>
</tr>
<tr>
<td>Minimal runnable harness</td>
<td>RAG-MCP-example skeleton, lazy-tool local-first taste.</td>
<td>Use mocked GitHub/PostHog/Sentry/Slack/Jira-like tools.</td>
</tr>
<tr>
<td>Exact IDs and numerals</td>
<td>lazy-tool lexical scoring, mcpproxy boosts, VOTR BM25.</td>
<td>Keep BM25/exact match first-class; embeddings are not enough.</td>
</tr>
<tr>
<td>Ambiguity and safety</td>
<td>VOTR abstention, semantic-router thresholds, ToolRoute unresolved.</td>
<td>Use confidence margins; grow K or return unresolved when close.</td>
</tr>
<tr>
<td>Reviewer trust</td>
<td>lazy-tool why_matched, contextweaver explanations, ToolRoute honesty.</td>
<td>Include scores, matched fields, reasons, and failure labels.</td>
</tr>
</tbody>
</table>

Recommended product shape for the assignment:

```text
MCP-style tool metadata fixtures
    |
    v
+-----------------------------+
| Capability normalizer       |
| fields from name/desc/schema|
+-----------------------------+
    |
    v
+-----------------------------+
| Hybrid candidate generator  |
| BM25 + embeddings + exact   |
+-----------------------------+
    |
    v
+-----------------------------+
| Light reranker              |
| operation/object/params/    |
| side-effect compatibility   |
+-----------------------------+
    |
    v
+-----------------------------+
| Adaptive shortlist          |
| K small when clear, larger  |
| or unresolved when close    |
+-----------------------------+
    |
    v
+-----------------------------+
| Choice cards + selected     |
| schemas for harness         |
+-----------------------------+
    |
    v
+-----------------------------+
| Eval report                 |
| recall, precision, tokens,  |
| failure buckets, ablations  |
+-----------------------------+
```

The main implementation bet should be boring and testable:

- Leverage: evaluation design, labeled cases, baselines, hybrid
  retrieval, capability extraction, explainable route output.
- Neutral: clean CLI, small fixtures, readable README,
  deterministic local run.
- Overhead: real MCP auth, live server federation, UI, Redis,
  gateway security, persistent telemetry, model marketplace,
  auto-execution.

This is also the clearest differentiation from the repo
landscape. Most repos say "fewer tools in context." The
assignment answer should say "fewer tools, with measured
preservation of task success, under unknown tool metadata."
