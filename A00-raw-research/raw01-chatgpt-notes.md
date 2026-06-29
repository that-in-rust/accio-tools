## What GitHub research shows

The GitHub landscape is converging on the same problem as the take-home: **too many tools in context → higher token cost, more latency, worse tool choice**. The difference is that most public repos are optimized for **token reduction or gateway UX**, while the take-home is explicitly optimized for **generalization, correctness, and evaluation rigor**: unknown servers, metadata-only routing, no hardcoded categories, a minimal harness, and an evaluation where correctness/baseline/failure-mode coverage are weighted most heavily.   

Using a Shreyas Doshi lens: his LNO framework says not all work deserves equal effort; “Leverage” work can have 10x/100x return, “Neutral” work has modest return, and “Overhead” work is necessary but low-return. For this assignment, the **L-task is not cloning a production MCP proxy**; the L-task is proving, with evaluation, that our router preserves task success while reducing the visible tool surface. ([Coda][1])

---

## Generic tool-router / MCP-router repos on GitHub

| Repo / link                                                  | What it does                                                                                                                                                                                                                                                                                              | Their differentiation, Shreyas POV                                                                                            | Fit for our use case                                                                                                                                                                               |
| ------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **VOTR — Vector Orchestrated Tool Retrieval** ([GitHub][2])  | FastAPI service that retrieves and ranks MCP tools before model invocation, returning a compact candidate set for schema injection. Uses dense similarity, BM25, SPLADE-lite, weighted RRF, and field-aware reranking; claims 96.4% recall on 309 servers / 2,806 tools. ([GitHub][2])                    | **L-task reference.** Closest to what we need technically: hybrid retrieval + reranking + benchmark story.                    | Very high. Borrow hybrid retrieval and field-aware scoring. But our differentiation should be richer eval on GitHub/PostHog/Sentry-style ambiguity, not just retrieval recall.                     |
| **MCP-Zero** ([GitHub][3])                                   | Research repo for “Active Tool Discovery.” Includes matcher/reformatter/prompt code and a dataset of 308 servers / 2,797 tools with tool descriptions, parameters, and embeddings. ([GitHub][3])                                                                                                          | **L-task reference for research framing.** Differentiation is active discovery and large tool-pool experimentation.           | High as inspiration. Use its dataset/eval thinking, but do not copy the whole “active discovery” system; take-home wants a narrow runnable router/harness.                                         |
| **graph-tool-call** ([GitHub][4])                            | Builds a graph over tools and retrieves the right **chain**, not just the nearest single tool. Reports 248 Kubernetes tools: 12% baseline vs. 82% with graph retrieval, and 1,068 GitHub tools with 78% Recall@5. Also supports OpenAI/Anthropic middleware and LangChain gateway patterns. ([GitHub][4]) | **L-task idea.** Strongest differentiation is “tool path retrieval,” which is more realistic than top-K independent tools.    | Very high for multi-step examples like “errors since PR #1242.” We should borrow the concept of evaluating complete viable paths, even if our implementation is simpler.                           |
| **MCPProxy / mcpproxy-go** ([GitHub][5])                     | Productized MCP proxy that federates many MCP servers, exposes one `retrieve_tools` style function, has web UI, security quarantine, scanners, audit logs, and cross-platform binary distribution. ([GitHub][5])                                                                                          | **Product differentiation.** Their wedge is operational UX + security + distribution, not just routing quality.               | Medium. Great “production future” reference. But for the take-home, UI, live MCP plumbing, auth, and scanner work are Overhead.                                                                    |
| **n2-QLN** ([GitHub][6])                                     | Query Layer Network: indexes tools into SQLite and exposes one MCP tool, `n2_qln_call`; search uses trigger match, BM25, optional semantic search, usage/success feedback, confidence gate, and fallback chain. ([GitHub][6]) ([GitHub][6]) ([GitHub][6])                                                 | **Sharp product wedge.** “1,000 tools through 1 tool” is memorable. Confidence gating and fallback are good product judgment. | Medium-high. Borrow confidence gates and fallback. But our take-home should still return a selected subset of tools before the model call, not only hide everything behind one universal executor. |
| **lazy-tool** ([GitHub][7])                                  | Local-first Go binary that reads existing IDE MCP configs, indexes tools in SQLite, and supports “search before invoke.” Retrieval combines exact/name/FTS5 BM25/substring and optional vector search, with `why_matched` explanations. ([GitHub][7]) ([GitHub][7])                                       | **Execution simplicity.** Differentiation is local-first, no cloud, no Docker, explainable scoring.                           | High for implementation taste. We should copy the `why_matched` idea because it helps the design note and failure analysis.                                                                        |
| **DMCP — Dynamic MCP** ([GitHub][8])                         | Query-driven semantic tool discovery for MCP. Uses ToolRet-trained E5-large-v2 embeddings, Redis HNSW vector search, and returns top-K tools that become available via `notifications/tools/list_changed`. ([GitHub][8]) ([GitHub][8])                                                                    | **Model differentiation.** It bets on a tool-specialized retrieval model rather than generic embeddings.                      | Medium. Good to mention specialized embeddings as future work. For our assignment, pure vector retrieval is risky for exact IDs, schema constraints, and read/write distinctions.                  |
| **ElBruno.ModelContextProtocol MCPToolRouter** ([GitHub][9]) | .NET library that indexes MCP tool definitions and returns relevant tools via local embeddings/vector similarity. Also has an LLM-assisted mode that distills verbose prompts into action phrases before search. ([GitHub][9])                                                                            | **Platform differentiation.** Strongest for .NET/Azure/Microsoft Agent Framework users.                                       | Medium. Useful design reference, especially local embeddings and prompt distillation. Less useful if we build in Python.                                                                           |
| **RAG-MCP-example** ([GitHub][10])                           | Lightweight prototype: fetch MCP tools via JSON-RPC `tools/list`, embed descriptions with sentence-transformers, rank top-K, inject only relevant tools, then generate. ([GitHub][10])                                                                                                                    | **Educational skeleton.** Differentiation is simplicity, not depth.                                                           | Medium-low. Good starting skeleton, but too thin for the rubric unless we add baselines, failure-mode eval, reranking, and path correctness.                                                       |
| **Semantic Router** ([GitHub][11])                           | Mature semantic decision/routing layer. It defines routes with example utterances, embeds them, and routes queries by semantic meaning; also has dynamic route/function-call docs. ([GitHub][11])                                                                                                         | **Mature primitive.** Differentiation is fast semantic intent routing, not MCP tool metadata routing.                         | Medium. Useful for thresholds and route-layer patterns, but not enough alone because the take-home forbids curated routing tables and expects arbitrary tool metadata.                             |
| **ToolRoute** ([GitHub][12])                                 | Routes a task to the best MCP server and LLM model. Uses an LLM classifier, ranks candidates by quality/reliability/efficiency/cost/trust, and updates scores from outcomes. ([GitHub][12])                                                                                                               | **Business-level router.** Differentiation is model + server recommendation, not individual tool-subset selection.            | Medium-low. Interesting for product roadmap and telemetry, but not the core take-home problem.                                                                                                     |
| **contextweaver** ([GitHub][13])                             | Budget-aware context compiler and context firewall. Can narrow a 100-tool catalog to 5 ChoiceCards, store large tool outputs out of band, and build deterministic phase-specific context packs. ([GitHub][13]) ([GitHub][13])                                                                             | **Context-management wedge.** Differentiation is deterministic context budgeting, not only routing.                           | Medium. Good future-work reference. For this assignment, the context firewall is mostly out of scope, but “ChoiceCards” are a useful design pattern for compact tool summaries.                    |

