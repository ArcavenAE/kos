"""Compare qtime output with the outside truth; p95 of ms. Usage: g3check.py <truth.jsonl> <qtime.jsonl>"""
import json, sys, math
t = {r["pair"]: r for r in map(json.loads, open(sys.argv[1]))}
rows = list(map(json.loads, open(sys.argv[2])))
eq = sum(1 for r in rows if t[r["label"]]["added"] == r["c1"] and t[r["label"]]["removed"] == r["c2"])
ms = sorted(r["ms"] for r in rows); p95 = ms[math.ceil(0.95 * len(ms)) - 1]
print(json.dumps({"pairs": len(rows), "equal": eq, "differ": len(rows) - eq, "p95_ms": p95, "max_ms": ms[-1], "median_ms": ms[len(ms)//2]}))
