"""D lock-in and round-trip, dolt CLI with the server stopped.
Usage: mde_d.py <dolt-repo> <load-d.jsonl> <schema-hash> <g3-pairs.txt> <new-repo> <work> <outdir>"""
import csv, json, os, random, subprocess, sys, time, yaml
repo, dload, base, pairsf, newrepo, work, od = sys.argv[1:8]
def q(r, sql):
    p = subprocess.run(["dolt", "sql", "-r", "json", "-q", sql], cwd=r, capture_output=True)
    if p.returncode: raise SystemExit(p.stderr.decode(errors="replace")[-300:])
    out = p.stdout.decode().strip()
    return json.loads(out).get("rows", []) if out else []  # an empty result prints {}
def dolt(r, *a):
    p = subprocess.run(["dolt", *a], cwd=r, capture_output=True)
    if p.returncode: raise SystemExit(p.stderr.decode(errors="replace")[-300:])
loads = [r for r in map(json.loads, open(dload)) if "dolt_hash" in r]
t0 = time.monotonic(); prev = base
with open(f"{od}/d-export.jsonl", "w") as fh:
    for r in loads:
        for x in q(repo, f"SELECT diff_type, from_e, from_a, from_v, to_e, to_a, to_v, to_tx FROM dolt_diff('{prev}','{r['dolt_hash']}','facts')"):
            if x["diff_type"] == "added": fh.write(json.dumps({"i": r["i"], "op": "add", "e": x["to_e"], "a": x["to_a"], "v": x["to_v"], "tx": x["to_tx"]}) + "\n")
            elif x["diff_type"] == "removed": fh.write(json.dumps({"i": r["i"], "op": "del", "e": x["from_e"], "a": x["from_a"], "v": x["from_v"]}) + "\n")
            else: fh.write(json.dumps({"i": r["i"], "op": "mod", "e": x["to_e"], "a": x["to_a"], "v": x["to_v"], "tx": x["to_tx"]}) + "\n")
        fh.write(json.dumps({"i": r["i"], "op": "commit", "commit": r["commit"]}) + "\n"); prev = r["dolt_hash"]
t1 = time.monotonic()
state = {}; newh = {}
for ev in map(json.loads, open(f"{od}/d-export.jsonl")):
    k = (ev.get("e"), ev.get("a"), ev.get("v"))
    if ev["op"] in ("add", "mod"): state[k] = ev["tx"]
    elif ev["op"] == "del": state.pop(k)
    else:
        p = f"{work}/re-{ev['i']}.csv"
        with open(p, "w", newline="") as f:
            w = csv.writer(f); w.writerow(["e", "a", "v", "tx"]); [w.writerow([*kk, tx]) for kk, tx in sorted(state.items())]
        dolt(newrepo, "table", "import", "-r", "facts", p); dolt(newrepo, "add", "facts"); dolt(newrepo, "commit", "--allow-empty", "-m", f"facts at marvel {ev['commit']} i={ev['i']}")
        newh[ev["i"]] = q(newrepo, "SELECT HASHOF('HEAD') AS h")[0]["h"]
t2 = time.monotonic()
cnt = lambda r, h: q(r, f"SELECT count(*) AS n FROM facts AS OF '{h}'")[0]["n"]
counts_equal = sum(cnt(repo, r["dolt_hash"]) == cnt(newrepo, newh[r["i"]]) for r in loads)
hmap = {r["i"]: r["dolt_hash"] for r in loads}
def dd(r, a, b):
    x = q(r, f"SELECT diff_type, count(*) AS n FROM dolt_diff('{a}','{b}','facts') GROUP BY diff_type")
    d = {y["diff_type"]: y["n"] for y in x}; return (d.get("added", 0), d.get("removed", 0))
pairs = [l.split() for l in open(pairsf) if l.strip()]
pairs_equal = sum(dd(repo, hmap[int(p[0])], hmap[int(p[2])]) == dd(newrepo, newh[int(p[0])], newh[int(p[2])]) for p in pairs)
head = loads[-1]
rows = sorted((x["e"], x["a"], x["v"], x["tx"]) for x in q(repo, f"SELECT e,a,v,tx FROM facts AS OF '{head['dolt_hash']}' ORDER BY e,a,v"))
pick = sorted(random.Random(20261022).sample(rows, 100))
doc = {"kind": "harvested-facts", "source": {"repo": "ArcavenAE/marvel", "commit": head["commit"], "arm": "D", "seed": 20261022},
       "facts": [{"e": e, "a": a, "v": v, "tx": tx} for e, a, v, tx in pick]}
yaml.safe_dump(doc, open(f"{od}/d-harvest.yaml", "w"), sort_keys=False, allow_unicode=False, width=1000)
back = yaml.safe_load(open(f"{od}/d-harvest.yaml"))
rt = sorted((f["e"], f["a"], f["v"], f["tx"]) for f in back["facts"])
print(json.dumps({"export_s": round(t1-t0, 2), "import_s": round(t2-t1, 2), "export_bytes": os.path.getsize(f"{od}/d-export.jsonl"),
                  "commits": len(loads), "per_commit_counts_equal": counts_equal, "pairs": len(pairs), "pair_diffs_equal": pairs_equal,
                  "harvest_rows": len(pick), "yaml_round_trip_equal": rt == pick, "source_commit_back": back["source"]["commit"] == head["commit"]}))