---

## What this means for our differentiation

Most repos are saying:

> “We reduce context/token bloat by retrieving fewer tools.”

That is not differentiated enough for this take-home. Our version should say:

> “We preserve task success under unknown tool servers by selecting a small subset, and our evaluation proves where it works and breaks.”

That difference matters because the PDF explicitly says the connected servers are unknown in advance, the router only gets name/description/schema metadata, hardcoded categories are disallowed, and evaluation quality is the top grading criterion.  

| Existing repo pattern                  | Our differentiated angle                    | What we should build                                                                                                                                                                    |
| -------------------------------------- | ------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Top-K semantic retrieval**           | **Correctness-first routing**               | Primary metric should be **path recall@K**: did the selected tools contain at least one viable path to solve the task?                                                                  |
| **Token reduction claims**             | **Failure-mode coverage**                   | Eval set should cover overlapping nouns, sparse descriptions, misleading descriptions, exact IDs, multi-server tasks, and read-vs-write distinctions.                                   |
| **One universal router/executor tool** | **Subset injection before model reasoning** | The router should return selected tool schemas to the harness before the model call, because that is what the assignment asks for.                                                      |
| **Pure vector search**                 | **Hybrid retrieval + capability scoring**   | Use BM25/lexical for exact terms like `#1242`, embeddings for semantic matches, and capability features for `create/search/update`, object type, side effects, and required parameters. |
| **Production gateway features**        | **Small runnable evidence loop**            | No UI, no live MCP auth, no deployment. Mock tools are fine. The repo should run with one command and produce an eval table.                                                            |
| **General “accuracy” numbers**         | **Asymmetric error analysis**               | False negatives are worse than false positives. Missing the required tool can kill the task; including an extra tool mostly costs tokens/confusion.                                     |

