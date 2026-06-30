# Tool Reader Architecture Options

## Governing Thought

For this assignment, the strongest architecture is a cascaded tool reader: use CPU-local retrieval to shrink the tool universe, then use a cheap semantic or LLM judge only on the narrowed set before exposing compact tool cards to the final agent. This matches the assignment better than a full MCP proxy because the proof burden is not "can we execute every tool"; it is "can we preserve the right tool while reducing context and confusion."

## SCQA

**Situation:** A tool router receives a user query, maybe recent conversation context, and a catalog of tool metadata: names, descriptions, schemas, annotations, and possibly server-level summaries.

**Complication:** Sending every tool to the final LLM is expensive and degrades selection quality, but asking an LLM to inspect hundreds or thousands of tools directly recreates the same context problem.

**Question:** Which architecture gives us the best proof of routing quality under time pressure?

**Answer:** Build the router as a CPU-first cascade with optional cheap semantic/LLM judgment, plus a Confido-style evidence console that shows why each tool survived or was dropped.

## Recommended Direction

Use **Option E + Option I**:

```text
query + recent conversation context
  -> CPU-local candidate generation
  -> optional cheap semantic / LLM review
  -> compact top-K ChoiceCards / tool cards
  -> final agent sees only the shortlist
  -> result, error, or uncertainty feeds back into routing
```

This gives the cleanest product story: CPU does the scalable search work, the cheap model reviews a small judgment surface, and the expensive final agent context is protected.

## Architecture Options Compared

PMF score is a Shreyas Doshi-style 1-100 product judgment score for this assignment: higher means better leverage, clearer proof, stronger reviewer signal, and better fit to a 5-hour execution window.

