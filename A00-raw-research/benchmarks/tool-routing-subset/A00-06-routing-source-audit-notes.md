# Reference Repo Audit

## All 21 Repos Inspected

| Repo | Verdict | Relevant local evidence |
| --- | --- | --- |
| VOTR | Primary source | Router-only suites in `benchmarks/functional_correctness/`; no-route controls in `benchmarks/negative_controls/`; catalog subsets in `data/catalog_subsets/`. |
| contextweaver | Primary source | `benchmarks/routing_gold.json` plus generated catalog families in `src/contextweaver/routing/catalog.py`; benchmark docs define recall/MRR/latency. |
| graph-tool-call | Primary source | `benchmarks/ground_truth/github.json`, `benchmarks/ground_truth/multi_mcp.json`, OpenAPI spec, and MCP tool metadata under `benchmarks/mcp_tools/`. |
| mcpproxy-go | Primary source | Frozen MCP corpus and graded retrieval labels in `specs/065-evaluation-foundation/datasets/`. |
| LiveMCPBench | Primary source | 95 annotated MCP tasks in `annotated_data/all_annotations.json`; 525 tool records in `tools/LiveMCPTool/tools.json`. |
| mcp-bench | Primary source | MCP runner tasks in `tasks/mcpbench_tasks_*_runner_format.json`; bundled MCP server implementations under `mcp_servers/`. |
| lazy-tool | Secondary source | Benchmark methodology and sample no-tool/output rows under `benchmark/`, but local golden rows are harness-output samples rather than query gold. |
| benchmarking-tool-retrieval | Secondary source | Retrieval methodology/code under `toolret/`, but local clone does not include a directly reusable query/tool gold corpus. |
| ToolBench | Secondary source | Broad API/tool-use benchmark data; less MCP/router-only than the chosen sources for a five-hour subset. |
| StableToolBench | Secondary source | Stable ToolBench query/API data; useful later for broad API generalization, not primary for this MCP router slice. |
| gorilla | Secondary source | BFCL/function-calling materials; useful for function-call baselines, not pre-model MCP tool routing. |
| ToolSandbox | Secondary source | Agent/tool-environment benchmark; better for end-to-end execution behavior than routing-only evaluation. |
| tau2-bench | Secondary source | Large task/domain benchmark; not a compact pre-tool-router benchmark. |
| appworld | Secondary source | App/task simulator benchmark; useful for downstream agents, not direct route gold. |
| ToolRoute | Implementation reference | Routing implementation/docs/tests, not a reusable labeled benchmark corpus. |
| semantic-router | Implementation reference | Routing library reference, not assignment-specific benchmark data. |
| n2-QLN | Catalog reference | Small provider test tools in `providers/test-tools.json`; useful as distractors, not selected as gold source. |
| dmcp | Implementation reference | MCP/server implementation material; no direct local routing-gold subset found. |
| ElBruno.ModelContextProtocol | Implementation/demo reference | MCP routing sample code under `src/samples/McpToolRouting/`; no benchmark gold. |
| RAG-MCP-example | Demo reference | Small RAG/MCP example; no benchmark gold. |
| MCP-Zero | Dataset pointer/reference | Local file has task-to-server examples and README claims a larger corpus, but the full dataset is not present in the clone. |

## Primary Sources Used

| Repo | Used for | Evidence |
| --- | --- | --- |
| VOTR | Router-only ambiguity, robustness, no-route, and multi-tool cases | `benchmarks/functional_correctness/*.json`, `benchmarks/negative_controls/priority_cases.json` |
| contextweaver | Hand-crafted routing gold and generated catalog families | `benchmarks/routing_gold.json`, `src/contextweaver/routing/catalog.py` |
| graph-tool-call | GitHub OpenAPI and mixed MCP ground-truth tool selection | `benchmarks/ground_truth/*.json`, `benchmarks/specs/github_subset.json`, `benchmarks/mcp_tools/*.json` |
| mcpproxy-go | Frozen MCP corpus and graded retrieval labels | `specs/065-evaluation-foundation/datasets/*.json` |
| LiveMCPBench | Real MCP-style multi-step annotated tasks | `annotated_data/all_annotations.json`, `tools/LiveMCPTool/tools.json` |
| mcp-bench | Real MCP task-runner descriptions with explicit server:tool references | `tasks/mcpbench_tasks_*_runner_format.json` |

## Secondary Sources Inspected

| Repo | Decision |
| --- | --- |
| benchmarking-tool-retrieval | Relevant methodology for tool retrieval, but local clone does not include a directly reusable query/tool gold set. |
| ToolBench / StableToolBench / gorilla BFCL | Useful broad function-calling/API-selection benchmarks, but less assignment-shaped than MCP/router-only sources. |
| ToolSandbox / tau2-bench / appworld | Agent-task benchmarks; useful later for end-to-end behavior, not for this five-hour pre-model router subset. |
| lazy-tool | Useful MCP efficiency methodology and no-tool examples; not used as primary query source because its local golden rows are harness-output samples. |
| ToolRoute / semantic-router / n2-QLN / dmcp / mcpproxy-go implementation code / ElBruno / RAG-MCP-example / MCP-Zero | Implementation or catalog references; inspected for context and distractors, not primary gold labels. |

## Why This Subset Is Enough for the Take-Home

The assignment asks whether a router can select a small relevant tool subset from
unknown connected servers using only tool metadata. The selected sources cover:

- overlapping names and capabilities;
- sparse or noisy user phrasing;
- multi-tool single-turn routing;
- explicit no-route/abstention cases;
- frozen-corpus graded retrieval labels;
- realistic MCP task descriptions.
