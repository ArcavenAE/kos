"""G4/G5 scoring. Usage: g4check.py <facts-dir> <summary.jsonl> <outdir> <arm> [check_sets]
Per run: slowest_ms over all commits; correctness on run 1 against the plain join."""
import collections, json, re, statistics, sys
fdir, summ, od, arm = sys.argv[1:5]; check = len(sys.argv) > 5
rows = [r for r in map(json.loads, open(summ)) if r["arm"] == arm]
per_run = collections.defaultdict(float)
for r in rows:
    per_run[r["run"]] = max(per_run[r["run"]], float(re.search(r"slowest_ms=([0-9.]+)", r["summary"]).group(1)))
res = {"arm": arm, "runs": dict(per_run), "median_of_slowest_ms": statistics.median(per_run.values())}
if check:
    bad = 0; seen = 0; empty = {}
    for r in rows:
        if r["run"] != 1: continue
        refs, defs = collections.defaultdict(set), collections.defaultdict(set)
        for f in map(json.loads, open(f"{fdir}/{r['commit'][:8]}.jsonl")):
            if f["a"] == "REFERENCES": refs[f["v"]].add(f["e"])
            elif f["a"] == "DEFINES": defs[f["e"]].add(f["v"])
        e = 0
        for t in map(json.loads, open(f"{od}/{arm}-r1-{r['commit'][:8]}.jsonl")):
            h1 = refs[t["s"]]; h2 = set().union(*(defs[d] for d in h1)) if h1 else set()
            h3 = set().union(*(refs[y] for y in h2)) if h2 else set()
            seen += 1; e += (not h1)
            bad += not (h1 == set(t["hop1_set"]) and h2 == set(t["hop2_set"]) and h3 == set(t["result"]))
        empty[r["commit"][:8]] = e
    res.update({"traversals_checked": seen, "hop_sets_differ": bad, "empty_seed_by_commit": empty})
print(json.dumps(res))
