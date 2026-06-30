#!/usr/bin/env python3
"""Build a reference-derived subset for the tool-routing take-home exercise.

The goal is deliberately modest: create a defensible 50-query subset from the
local benchmark/reference shelf without pretending to run a full benchmark.
Every generated tool/query carries source provenance back to git-ref-repo.
"""

from __future__ import annotations

import ast
import json
import re
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REF_ROOT = ROOT / "git-ref-repo"
OUT_DIR = ROOT / "benchmarks" / "tool-routing-subset"


SOURCE_FILES = {
    "votr_ambiguity": REF_ROOT
    / "VOTR/benchmarks/functional_correctness/ambiguity_collision.priority.json",
    "votr_robustness": REF_ROOT
    / "VOTR/benchmarks/functional_correctness/robustness_safety.priority.json",
    "votr_negative": REF_ROOT / "VOTR/benchmarks/negative_controls/priority_cases.json",
    "votr_multi": REF_ROOT
    / "VOTR/benchmarks/functional_correctness/multi_tool.single_turn.json",
    "votr_small_github": REF_ROOT / "VOTR/data/catalog_subsets/small_github.embedding.json",
    "votr_small_3": REF_ROOT / "VOTR/data/catalog_subsets/small_3_servers.embedding.json",
    "votr_small_telegram": REF_ROOT
    / "VOTR/data/catalog_subsets/small_telegram.embedding.json",
    "contextweaver_gold": REF_ROOT / "contextweaver/benchmarks/routing_gold.json",
    "contextweaver_catalog": REF_ROOT
    / "contextweaver/src/contextweaver/routing/catalog.py",
    "graph_github_gold": REF_ROOT / "graph-tool-call/benchmarks/ground_truth/github.json",
    "graph_multi_gold": REF_ROOT / "graph-tool-call/benchmarks/ground_truth/multi_mcp.json",
    "graph_github_spec": REF_ROOT / "graph-tool-call/benchmarks/specs/github_subset.json",
    "mcpproxy_corpus": REF_ROOT
    / "mcpproxy-go/specs/065-evaluation-foundation/datasets/corpus_v1.tools.json",
    "mcpproxy_gold": REF_ROOT
    / "mcpproxy-go/specs/065-evaluation-foundation/datasets/retrieval_golden_v1.json",
    "livemcp_annotations": REF_ROOT / "LiveMCPBench/annotated_data/all_annotations.json",
    "livemcp_tools": REF_ROOT / "LiveMCPBench/tools/LiveMCPTool/tools.json",
    "mcpbench_single": REF_ROOT / "mcp-bench/tasks/mcpbench_tasks_single_runner_format.json",
    "mcpbench_multi2": REF_ROOT / "mcp-bench/tasks/mcpbench_tasks_multi_2server_runner_format.json",
    "mcpbench_multi3": REF_ROOT / "mcp-bench/tasks/mcpbench_tasks_multi_3server_runner_format.json",
}


def read_json_file(path: Path) -> Any:
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def relative_path(path: Path) -> str:
    return str(path.relative_to(ROOT))


def source_pointer(repo: str, path: Path, case_id: str | None = None) -> dict[str, str]:
    pointer = {"repo": repo, "path": relative_path(path)}
    if case_id:
        pointer["case_id"] = case_id
    return pointer


def slug_text(value: str) -> str:
    return re.sub(r"[^a-zA-Z0-9_:-]+", "_", value.strip()).strip("_")


def schema_from_names(names: list[str]) -> dict[str, Any]:
    properties = {
        name: {
            "type": "string",
            "description": name.replace("_", " ").replace("-", " "),
        }
        for name in names
        if name
    }
    return {
        "type": "object",
        "properties": properties,
        "required": list(properties)[:1],
    }


def schema_from_votr_params(params: dict[str, str] | None) -> dict[str, Any]:
    if not params:
        return schema_from_names(["input"])
    properties = {}
    required = []
    for name, description in params.items():
        is_optional = "optional" in str(description).lower()
        properties[name] = {
            "type": "string",
            "description": str(description),
        }
        if not is_optional:
            required.append(name)
    return {"type": "object", "properties": properties, "required": required[:8]}


