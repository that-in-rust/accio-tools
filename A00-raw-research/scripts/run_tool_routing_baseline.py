#!/usr/bin/env python3
"""Run a small dependency-free lexical baseline over the tool-routing subset."""

from __future__ import annotations

import argparse
import json
import math
import re
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_DATASET = ROOT / "benchmarks" / "tool-routing-subset"


def tokenize_text(value: str) -> list[str]:
    return re.findall(r"[a-z0-9]+", value.lower())


def flatten_metadata(value: Any) -> str:
    if isinstance(value, dict):
        return " ".join(flatten_metadata(v) for v in value.values())
    if isinstance(value, list):
        return " ".join(flatten_metadata(v) for v in value)
    return str(value)


def tool_search_text(tool: dict[str, Any]) -> str:
    return " ".join(
        [
            tool.get("id", ""),
            tool.get("source_tool_id", ""),
            tool.get("server_name", ""),
            tool.get("name", ""),
            tool.get("description", ""),
            " ".join(tool.get("tags", [])),
            flatten_metadata(tool.get("input_schema", {})),
        ]
    )


def build_tool_index(tools: list[dict[str, Any]]) -> tuple[dict[str, Counter[str]], dict[str, float]]:
    term_counts: dict[str, Counter[str]] = {}
    document_frequency: Counter[str] = Counter()
    for tool in tools:
        counts = Counter(tokenize_text(tool_search_text(tool)))
        term_counts[tool["id"]] = counts
        document_frequency.update(counts.keys())
    total_documents = len(tools)
    idf = {
        term: math.log((1 + total_documents) / (1 + frequency)) + 1.0
        for term, frequency in document_frequency.items()
    }
    return term_counts, idf


def rank_tools_for_query(
    query: str,
    tool_counts: dict[str, Counter[str]],
    idf: dict[str, float],
    threshold: float,
) -> list[str]:
    query_terms = Counter(tokenize_text(query))
    scores: list[tuple[float, str]] = []
    for tool_id, counts in tool_counts.items():
        score = 0.0
        for term, query_count in query_terms.items():
            if term in counts:
                score += query_count * idf.get(term, 1.0) * (1.0 + math.log(1 + counts[term]))
        if score >= threshold:
            scores.append((score, tool_id))
    return [tool_id for _, tool_id in sorted(scores, reverse=True)]


def discounted_gain(relevance: int, rank: int) -> float:
    return (2**relevance - 1) / math.log2(rank + 1)


def score_predictions(
    queries: list[dict[str, Any]],
    predictions: dict[str, list[str]],
    k_values: list[int],
) -> dict[str, Any]:
    routed = [query for query in queries if query["should_route"]]
    abstentions = [query for query in queries if not query["should_route"]]
    recall_sums = defaultdict(float)
    mrr_sum = 0.0
    ndcg_sum = 0.0
    ndcg_count = 0

    for query in routed:
        ranked = predictions[query["id"]]
        required = set(query["required_tool_ids"])
        for k in k_values:
            selected = set(ranked[:k])
            recall_sums[k] += len(required & selected) / len(required)
        reciprocal_rank = 0.0
        for rank, tool_id in enumerate(ranked, 1):
            if tool_id in required:
                reciprocal_rank = 1.0 / rank
                break
        mrr_sum += reciprocal_rank

        relevance_by_tool = {
            item["tool_id"]: item["relevance"] for item in query.get("graded_relevance", [])
        }
        if relevance_by_tool:
            actual = sum(
                discounted_gain(relevance_by_tool.get(tool_id, 0), rank)
                for rank, tool_id in enumerate(ranked[:10], 1)
            )
            ideal_values = sorted(relevance_by_tool.values(), reverse=True)[:10]
            ideal = sum(discounted_gain(rel, rank) for rank, rel in enumerate(ideal_values, 1))
            ndcg_sum += actual / ideal if ideal else 0.0
            ndcg_count += 1

    abstention_hits = sum(1 for query in abstentions if not predictions[query["id"]])
    routed_count = max(1, len(routed))
    return {
        "queries": len(queries),
        "route_required_queries": len(routed),
        "abstention_queries": len(abstentions),
        "recall_at_k": {
            str(k): round(recall_sums[k] / routed_count, 4) for k in k_values
        },
        "mrr": round(mrr_sum / routed_count, 4),
        "ndcg_at_10": round(ndcg_sum / max(1, ndcg_count), 4),
        "abstention_accuracy": round(
            abstention_hits / max(1, len(abstentions)),
            4,
        ),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dataset", type=Path, default=DEFAULT_DATASET)
    parser.add_argument("--threshold", type=float, default=2.0)
    parser.add_argument("--max-k", type=int, default=10)
    args = parser.parse_args()

    tools = json.loads((args.dataset / "tools.json").read_text(encoding="utf-8"))
    queries = json.loads((args.dataset / "queries.json").read_text(encoding="utf-8"))
    tool_counts, idf = build_tool_index(tools)
    predictions = {
        query["id"]: rank_tools_for_query(query["query"], tool_counts, idf, args.threshold)[
            : args.max_k
        ]
        for query in queries
    }
    result = score_predictions(queries, predictions, [1, 3, 5, 10])
    result["baseline"] = {
        "name": "lexical_tfidf_overlap",
        "threshold": args.threshold,
        "max_k": args.max_k,
    }
    print(json.dumps(result, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
