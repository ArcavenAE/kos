"""Probe C draws, in the brief's order: 20 G4 commits, 50 G3 pairs, then seeds.
Usage: python3 -I draws.py <index.jsonl> <facts-dir> <outdir>"""
import collections, hashlib, json, random, sys
idx, fdir, od = sys.argv[1:4]
SEED = 20261022; LAST = "d0616f9841bcb4bd78479d64ded42151b8ab0816"
ok = [(r["i"], r["commit"]) for r in (json.loads(l) for l in open(idx) if l.startswith('{"i"')) if r["status"] == "ok"]
ok.sort()
def w(name, lines):
    t = "\n".join(lines) + "\n"; open(f"{od}/{name}", "w").write(t)
    return hashlib.sha256(t.encode()).hexdigest()
rep = {"indexed": len(ok), "indexed_list_sha256": w("indexed.txt", [f"{i} {c}" for i, c in ok])}
g4 = sorted(random.Random(SEED).sample(ok, 20))
rep["g4_sha256"] = w("g4-commits.txt", [f"{i} {c}" for i, c in g4])
pairs = [(a, b) for x, a in enumerate(ok) for b in ok[x+1:]]
rep["pair_list"] = "all (i, j), i < j, over indexed.txt, in lexicographic order of (i, j)"
rep["pair_list_size"] = len(pairs)
g3 = sorted(random.Random(SEED).sample(pairs, 50))
rep["g3_sha256"] = w("g3-pairs.txt", [f"{a[0]} {a[1]} {b[0]} {b[1]}" for a, b in g3])
def load(c):
    refs, defs = collections.defaultdict(set), set()
    for r in map(json.loads, open(f"{fdir}/{c[:8]}.jsonl")):
        if r["a"] == "REFERENCES": refs[r["v"]].add(r["e"])
        elif r["a"] == "DEFINES": defs.add(r["v"])
    return refs, defs
last_refs, last_defs = load(LAST)
pool = {s for s in last_defs if last_refs.get(s)}
for i, c in g4:
    refs, _ = load(c); pool = {s for s in pool if refs.get(s)}
pool = sorted(pool); rep["seed_pool"] = len(pool)
seeds = random.Random(SEED).sample(pool, min(100, len(pool)))
rep["seeds"] = len(seeds); rep["seeds_sha256"] = w("seeds.txt", seeds)
hot = sorted(last_defs, key=lambda s: (-len(last_refs.get(s, ())), s))[:100]
rep["hot_sha256"] = w("hot.txt", hot)
empty = {}
for i, c in g4 + ([] if any(c == LAST for _, c in g4) else [(200, LAST)]):
    refs, _ = load(c)
    empty[c[:8]] = {"seeds_empty": sum(1 for s in seeds if not refs.get(s)), "hot_empty": sum(1 for s in hot if not refs.get(s))}
rep["empty_counts"] = empty
json.dump(rep, open(f"{od}/draws.json", "w"), indent=1); print(json.dumps(rep, indent=1))