def create_tool_record(
    *,
    tool_id: str,
    source_tool_id: str,
    source_repo: str,
    source_file: Path,
    server_id: str,
    server_name: str,
    name: str,
    description: str,
    input_schema: dict[str, Any] | None = None,
    tags: list[str] | None = None,
    metadata: dict[str, Any] | None = None,
) -> dict[str, Any]:
    return {
        "id": tool_id,
        "source_tool_id": source_tool_id,
        "server_id": server_id,
        "server_name": server_name,
        "name": name,
        "description": str(description or "").strip() or f"Tool named {name}",
        "input_schema": input_schema or schema_from_names(["input"]),
        "tags": sorted(set(tags or [])),
        "source": source_pointer(source_repo, source_file),
        "metadata": metadata or {},
    }


def add_tool_record(tools: dict[str, dict[str, Any]], record: dict[str, Any]) -> None:
    existing = tools.get(record["id"])
    if existing is None:
        tools[record["id"]] = record
        return
    if existing["description"] == f"Tool named {existing['name']}" and record["description"]:
        tools[record["id"]] = record


def add_derived_tool(
    tools: dict[str, dict[str, Any]],
    *,
    source_repo: str,
    source_file: Path,
    source_tool_id: str,
    description: str,
) -> str:
    if "::" in source_tool_id:
        server_name, tool_name = source_tool_id.split("::", 1)
        tool_id = f"votr.{server_name}::{tool_name}"
        server_id = f"votr.{slug_text(server_name).lower()}"
    elif "." in source_tool_id:
        namespace, tool_name = source_tool_id.rsplit(".", 1)
        tool_id = f"contextweaver.{source_tool_id}"
        server_id = f"contextweaver.{namespace.split('.', 1)[0]}"
        server_name = namespace.split(".", 1)[0]
        add_tool_record(
            tools,
            create_tool_record(
                tool_id=tool_id,
                source_tool_id=source_tool_id,
                source_repo=source_repo,
                source_file=source_file,
                server_id=server_id,
                server_name=server_name,
                name=tool_name,
                description=description,
                tags=namespace.split("."),
            ),
        )
        return tool_id
    else:
        server_name = "derived"
        tool_name = source_tool_id
        tool_id = f"derived.{slug_text(source_tool_id)}"
        server_id = "derived"

    add_tool_record(
        tools,
        create_tool_record(
            tool_id=tool_id,
            source_tool_id=source_tool_id,
            source_repo=source_repo,
            source_file=source_file,
            server_id=server_id,
            server_name=server_name,
            name=tool_name,
            description=description,
            tags=[slug_text(server_name).lower()],
            metadata={"derived_from_query_label": True},
        ),
    )
    return tool_id


def create_query_record(
    *,
    query_id: str,
    query: str,
    required_tool_ids: list[str],
    source_repo: str,
    source_file: Path,
    source_case_id: str,
    source_expected_tools: list[str],
    failure_modes: list[str],
    difficulty: str,
    notes: str,
    graded_relevance: list[dict[str, Any]] | None = None,
) -> dict[str, Any]:
    return {
        "id": query_id,
        "query": query.strip(),
        "connected_tool_pool": "all_tools",
        "required_tool_ids": required_tool_ids,
        "graded_relevance": graded_relevance
        or [{"tool_id": tool_id, "relevance": 2} for tool_id in required_tool_ids],
        "expected_servers": sorted({tool_id.split(".", 1)[0] for tool_id in required_tool_ids}),
        "source_expected_tools": source_expected_tools,
        "source": source_pointer(source_repo, source_file, source_case_id),
        "failure_modes": failure_modes,
        "difficulty": difficulty,
        "correctness_notes": notes,
        "should_route": bool(required_tool_ids),
    }


def load_votr_catalog_tools(tools: dict[str, dict[str, Any]]) -> None:
    for source_name, path in [
        ("votr_small_github", SOURCE_FILES["votr_small_github"]),
        ("votr_small_3", SOURCE_FILES["votr_small_3"]),
        ("votr_small_telegram", SOURCE_FILES["votr_small_telegram"]),
    ]:
        for server in read_json_file(path):
            server_name = server["name"]
            for tool in server.get("tools", []):
                source_tool_id = f"{server_name}::{tool['name']}"
                add_tool_record(
                    tools,
                    create_tool_record(
                        tool_id=f"votr.{source_tool_id}",
                        source_tool_id=source_tool_id,
                        source_repo="VOTR",
                        source_file=path,
                        server_id=f"votr.{slug_text(server_name).lower()}",
                        server_name=server_name,
                        name=tool["name"],
                        description=tool.get("description", ""),
                        input_schema=schema_from_votr_params(tool.get("parameter")),
                        tags=[slug_text(server_name).lower(), source_name],
                    ),
                )


