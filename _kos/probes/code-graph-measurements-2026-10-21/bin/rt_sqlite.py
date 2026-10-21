"""MDE lock-in: export the whole SQLite fact log to JSONL, re-import into a fresh file, compare.
Usage: rt_sqlite.py <db> <out.jsonl> <new.db>"""
import json, os, sqlite3, sys, time
db, out, new = sys.argv[1:4]
src = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
t0 = time.monotonic()
with open(out, "w") as fh:
    for e, a, v, tx in src.execute("SELECT e,a,v,tx FROM facts ORDER BY tx,e,a,v"):
        fh.write(json.dumps({"e": e, "a": a, "v": v, "tx": tx}) + "\n")
t1 = time.monotonic()
dst = sqlite3.connect(new)
dst.execute("CREATE TABLE facts (e TEXT NOT NULL, a TEXT NOT NULL, v TEXT NOT NULL, tx INTEGER NOT NULL)")
with dst:
    dst.executemany("INSERT INTO facts VALUES (?,?,?,?)", ((r["e"], r["a"], r["v"], r["tx"]) for r in map(json.loads, open(out))))
t2 = time.monotonic()
q = "SELECT tx, count(*) FROM facts GROUP BY tx ORDER BY tx"
print("counts src", src.execute(q).fetchall(), "dst", dst.execute(q).fetchall())
s1 = set(src.execute("SELECT e,a,v,tx FROM facts")); s2 = set(dst.execute("SELECT e,a,v,tx FROM facts"))
print("set equal", s1 == s2, "export_s %.2f import_s %.2f jsonl_bytes %d" % (t1 - t0, t2 - t1, os.path.getsize(out)))
