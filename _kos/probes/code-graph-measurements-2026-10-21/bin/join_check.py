"""Plain set join outside the store, compared with trav results, hop by hop as sets.
Usage: join_check.py <facts.jsonl> <results.jsonl>"""
import json, sys, collections
refs, defs = collections.defaultdict(set), collections.defaultdict(set)
for r in map(json.loads, open(sys.argv[1])):
    if r["a"] == "REFERENCES": refs[r["v"]].add(r["e"])
    elif r["a"] == "DEFINES": defs[r["e"]].add(r["v"])
ok = bad = 0; sizes = []; sets_checked = 0
for row in map(json.loads, open(sys.argv[2])):
    h1 = refs[row["s"]]; h2 = set().union(*(defs[d] for d in h1)) if h1 else set()
    h3 = set().union(*(refs[y] for y in h2)) if h2 else set()
    if "hop1_set" in row:
        sets_checked += 1
        same = h1 == set(row["hop1_set"]) and h2 == set(row["hop2_set"]) and h3 == set(row["result"])
    else:
        same = h3 == set(row["result"]) and len(h1) == row["hop1"] and len(h2) == row["hop2"]
    ok += same; bad += (not same); sizes.append(len(h3))
print("equal", ok, "differ", bad, "hop sets compared", sets_checked, "empty results", sum(1 for x in sizes if x == 0), "max result", max(sizes))