| Option | Architecture | CPU role | Low-cost API / LLM role | Bidirectional? | Tools in `ignore-references` using this method | How this method works | Assignment fit | PMF / 100 |
| --- | --- | --- | --- | --- | --- | --- | --- | ---: |
| A | Full tool list in final LLM context | None, except serializing schemas | Final LLM reads all tools and decides | No | Baseline / anti-pattern in [contextweaver](../ignore-references/git-ref-repo/contextweaver/README.md), [lazy-tool](../ignore-references/git-ref-repo/lazy-tool/README.md), [graph-tool-call](../ignore-references/git-ref-repo/graph-tool-call/README.md), [DMCP](../ignore-references/git-ref-repo/dmcp/README.md) | Every tool schema is injected into the prompt. This creates high token load and makes the LLM perform search, ranking, and reasoning in one overloaded context. | Useful only as a baseline that shows why routing exists. | 25 |
| B | Pure lexical router | Main engine: exact match, tokenization, FTS/BM25, schema-name overlap | None | Weak | [lazy-tool](../ignore-references/git-ref-repo/lazy-tool/README.md), [n2-QLN](../ignore-references/git-ref-repo/n2-QLN/README.md), [VOTR](../ignore-references/git-ref-repo/VOTR/README.md) ablations | Query and tool metadata are tokenized. The router scores exact name hits, BM25/FTS matches, substring matches, boosted keywords, and schema terms. | Fast and explainable, but misses semantic paraphrases and vague user intent. | 68 |
| C | Pure embedding router | Indexing and nearest-neighbor search | Embedding model/API converts query and tool text into vectors | Weak | [RAG-MCP-example](../ignore-references/git-ref-repo/RAG-MCP-example/README.md), [DMCP](../ignore-references/git-ref-repo/dmcp/README.md), [ElBruno Mode 1](../ignore-references/git-ref-repo/ElBruno.ModelContextProtocol/README.md), [MCP-Zero](../ignore-references/git-ref-repo/MCP-Zero/README.md) dataset embeddings, [semantic-router](../ignore-references/git-ref-repo/semantic-router/README.md) | Tool descriptions and queries are embedded into vector space. The router returns nearest tools by cosine similarity or vector index score. | Good demo, but risky for exact IDs, read/write distinctions, and sparse descriptions. | 72 |
| D | Hybrid CPU router | Main engine: BM25, schema parsing, capability extraction, side-effect filters, dedupe | Optional embeddings or reranker | Medium | [VOTR](../ignore-references/git-ref-repo/VOTR/README.md), [lazy-tool](../ignore-references/git-ref-repo/lazy-tool/README.md), [n2-QLN](../ignore-references/git-ref-repo/n2-QLN/README.md), [graph-tool-call](../ignore-references/git-ref-repo/graph-tool-call/README.md), [contextweaver](../ignore-references/git-ref-repo/contextweaver/README.md) | Multiple retrieval signals are fused: lexical match, vector similarity, schema overlap, tool annotations, usage history, and deterministic filters. Results are merged, deduped, scored, and returned with explanations. | Strong core for the take-home. It shows real routing judgment without overusing LLM context. | 86 |
| E | CPU shortlist plus cheap LLM judge | Generates top 25-50 candidates and extracts evidence fields | Cheap model distills intent, reranks, resolves ambiguity, assigns confidence, or abstains | Yes | [ElBruno Mode 2](../ignore-references/git-ref-repo/ElBruno.ModelContextProtocol/README.md), [ToolRoute](../ignore-references/git-ref-repo/ToolRoute/README.md), [confido-exploration-01](../ignore-references/git-ref-repo/confido-exploration-01/ReadMe.md) as evidence-review pattern | CPU keeps the candidate set small. A cheap model sees only the query, compact tool cards, and evidence, then returns structured decisions: include, exclude, abstain, ask for more info, or escalate. | Best assignment shape: scalable retrieval plus visible judgment. | **94** |
| F | Graph/path router | Builds and searches a graph of tools, prerequisites, complementary tools, and workflow order | Optional semantic expansion or plan validation | Yes | [graph-tool-call](../ignore-references/git-ref-repo/graph-tool-call/README.md), [ToolBench](../ignore-references/git-ref-repo/ToolBench/README.md) traces as evaluation inspiration, [ToolSandbox](../ignore-references/git-ref-repo/ToolSandbox/README.md) milestone DAG as evaluation inspiration | Tools become graph nodes. Edges encode relationships such as precedes, requires, complementary, destructive, or follow-up. Retrieval returns a viable tool path, not just the nearest single tool. | Highly relevant for multi-step tasks, but heavier to build in 5 hours. | 82 |
| G | Active tool reader | Detects uncertainty and missing information; fetches more metadata/docs when needed | Summarizes extra docs, expands queries, or asks clarification | Yes | [MCP-Zero](../ignore-references/git-ref-repo/MCP-Zero/README.md), [AppWorld](../ignore-references/git-ref-repo/appworld/README.md), [LiveMCPBench](../ignore-references/git-ref-repo/LiveMCPBench/README.md) retrieval setup | The router starts with metadata, then actively retrieves server summaries, API docs, examples, or extra schema information only when the first-pass confidence is low. | Conceptually excellent, but risky unless mocked. | 78 |
| H | Universal proxy tool | Internal retrieval and execution hidden behind one exposed tool | May classify, search, execute, and recover internally | Yes | [n2-QLN](../ignore-references/git-ref-repo/n2-QLN/README.md), [lazy-tool search mode](../ignore-references/git-ref-repo/lazy-tool/README.md), [DMCP](../ignore-references/git-ref-repo/dmcp/README.md), [mcpproxy-go](../ignore-references/git-ref-repo/mcpproxy-go/README.md) | The final agent sees one or a few meta-tools such as `search_tools` or `n2_qln_call`. The proxy searches its catalog, optionally inspects schemas, then invokes the real upstream tool. | Productizable, but less aligned if the assignment expects selected tool schemas before model invocation. | 63 |
| I | Confido-style evidence console | Renders deterministic scores, matched fields, traces, and before/after comparisons | Renders model judgment and reviewer-facing explanations | Yes | [confido-exploration-01](../ignore-references/git-ref-repo/confido-exploration-01/ReadMe.md), [contextweaver](../ignore-references/git-ref-repo/contextweaver/README.md), [lazy-tool web/TUI](../ignore-references/git-ref-repo/lazy-tool/README.md), [mcpproxy-go web UI](../ignore-references/git-ref-repo/mcpproxy-go/README.md) | The UI is not the router. It exposes the routing trace: query, candidate scores, matched metadata, selected cards, token savings, confidence, and failure bucket. | Excellent presentation layer because it makes routing quality legible. | 88 |
| J | Production MCP gateway | Server federation, auth, connection reuse, health checks, security scanning | Optional routing, summarization, and telemetry | Yes | [mcpproxy-go](../ignore-references/git-ref-repo/mcpproxy-go/README.md), [lazy-tool](../ignore-references/git-ref-repo/lazy-tool/README.md), [DMCP](../ignore-references/git-ref-repo/dmcp/README.md), [n2-QLN](../ignore-references/git-ref-repo/n2-QLN/README.md) | A gateway sits between agents and many MCP servers. It handles upstream discovery, transport, security, logs, caching, proxying, and sometimes search. | Strong future roadmap, but too much for the take-home slice. | 45 |
| K | Learning router | Updates scoring from usage, success, failures, and explicit reports | Reviews failures and suggests scoring/prompt changes | Yes | [n2-QLN](../ignore-references/git-ref-repo/n2-QLN/README.md), [ToolRoute](../ignore-references/git-ref-repo/ToolRoute/README.md), [lazy-tool](../ignore-references/git-ref-repo/lazy-tool/README.md) | The router records which candidates were selected and whether execution succeeded. Future scores incorporate usage count, success rate, source trust, or explicit reports. | Great second-order story, but should be backlog unless a tiny feedback field already exists. | 74 |