---

## Shreyas Doshi POV: what to spend effort on

| Work item                                                |                        LNO classification | Why                                                                                                                        |
| -------------------------------------------------------- | ----------------------------------------: | -------------------------------------------------------------------------------------------------------------------------- |
| Evaluation design, labeled query set, metrics, baselines |                                     **L** | This is the highest-leverage part of the assignment. It is also rubric item #1.                                            |
| Hybrid router: BM25 + embeddings + light reranker        |                                     **L** | Strong enough to show judgment; simple enough to finish.                                                                   |
| Capability extraction from schema/name/description       |                                     **L** | This is where we differentiate from simple semantic search. It handles “GitHub issue can be created; Sentry issue cannot.” |
| Runnable harness with mocked tools                       |                                     **N** | Necessary, but keep it minimal. The PDF explicitly says a simple loop is sufficient.                                       |
| Pretty CLI output / README polish                        |                                     **N** | Important for reviewer experience, but do not overbuild.                                                                   |
| Live MCP servers, OAuth, UI, memory, streaming, retries  |                                     **O** | Explicitly out of scope or low-return for this take-home.                                                                  |
| Production security gateway / scanner / quarantine       | **O for take-home, L for future roadmap** | Good to mention as future work, but not to implement now.                                                                  |

---

## Recommended tech stack for our use case

| Layer               | Stack                                                                       | Why                                                                                                                          |
| ------------------- | --------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| Language            | **Python 3.11+**                                                            | Fastest for eval, retrieval experiments, JSON fixtures, and simple mock harness.                                             |
| Package/CLI         | `uv`, `typer`, `rich`                                                       | One-command execution and readable eval output.                                                                              |
| Schemas             | `pydantic`, `jsonschema`                                                    | Normalize arbitrary MCP-style tool metadata: name, description, parameter schema.                                            |
| Lexical retrieval   | `rank-bm25` or `scikit-learn` TF-IDF/BM25-style scoring                     | Handles exact strings, repo names, branch names, PR numbers, event names.                                                    |
| Semantic retrieval  | `sentence-transformers` locally, or optional OpenAI/Cohere embeddings       | Handles “broken build” → CI/workflow, “signups down” → analytics/funnel/trends.                                              |
| Reranking           | Optional LLM structured-output reranker                                     | Use only on top 25–40 candidates; make it optional so the baseline runs locally.                                             |
| Capability features | Custom parser over name/description/schema                                  | Extract `operation=create/search/update/list`, `object=issue/event/build/funnel`, `side_effect=read/write`, required params. |
| Evaluation          | `pytest`, `pandas`, JSONL fixtures                                          | Produce reproducible metrics: path recall@K, required recall@K, precision@K, token reduction, write-tool exposure.           |
| Mock harness        | Simple loop with fake GitHub/PostHog/Sentry/Linear/Jira/Slack/Datadog tools | Shows end-to-end behavior without live MCP, auth, deployment, or UI.                                                         |

---

## The positioning I would use in the repo/design note

**One-line differentiation:**

> Unlike generic semantic routers that optimize mainly for token reduction, this router optimizes for task-success recall under unknown MCP toolsets, with an evaluation focused on ambiguous, overlapping, sparse, and multi-step tool-routing failures.

**What to borrow from GitHub projects:**

VOTR gives us hybrid retrieval and field-aware reranking. MCP-Zero gives us research/eval framing. graph-tool-call gives us the idea that the unit of correctness is often a **tool path**, not one tool. lazy-tool gives us explainable `why_matched` diagnostics. n2-QLN gives us confidence gating and fallback as product polish.

**What not to copy:**

Do not build a full MCP proxy, UI, live server connector, security scanner, model router, or production context firewall. Those are impressive, but for this exercise they distract from the L-task: **prove the router works with a credible evaluation.**