def load_graph_mcp_tools(tools: dict[str, dict[str, Any]]) -> dict[str, str]:
    by_name: dict[str, str] = {}
    for path in sorted((REF_ROOT / "graph-tool-call/benchmarks/mcp_tools").glob("*.json")):
        data = read_json_file(path)
        server = data["name"]
        for tool in data.get("tools", []):
            tool_id = f"graph_mcp.{server}.{tool['name']}"
            by_name.setdefault(tool["name"], tool_id)
            add_tool_record(
                tools,
                create_tool_record(
                    tool_id=tool_id,
                    source_tool_id=tool["name"],
                    source_repo="graph-tool-call",
                    source_file=path,
                    server_id=f"graph_mcp.{server}",
                    server_name=server,
                    name=tool["name"],
                    description=tool.get("description", ""),
                    input_schema=tool.get("inputSchema") or tool.get("input_schema"),
                    tags=[server, "mcp"],
                    metadata={"annotations": tool.get("annotations", {})},
                ),
            )
    return by_name


def load_graph_openapi_tools(tools: dict[str, dict[str, Any]]) -> dict[str, str]:
    spec_path = SOURCE_FILES["graph_github_spec"]
    spec = read_json_file(spec_path)
    by_operation: dict[str, str] = {}
    for path_name, path_item in spec.get("paths", {}).items():
        for method, operation in path_item.items():
            if method.lower() not in {"get", "post", "put", "patch", "delete"}:
                continue
            operation_id = operation.get("operationId")
            if not operation_id:
                continue
            params = [param.get("name", "") for param in operation.get("parameters", [])]
            body_schema = (
                operation.get("requestBody", {})
                .get("content", {})
                .get("application/json", {})
                .get("schema", {})
            )
            params.extend((body_schema.get("properties") or {}).keys())
            description = " ".join(
                part
                for part in [operation.get("summary", ""), operation.get("description", "")]
                if part
            )
            tool_id = f"graph_openapi.github.{operation_id}"
            by_operation[operation_id] = tool_id
            add_tool_record(
                tools,
                create_tool_record(
                    tool_id=tool_id,
                    source_tool_id=operation_id,
                    source_repo="graph-tool-call",
                    source_file=spec_path,
                    server_id="graph_openapi.github",
                    server_name="GitHub REST API subset",
                    name=operation_id,
                    description=description,
                    input_schema=schema_from_names(list(dict.fromkeys(params))),
                    tags=operation.get("tags", []) + ["openapi", method.lower()],
                    metadata={"path": path_name, "method": method.upper()},
                ),
            )
    return by_operation


def parse_contextweaver_sample_families() -> dict[str, list[tuple[str, str, list[str]]]]:
    path = SOURCE_FILES["contextweaver_catalog"]
    module = ast.parse(path.read_text(encoding="utf-8"))
    for node in module.body:
        if isinstance(node, ast.AnnAssign) and getattr(node.target, "id", "") == "_SAMPLE_FAMILIES":
            return ast.literal_eval(node.value)
        if isinstance(node, ast.Assign):
            for target in node.targets:
                if getattr(target, "id", "") == "_SAMPLE_FAMILIES":
                    return ast.literal_eval(node.value)
    raise RuntimeError("Could not find _SAMPLE_FAMILIES in contextweaver catalog.py")


def load_contextweaver_tools(tools: dict[str, dict[str, Any]]) -> None:
    path = SOURCE_FILES["contextweaver_catalog"]
    for family, entries in parse_contextweaver_sample_families().items():
        for suffix, description, tags in entries:
            source_tool_id = f"{family}.{suffix}"
            add_tool_record(
                tools,
                create_tool_record(
                    tool_id=f"contextweaver.{source_tool_id}",
                    source_tool_id=source_tool_id,
                    source_repo="contextweaver",
                    source_file=path,
                    server_id=f"contextweaver.{family}",
                    server_name=family,
                    name=suffix.replace(".", "_"),
                    description=description,
                    input_schema={},
                    tags=tags,
                ),
            )


def load_mcpproxy_tools(tools: dict[str, dict[str, Any]]) -> None:
    path = SOURCE_FILES["mcpproxy_corpus"]
    for tool in read_json_file(path)["tools"]:
        source_tool_id = tool["tool_id"]
        add_tool_record(
            tools,
            create_tool_record(
                tool_id=f"mcpproxy.{source_tool_id}",
                source_tool_id=source_tool_id,
                source_repo="mcpproxy-go",
                source_file=path,
                server_id=f"mcpproxy.{tool['server']}",
                server_name=tool["server"],
                name=tool["tool"],
                description=tool.get("description", ""),
                input_schema=tool.get("input_schema") or tool.get("schema"),
                tags=[tool["server"]],
            ),
        )


