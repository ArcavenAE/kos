"""S-vt lock-in and round-trip. Usage: mde_vt.py <db> <index.jsonl> <g3-pairs.txt> <outdir>"""
import json, os, random, sqlite3, sys, time, yaml
db, idx, pairsf, od = sys.argv[1:5]
src = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
ok = [r["i"] for r in (json.loads(l) for l in open(idx) if l.startswith('{"i"')) if r["status"] == "ok"]
t0 = time.monotonic()
with open(f"{od}/vt-export.jsonl", "w") as fh:
    for e, a, v, f, t in src.execute("SELECT e,a,v,valid_from,valid_to FROM facts ORDER BY valid_from,e,a,v"):
        fh.write(json.dumps({"e": e, "a": a, "v": v, "valid_from": f, "valid_to": t}) + "\n")
    for tx, c in src.execute("SELECT tx, commit_id FROM txs ORDER BY tx"):
        fh.write(json.dumps({"tx": tx, "commit": c}) + "\n")
t1 = time.monotonic()
new = f"{od}/vt-reimport.sqlite"
dst = sqlite3.connect(new, isolation_level=None)
dst.execute("PRAGMA journal_mode=WAL"); dst.execute("PRAGMA synchronous=NORMAL")
dst.executescript(open(f"{od}/vt-schema.sql").read())
dst.execute("BEGIN")
for r in map(json.loads, open(f"{od}/vt-export.jsonl")):
    if "tx" in r: dst.execute("INSERT INTO txs VALUES (?,?)", (r["tx"], r["commit"]))
    else: dst.execute("INSERT INTO facts VALUES (?,?,?,?,?)", (r["e"], r["a"], r["v"], r["valid_from"], r["valid_to"]))
dst.execute("COMMIT"); dst.execute("PRAGMA wal_checkpoint(TRUNCATE)")
t2 = time.monotonic()
live = lambda n: f"SELECT e,a,v FROM facts WHERE valid_from<={n} AND (valid_to IS NULL OR valid_to>{n})"
cnt = lambda c, n: c.execute(f"SELECT count(*) FROM ({live(n)})").fetchone()[0]
counts_equal = sum(cnt(src, n) == cnt(dst, n) for n in ok)
diff = lambda c, i, j: (c.execute(f"SELECT count(*) FROM ({live(j)} EXCEPT {live(i)})").fetchone()[0], c.execute(f"SELECT count(*) FROM ({live(i)} EXCEPT {live(j)})").fetchone()[0])
pairs = [l.split() for l in open(pairsf) if l.strip()]
pairs_equal = sum(diff(src, p[0], p[2]) == diff(dst, p[0], p[2]) for p in pairs)
rows = src.execute(f"SELECT e,a,v,valid_from FROM facts WHERE valid_to IS NULL ORDER BY e,a,v").fetchall()
pick = sorted(random.Random(20261022).sample(rows, 100))
commit = src.execute("SELECT commit_id FROM txs ORDER BY tx DESC LIMIT 1").fetchone()[0]
doc = {"kind": "harvested-facts", "source": {"repo": "ArcavenAE/marvel", "commit": commit, "arm": "S-vt", "seed": 20261022},
       "facts": [{"e": e, "a": a, "v": v, "valid_from": f} for e, a, v, f in pick]}
yaml.safe_dump(doc, open(f"{od}/vt-harvest.yaml", "w"), sort_keys=False, allow_unicode=False, width=1000)
back = yaml.safe_load(open(f"{od}/vt-harvest.yaml"))
rt = sorted((f["e"], f["a"], f["v"], f["valid_from"]) for f in back["facts"])
print(json.dumps({"export_s": round(t1-t0, 2), "import_s": round(t2-t1, 2), "export_bytes": os.path.getsize(f"{od}/vt-export.jsonl"),
                  "commits": len(ok), "per_commit_counts_equal": counts_equal, "pairs": len(pairs), "pair_diffs_equal": pairs_equal,
                  "harvest_rows": len(pick), "yaml_round_trip_equal": rt == pick, "source_commit_back": back["source"]["commit"] == commit}))