[1]: https://coda.io/%40shreyas/lno-framework "LNO Framework"
[2]: https://github.com/iamAmiK/VOTR "GitHub - iamAmiK/VOTR: [ANON for Submission] · GitHub"
[3]: https://github.com/xfey/MCP-Zero "GitHub - xfey/MCP-Zero: MCP-Zero: Active Tool Discovery for Autonomous LLM Agents · GitHub"
[4]: https://github.com/SonAIengine/graph-tool-call "GitHub - SonAIengine/graph-tool-call: Graph-based tool retrieval for LLM agents — 248 tools → 82% accuracy, 79% fewer tokens. Zero dependencies. OpenAPI / MCP / LangChain. · GitHub"
[5]: https://github.com/smart-mcp-proxy/mcpproxy-go "GitHub - smart-mcp-proxy/mcpproxy-go: Supercharge AI Agents, Safely · GitHub"
[6]: https://github.com/choihyunsus/n2-QLN "GitHub - choihyunsus/n2-QLN: n2-QLN: Intelligent Tool Router & Semantic Search Layer for MCP. Connect 1,000+ tools through 1 interface. Prevent AI model confusion and maximize context window efficiency · GitHub"
[7]: https://github.com/mcp-shark/lazy-tool "GitHub - mcp-shark/lazy-tool: local-first MCP discovery runtime for agents — search before invoke, reduce prompt bloat, and route to local MCP tools · GitHub"
[8]: https://github.com/Grep-Juub/dmcp "GitHub - Grep-Juub/dmcp: Dynamic MCP - Semantic tool discovery for Model Context Protocol · GitHub"
[9]: https://github.com/elbruno/ElBruno.ModelContextProtocol "GitHub - elbruno/ElBruno.ModelContextProtocol: Semantic routing for MCP tools - .NET library that indexes MCP tool definitions and returns the most relevant tools via vector search · GitHub"
[10]: https://github.com/adeweaver/RAG-MCP-example "GitHub - adeweaver/RAG-MCP-example · GitHub"
[11]: https://github.com/aurelio-labs/semantic-router "GitHub - aurelio-labs/semantic-router: Superfast AI decision making and intelligent processing of multi-modal data. · GitHub"
[12]: https://github.com/grossiweb/ToolRoute "GitHub - grossiweb/ToolRoute: Intelligent routing layer for AI agents: recommends the best MCP server and LLM for any task, scored on 132+ real benchmark executions. · GitHub"
[13]: https://github.com/dgenio/contextweaver "GitHub - dgenio/contextweaver: Budget-aware context compilation and context firewall for tool-heavy AI agents. · GitHub"


# Other Notes

``` text

you've a query and you've tool descriptions and probably features. 

Next you'll calculate cosine similarity between your query and tool descriptions and re-rank them based on increasing distance.

You'll then return top-k tools in that ordering.

If you remove the MCP/server/tool language we are left with: Given a natural language query and a large graph of capabilities, identify the smallest connected subgraph that contains everything needed to answer the query.

I’m thinking of each tool as a node in a graph. Each node can be connected based on semantic similarity of descriptions, parameter overlap, historical co-usage etc.

From the example that’s been given: “Checkout started failing after release 1242” release gets mapped to commit which gets mapped to GitHub as a tool.

So this becomes a graph problem: Find the minimum neighborhood around the most relevant nodes.


* **Baseline approach (semantic retrieval):**

  * Convert the user's natural language query into an embedding.
  * Embed every tool description (and optionally tool metadata/features).
  * Compute **cosine similarity** between the query embedding and each tool embedding.
  * Rank tools by decreasing similarity (or increasing cosine distance).
  * Return the **top-k** most relevant tools. This is the standard semantic tool routing approach. ([vLLM Semantic Router][1])

* **Abstracting away MCP/tools:**

  * Ignore terms like *MCP*, *server*, and *tool*.
  * The problem becomes:

    * **Given a natural language query and a large graph of capabilities, identify the smallest connected subgraph that contains everything needed to answer the query.**

* **Graph interpretation:**

  * Model each capability/tool as a **node**.
  * Add edges based on relationships such as:

    * Semantic similarity between descriptions.
    * Parameter or schema overlap.
    * Historical co-usage.
    * Shared data sources or dependencies.
    * Domain relationships.

* **Example reasoning:**

  * Query: *"Checkout started failing after release 1242."*
  * Semantic retrieval identifies **Release** as the primary concept.
  * The graph expands through connected capabilities:

    * Release → Commit
    * Commit → GitHub
    * Potentially GitHub → CI/CD → Logs → Monitoring
  * The system retrieves only the local neighborhood required to investigate the issue.

* **Key insight:**

  * Instead of selecting **independent top-k tools**, treat routing as a **graph search problem**.
  * The objective shifts from:

    * *"Which tools are individually most similar?"*
  * To:

    * *"Which minimal connected neighborhood of capabilities collectively solves the user's task?"*

* **Implication:**

  * Semantic similarity becomes the **entry point** into the graph.
  * Graph traversal then discovers additional capabilities needed through explicit relationships, yielding a smaller, more coherent execution plan than simple top-k retrieval. This aligns with broader work on semantic routing and graph-based retrieval.

[1]: https://vllm-semantic-router.com/blog/semantic-tool-selection/?utm_source=chatgpt.com "Building Smarter AI Agents with Context-Aware Routing"

```