def load_livemcp_tools(tools: dict[str, dict[str, Any]]) -> dict[str, list[str]]:
    path = SOURCE_FILES["livemcp_tools"]
    by_name: dict[str, list[str]] = defaultdict(list)
    for package in read_json_file(path):
        package_name = package.get("name", "LiveMCP package")
        category = package.get("category", "")
        for server_name, block in (package.get("tools") or {}).items():
            for tool in block.get("tools", []):
                tool_name = tool["name"]
                tool_id = f"livemcp.{slug_text(server_name).lower()}.{slug_text(tool_name)}"
                by_name[tool_name].append(tool_id)
                add_tool_record(
                    tools,
                    create_tool_record(
                        tool_id=tool_id,
                        source_tool_id=tool_name,
                        source_repo="LiveMCPBench",
                        source_file=path,
                        server_id=f"livemcp.{slug_text(server_name).lower()}",
                        server_name=server_name,
                        name=tool_name,
                        description=tool.get("description", package.get("description", "")),
                        input_schema=tool.get("inputSchema") or tool.get("input_schema"),
                        tags=[category, package_name],
                    ),
                )
    return by_name


def extract_livemcp_tool_names(row: dict[str, Any]) -> list[str]:
    tools_text = (row.get("Annotator Metadata") or {}).get("Tools", "")
    names: list[str] = []
    for line in tools_text.splitlines():
        cleaned = re.sub(r"^\s*\d+\.\s*", "", line).strip()
        if cleaned:
            names.append(cleaned)
    return names


def ensure_livemcp_tools(
    tools: dict[str, dict[str, Any]],
    by_name: dict[str, list[str]],
    tool_names: list[str],
) -> list[str]:
    required: list[str] = []
    path = SOURCE_FILES["livemcp_tools"]
    for name in tool_names:
        ids = by_name.get(name)
        if ids:
            required.append(ids[0])
            continue
        tool_id = f"livemcp.derived.{slug_text(name)}"
        add_tool_record(
            tools,
            create_tool_record(
                tool_id=tool_id,
                source_tool_id=name,
                source_repo="LiveMCPBench",
                source_file=path,
                server_id="livemcp.derived",
                server_name="LiveMCPBench derived",
                name=name,
                description=f"Tool required by LiveMCPBench annotation: {name}",
                tags=["livemcp", "derived"],
                metadata={"derived_from_annotation": True},
            ),
        )
        required.append(tool_id)
    return required


def extract_mcpbench_tool_names(
    description: str,
    fallback_server: str | None = None,
) -> list[tuple[str, str]]:
    known_servers = [
        "BioMCP",
        "Paper Search",
        "OpenAPI Explorer",
        "Google Maps",
        "Weather Data",
        "National Parks",
    ]
    server_pattern = "|".join(re.escape(server) for server in known_servers)
    matches = re.findall(rf"\b({server_pattern})\s*:\s*([A-Za-z][A-Za-z0-9_-]+)", description)
    cleaned: list[tuple[str, str]] = []
    for server, tool in matches:
        server = server.strip(" .,-")
        cleaned.append((server, tool.strip()))
    if fallback_server:
        for tool in re.findall(
            r"\b(?:call|use|using|re-use|reuse)\s+([A-Za-z][A-Za-z0-9_-]+)\b",
            description,
            flags=re.IGNORECASE,
        ):
            if tool.lower() in {"and", "the", "a", "an", "its", "it"}:
                continue
            cleaned.append((fallback_server, tool.strip()))
    return list(dict.fromkeys(cleaned))


def ensure_mcpbench_tools(
    tools: dict[str, dict[str, Any]],
    tool_pairs: list[tuple[str, str]],
    source_file: Path,
) -> list[str]:
    required: list[str] = []
    for server, tool_name in tool_pairs:
        server_slug = slug_text(server).lower()
        tool_id = f"mcpbench.{server_slug}.{slug_text(tool_name)}"
        add_tool_record(
            tools,
            create_tool_record(
                tool_id=tool_id,
                source_tool_id=f"{server}:{tool_name}",
                source_repo="mcp-bench",
                source_file=source_file,
                server_id=f"mcpbench.{server_slug}",
                server_name=server,
                name=tool_name,
                description=f"MCP-Bench referenced tool {server}:{tool_name}",
                tags=["mcpbench", server_slug],
                metadata={"derived_from_task_description": True},
            ),
        )
        required.append(tool_id)
    return required