## What The Local References Prove

The reference repos cluster into four method families.

| Method family | Repos | Useful lesson for us |
| --- | --- | --- |
| Hybrid retrieval routers | [VOTR](../ignore-references/git-ref-repo/VOTR/README.md), [lazy-tool](../ignore-references/git-ref-repo/lazy-tool/README.md), [n2-QLN](../ignore-references/git-ref-repo/n2-QLN/README.md), [graph-tool-call](../ignore-references/git-ref-repo/graph-tool-call/README.md) | Do not rely on one signal. Combine exact terms, BM25, embeddings, schemas, annotations, and confidence gates. |
| Pure semantic retrieval | [RAG-MCP-example](../ignore-references/git-ref-repo/RAG-MCP-example/README.md), [DMCP](../ignore-references/git-ref-repo/dmcp/README.md), [ElBruno Mode 1](../ignore-references/git-ref-repo/ElBruno.ModelContextProtocol/README.md), [semantic-router](../ignore-references/git-ref-repo/semantic-router/README.md) | Embeddings are useful for intent, but insufficient alone for exact tool constraints and side-effect safety. |
| Proxy and gateway products | [mcpproxy-go](../ignore-references/git-ref-repo/mcpproxy-go/README.md), [lazy-tool](../ignore-references/git-ref-repo/lazy-tool/README.md), [n2-QLN](../ignore-references/git-ref-repo/n2-QLN/README.md), [DMCP](../ignore-references/git-ref-repo/dmcp/README.md) | One exposed meta-tool is a strong product shape, but it can obscure the assignment proof unless we still report shortlist recall and reasons. |
| Evaluation environments | [ToolRet](../ignore-references/git-ref-repo/benchmarking-tool-retrieval/README.md), [LiveMCPBench](../ignore-references/git-ref-repo/LiveMCPBench/README.md), [ToolBench](../ignore-references/git-ref-repo/ToolBench/README.md), [StableToolBench](../ignore-references/git-ref-repo/StableToolBench/README.md), [ToolSandbox](../ignore-references/git-ref-repo/ToolSandbox/README.md), [AppWorld](../ignore-references/git-ref-repo/appworld/README.md) | Evaluate retrieval separately from execution. Then add task-level or trajectory-level checks only after shortlist recall is credible. |

## Tool Reader Pipeline

The version we should present is:

```text
1. Normalize tool metadata
   - name, description, schema, required params, annotations, server summary

2. Extract query intent
   - operation, object, entities, constraints, read/write risk, conversation references

3. CPU candidate generation
   - exact name/alias match
   - BM25 / FTS lexical search
   - schema-field overlap
   - capability and side-effect filters
   - optional embedding similarity

4. Cheap judge on top candidates
   - include / exclude / abstain
   - reason codes
   - confidence
   - ask for more metadata if needed

5. Compile compact tool cards
   - selected top-K tools
   - minimal schema
   - why matched
   - risk flag

6. Feedback loop
   - if confidence low, broaden search
   - if execution fails, try fallback candidate
   - if required tool missing in eval, record failure bucket
```

## Presentation Claim

The strongest phrasing is:

> We do not use the LLM to search the whole tool universe. We use CPU-local retrieval to shrink the universe, then use a cheap model only to review the narrowed judgment surface before exposing compact tool cards to the final agent.

## Immediate Build Choice

For a 5-hour assignment slice:

1. Build **Option D** as the deterministic local baseline.
2. Add **Option E** as an optional rerank/judge mode.
3. Present with **Option I** so the reviewer sees the evidence.
4. Keep **Option F/K/J** as roadmap, not MVP.

That keeps the work aligned with the rubric: correctness, evaluation, generalization, and failure-mode coverage before production gateway polish.
