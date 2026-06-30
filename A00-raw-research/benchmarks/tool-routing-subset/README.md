# Tool Routing Reference Subset

This is a small, reference-derived benchmark subset for the take-home exercise.
It is not a full benchmark suite; it is the fastest defensible slice from the
local reference shelf.

## Contents

- `tools.json`: 947 tool metadata records. Each record has name,
  description, schema, source repo, and source file.
- `queries.json`: 50 routing queries. Each query lists required
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