def build_votr_queries(tools: dict[str, dict[str, Any]]) -> list[dict[str, Any]]:
    queries: list[dict[str, Any]] = []
    ambiguity = read_json_file(SOURCE_FILES["votr_ambiguity"])["cases"]
    robustness = read_json_file(SOURCE_FILES["votr_robustness"])["cases"]
    negatives = read_json_file(SOURCE_FILES["votr_negative"])["cases"]
    multi = read_json_file(SOURCE_FILES["votr_multi"])["cases"]

    for row in ambiguity[:12]:
        expected = row["expected_tool_key"]
        required = [
            add_derived_tool(
                tools,
                source_repo="VOTR",
                source_file=SOURCE_FILES["votr_ambiguity"],
                source_tool_id=expected,
                description=row["tool_intent"],
            )
        ]
        queries.append(
            create_query_record(
                query_id=f"TRQ-{len(queries)+1:03d}",
                query=f"{row['tool_intent']} ({row['server_intent']})",
                required_tool_ids=required,
                source_repo="VOTR",
                source_file=SOURCE_FILES["votr_ambiguity"],
                source_case_id=row["id"],
                source_expected_tools=[expected],
                failure_modes=["ambiguous_tool_name", "near_duplicate_capability"],
                difficulty="hard",
                notes="Router-only ambiguity/collision case from VOTR.",
            )
        )

    for row in robustness[:4]:
        expected = row["expected_tool_key"]
        required = [
            add_derived_tool(
                tools,
                source_repo="VOTR",
                source_file=SOURCE_FILES["votr_robustness"],
                source_tool_id=expected,
                description=row["tool_intent"],
            )
        ]
        queries.append(
            create_query_record(
                query_id=f"TRQ-{len(queries)+1:03d}",
                query=f"{row['tool_intent']} ({row['server_intent']})",
                required_tool_ids=required,
                source_repo="VOTR",
                source_file=SOURCE_FILES["votr_robustness"],
                source_case_id=row["id"],
                source_expected_tools=[expected],
                failure_modes=["noisy_user_language", "typo_or_abbreviation"],
                difficulty="medium",
                notes="Router robustness case with informal or misspelled phrasing.",
            )
        )

    for row in [negatives[i] for i in [0, 3, 5, 7]]:
        queries.append(
            create_query_record(
                query_id=f"TRQ-{len(queries)+1:03d}",
                query=f"{row['tool_intent']} ({row['server_intent']})",
                required_tool_ids=[],
                source_repo="VOTR",
                source_file=SOURCE_FILES["votr_negative"],
                source_case_id=row["id"],
                source_expected_tools=[],
                failure_modes=["unsupported_intent", "abstention_required"],
                difficulty="hard",
                notes="Must return no route or empty handoff set.",
                graded_relevance=[],
            )
        )

    for row in multi[:2]:
        expected_tools = [sub["expected_tool_key"] for sub in row["subtasks"]]
        required = [
            add_derived_tool(
                tools,
                source_repo="VOTR",
                source_file=SOURCE_FILES["votr_multi"],
                source_tool_id=tool_id,
                description=next(
                    sub["tool_intent"] for sub in row["subtasks"] if sub["expected_tool_key"] == tool_id
                ),
            )
            for tool_id in expected_tools
        ]
        queries.append(
            create_query_record(
                query_id=f"TRQ-{len(queries)+1:03d}",
                query=row["user_intent"],
                required_tool_ids=required,
                source_repo="VOTR",
                source_file=SOURCE_FILES["votr_multi"],
                source_case_id=row["id"],
                source_expected_tools=expected_tools,
                failure_modes=["multi_tool_single_turn", "ordered_subtasks"],
                difficulty="hard",
                notes="Multi-target router case: recall over all required tools matters more than top-1.",
            )
        )
    return queries


