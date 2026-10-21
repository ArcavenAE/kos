"""Probe B criterion 3: cherry-pick 100 facts into their own file, export to YAML, round-trip.
Usage: python3 -I harvest_sqlite.py <db> <harvest.db> <out.yaml> <seed>"""
import json, os, random, sqlite3, sys, yaml
db, hdb, out, seed = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4])
con = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
rows = con.execute("SELECT e,a,v,tx FROM facts WHERE tx=2 ORDER BY e,a,v").fetchall()
pick = sorted(random.Random(seed).sample(rows, 100))
h = sqlite3.connect(hdb)
h.execute("CREATE TABLE facts (e TEXT NOT NULL, a TEXT NOT NULL, v TEXT NOT NULL, tx INTEGER NOT NULL)")
with h:
    h.executemany("INSERT INTO facts VALUES (?,?,?,?)", pick)
h.execute("VACUUM")
n = h.execute("SELECT count(*) FROM facts").fetchone()[0]
commit = con.execute("SELECT commit_id FROM txs WHERE tx=2").fetchone()[0]
doc = {"kind": "harvested-facts", "source": {"repo": "ArcavenAE/marvel", "commit": commit, "tx": 2, "seed": seed},
       "facts": [{"e": e, "a": a, "v": v, "tx": tx} for e, a, v, tx in pick]}
yaml.safe_dump(doc, open(out, "w"), sort_keys=False, allow_unicode=False, width=1000)
back = yaml.safe_load(open(out))
rt = sorted((f["e"], f["a"], f["v"], f["tx"]) for f in back["facts"])
hrows = sorted(h.execute("SELECT e,a,v,tx FROM facts").fetchall())
print("harvest rows", n, "harvest file bytes", os.path.getsize(hdb), "source bytes", os.path.getsize(db))
print("yaml bytes", os.path.getsize(out), "round-trip equal", rt == pick, "harvest db equal", hrows == pick,
      "source rows in harvest", n == len(rows))
print("by a", sorted({a: sum(1 for x in pick if x[1] == a) for a in {x[1] for x in pick}}.items()))