def build_contextweaver_queries() -> list[dict[str, Any]]:
    rows = read_json_file(SOURCE_FILES["contextweaver_gold"])
    by_namespace: dict[str, dict[str, Any]] = {}
    for row in rows:
        by_namespace.setdefault(row["namespace"], row)

    queries: list[dict[str, Any]] = []
    for namespace in sorted(by_namespace):
        row = by_namespace[namespace]
        required = [f"contextweaver.{tool_id}" for tool_id in row["expected"]]
        queries.append(
            create_query_record(
                query_id="",
                query=row["query"],
                required_tool_ids=required,
                source_repo="contextweaver",
                source_file=SOURCE_FILES["contextweaver_gold"],
                source_case_id=f"namespace:{namespace}",
                source_expected_tools=row["expected"],
                failure_modes=["namespace_breadth", "exact_route_gold"],
                difficulty="medium",
                notes="One representative hand-crafted routing-gold query per contextweaver namespace.",
            )
        )
    return queries


def build_graph_queries(
    graph_openapi_by_id: dict[str, str],
    graph_mcp_by_name: dict[str, str],
    tools: dict[str, dict[str, Any]],
) -> list[dict[str, Any]]:
    queries: list[dict[str, Any]] = []
    github_rows = read_json_file(SOURCE_FILES["graph_github_gold"])["queries"]
    multi_rows = read_json_file(SOURCE_FILES["graph_multi_gold"])["queries"]
    for index in [0, 6, 8, 11]:
        row = github_rows[index]
        required = [graph_openapi_by_id[name] for name in row["expected_tools"]]
        queries.append(
            create_query_record(
                query_id="",
                query=row["query"],
                required_tool_ids=required,
                source_repo="graph-tool-call",
                source_file=SOURCE_FILES["graph_github_gold"],
                source_case_id=f"github:{index}",
                source_expected_tools=row["expected_tools"],
                failure_modes=["openapi_tool_selection", row.get("category", "unknown")],
                difficulty=row.get("difficulty", "medium"),
                notes="Graph-tool-call GitHub OpenAPI ground-truth case.",
            )
        )
    for index in [0, 1, 2, 6]:
        row = multi_rows[index]
        required: list[str] = []
        for name in row["expected_tools"]:
            tool_id = graph_mcp_by_name.get(name)
            if not tool_id:
                tool_id = add_derived_tool(
                    tools,
                    source_repo="graph-tool-call",
                    source_file=SOURCE_FILES["graph_multi_gold"],
                    source_tool_id=name,
                    description=f"Graph-tool-call multi-MCP expected tool: {name}",
                )
            required.append(tool_id)
        queries.append(
            create_query_record(
                query_id="",
                query=row["query"],
                required_tool_ids=required,
                source_repo="graph-tool-call",
                source_file=SOURCE_FILES["graph_multi_gold"],
                source_case_id=f"multi_mcp:{index}",
                source_expected_tools=row["expected_tools"],
                failure_modes=["multi_server_mcp", row.get("category", "unknown")],
                difficulty=row.get("difficulty", "medium"),
                notes="Graph-tool-call mixed MCP ground-truth case.",
            )
        )
    return queries


def build_mcpproxy_queries() -> list[dict[str, Any]]:
    selected = {
        "q-fs-search",
        "q-git-status",
        "q-mem-search",
        "q-sql-select",
        "q-fetch",
        "q-time-now",
    }
    rows = read_json_file(SOURCE_FILES["mcpproxy_gold"])["queries"]
    queries: list[dict[str, Any]] = []
    for row in rows:
        if row["id"] not in selected:
            continue
        graded = [
            {"tool_id": f"mcpproxy.{label['tool_id']}", "relevance": label["relevance"]}
            for label in row["labels"]
            if label["relevance"] > 0
        ]
        required = [item["tool_id"] for item in graded if item["relevance"] >= 2]
        queries.append(
            create_query_record(
                query_id="",
                query=row["query"],
                required_tool_ids=required,
                source_repo="mcpproxy-go",
                source_file=SOURCE_FILES["mcpproxy_gold"],
                source_case_id=row["id"],
                source_expected_tools=[label["tool_id"] for label in row["labels"]],
                failure_modes=["graded_relevance", "hard_negative"] if "notes" in row else ["graded_relevance"],
                difficulty="medium",
                notes=row.get("notes", "Frozen-corpus tool retrieval golden query."),
                graded_relevance=graded,
            )
        )
    return queries


def build_livemcp_queries(
    tools: dict[str, dict[str, Any]],
    by_name: dict[str, list[str]],
) -> list[dict[str, Any]]:
    rows = read_json_file(SOURCE_FILES["livemcp_annotations"])
    queries: list[dict[str, Any]] = []
    for index in [0, 7, 9]:
        row = rows[index]
        names = extract_livemcp_tool_names(row)
        required = ensure_livemcp_tools(tools, by_name, names)
        queries.append(
            create_query_record(
                query_id="",
                query=row["Question"],
                required_tool_ids=required,
                source_repo="LiveMCPBench",
                source_file=SOURCE_FILES["livemcp_annotations"],
                source_case_id=row["task_id"],
                source_expected_tools=names,
                failure_modes=["real_mcp_task", "multi_step_task"],
                difficulty="hard" if len(required) > 3 else "medium",
                notes=f"Annotated LiveMCPBench task in category {row.get('category', 'unknown')}.",
            )
        )
    return queries


def build_mcpbench_queries(tools: dict[str, dict[str, Any]]) -> list[dict[str, Any]]:
    queries: list[dict[str, Any]] = []
    fallback_tools = {
        "google_maps_weather_data_national_parks_000": [
            ("National Parks", "findParks"),
            ("National Parks", "getCampgrounds"),
            ("National Parks", "getAlerts"),
            ("National Parks", "getVisitorCenters"),
            ("Google Maps", "geocode"),
            ("Google Maps", "getDirections"),
            ("Google Maps", "calculateDistanceMatrix"),
            ("Google Maps", "getElevation"),
            ("Google Maps", "searchNearbyPlaces"),
            ("Weather Data", "get_weather_forecast_tool"),
        ]
    }
    specs = [
        (SOURCE_FILES["mcpbench_single"], 0, 0),
        (SOURCE_FILES["mcpbench_multi2"], 0, 0),
        (SOURCE_FILES["mcpbench_multi3"], 0, 0),
    ]
    for source_file, server_index, task_index in specs:
        data = read_json_file(source_file)
        server_block = data["server_tasks"][server_index]
        row = server_block["tasks"][task_index]
        fallback_server = server_block["server_name"]
        if "+" in fallback_server:
            fallback_server = None
        tool_pairs = extract_mcpbench_tool_names(row["task_description"], fallback_server)[:8]
        if not tool_pairs:
            tool_pairs = fallback_tools.get(row["task_id"], [])
        required = ensure_mcpbench_tools(tools, tool_pairs, source_file)
        queries.append(
            create_query_record(
                query_id="",
                query=row["task_description"],
                required_tool_ids=required,
                source_repo="mcp-bench",
                source_file=source_file,
                source_case_id=row["task_id"],
                source_expected_tools=[f"{server}:{tool}" for server, tool in tool_pairs],
                failure_modes=["mcp_task_runner", "multi_server_task" if "multi" in source_file.name else "single_server_task"],
                difficulty="hard",
                notes=f"MCP-Bench task from server group {server_block['server_name']}.",
            )
        )
    return queries


def assign_query_ids(queries: list[dict[str, Any]]) -> None:
    for index, query in enumerate(queries, 1):
        query["id"] = f"TRQ-{index:03d}"


def build_subset() -> tuple[list[dict[str, Any]], list[dict[str, Any]], dict[str, Any]]:
    tools: dict[str, dict[str, Any]] = {}
    load_votr_catalog_tools(tools)
    graph_mcp_by_name = load_graph_mcp_tools(tools)
    graph_openapi_by_id = load_graph_openapi_tools(tools)
    load_contextweaver_tools(tools)
    load_mcpproxy_tools(tools)
    livemcp_by_name = load_livemcp_tools(tools)

    queries: list[dict[str, Any]] = []
    queries.extend(build_votr_queries(tools))
    queries.extend(build_contextweaver_queries())
    queries.extend(build_graph_queries(graph_openapi_by_id, graph_mcp_by_name, tools))
    queries.extend(build_mcpproxy_queries())
    queries.extend(build_livemcp_queries(tools, livemcp_by_name))
    queries.extend(build_mcpbench_queries(tools))
    assign_query_ids(queries)

    tool_list = sorted(tools.values(), key=lambda item: item["id"])
    manifest = create_manifest(tool_list, queries)
    return tool_list, queries, manifest


def create_manifest(tools: list[dict[str, Any]], queries: list[dict[str, Any]]) -> dict[str, Any]:
    return {
        "name": "tool-routing-takehome-reference-subset",
        "version": "2026-06-30",
        "description": (
            "A 50-query, provenance-preserving subset assembled from local benchmark "
            "reference repositories for evaluating pre-model MCP/tool routing."
        ),
        "assignment_fit": {
            "router_input": "user query plus tool metadata only",
            "primary_metric": "required-tool recall@k over the returned candidate subset",
            "secondary_metrics": [
                "abstention accuracy for no-route cases",
                "graded nDCG@10 for mcpproxy-go relevance labels",
                "candidate token reduction versus full catalog",
                "latency over catalog size",
            ],
            "baseline": "BM25 or TF-IDF over name + description + JSON schema text",
            "do_not_measure": "final LLM answer correctness; this subset isolates routing quality",
        },
        "counts": {
            "tools": len(tools),
            "queries": len(queries),
            "route_required_queries": sum(1 for query in queries if query["should_route"]),
            "abstention_queries": sum(1 for query in queries if not query["should_route"]),
        },
        "source_repo_counts": Counter(query["source"]["repo"] for query in queries),
        "tool_source_repo_counts": Counter(tool["source"]["repo"] for tool in tools),
        "failure_mode_counts": Counter(
            mode for query in queries for mode in query["failure_modes"]
        ),
        "source_files": {name: relative_path(path) for name, path in SOURCE_FILES.items()},
    }


def write_readme(manifest: dict[str, Any]) -> str:
    counts = manifest["counts"]
    return f"""# Tool Routing Reference Subset

This is a small, reference-derived benchmark subset for the take-home exercise.
It is not a full benchmark suite; it is the fastest defensible slice from the
local reference shelf.

## Contents

- `tools.json`: {counts["tools"]} tool metadata records. Each record has name,
  description, schema, source repo, and source file.
- `queries.json`: {counts["queries"]} routing queries. Each query lists required
  tool ids, graded relevance when the source has it, failure modes, and source
  provenance.
- `manifest.json`: source counts, intended metrics, and assignment-fit notes.
- `source-audit.md`: what each reference repo contributed or why it was not
  selected as a primary source.

## Recommended Evaluation

Use the router to return a candidate subset for each query. Score:

- `Recall@k`: every required tool should appear in the returned subset.
- `Abstention accuracy`: no-route cases should return an empty or null route.
- `nDCG@10`: only for queries with graded relevance from mcpproxy-go.
- `Token reduction`: compare full catalog schema tokens with selected candidate
  schema tokens.
- `Latency`: report p50/p95/p99 routing time.

This matches the assignment because it evaluates the routing layer before the
model reasons or calls tools.

## Baseline Command

Run the dependency-free lexical sanity baseline:

```bash
python3 scripts/run_tool_routing_baseline.py
```

The default threshold is intentionally simple; expect abstention to be weak.
"""


def write_source_audit() -> str:
    return """# Reference Repo Audit

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
"""


def validate_subset(tools: list[dict[str, Any]], queries: list[dict[str, Any]]) -> None:
    tool_ids = {tool["id"] for tool in tools}
    if len(queries) != 50:
        raise AssertionError(f"expected 50 queries, found {len(queries)}")
    if len(tool_ids) != len(tools):
        raise AssertionError("duplicate tool ids in catalog")
    for query in queries:
        missing = [tool_id for tool_id in query["required_tool_ids"] if tool_id not in tool_ids]
        if missing:
            raise AssertionError(f"{query['id']} references missing tools: {missing}")
        for item in query["graded_relevance"]:
            if item["tool_id"] not in tool_ids:
                raise AssertionError(f"{query['id']} graded relevance references {item['tool_id']}")
        if query["should_route"] != bool(query["required_tool_ids"]):
            raise AssertionError(f"{query['id']} has inconsistent should_route flag")
    abstentions = [query for query in queries if not query["should_route"]]
    if len(abstentions) < 4:
        raise AssertionError("expected at least four abstention cases")


def write_outputs() -> None:
    tools, queries, manifest = build_subset()
    validate_subset(tools, queries)
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    (OUT_DIR / "tools.json").write_text(
        json.dumps(tools, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    (OUT_DIR / "queries.json").write_text(
        json.dumps(queries, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    (OUT_DIR / "manifest.json").write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    (OUT_DIR / "README.md").write_text(write_readme(manifest), encoding="utf-8")
    (OUT_DIR / "source-audit.md").write_text(write_source_audit(), encoding="utf-8")
    print(
        json.dumps(
            {
                "output_dir": relative_path(OUT_DIR),
                "tools": len(tools),
                "queries": len(queries),
                "abstentions": manifest["counts"]["abstention_queries"],
                "source_repo_counts": manifest["source_repo_counts"],
            },
            indent=2,
            ensure_ascii=False,
        )
    )


if __name__ == "__main__":
    write_outputs()
