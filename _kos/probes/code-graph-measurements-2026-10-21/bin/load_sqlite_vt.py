"""Like-for-like SQLite layout for review item 4: one row per fact with valid_from and valid_to
(NULL = still valid). Usage: load_sqlite_vt.py <db> <facts-c1.jsonl> <facts-c2.jsonl>"""
import json, os, sqlite3, sys, time
db, f1, f2 = sys.argv[1:4]
load = lambda p: {(r["e"], r["a"], r["v"]) for r in map(json.loads, open(p))}
t0 = time.monotonic(); a, b = load(f1), load(f2)
con = sqlite3.connect(db)
con.executescript("""
CREATE TABLE facts (e TEXT NOT NULL, a TEXT NOT NULL, v TEXT NOT NULL, valid_from INTEGER NOT NULL, valid_to INTEGER);
CREATE INDEX eavt ON facts (e, a, v, valid_from);
CREATE INDEX aevt ON facts (a, e, v, valid_from);
CREATE INDEX avet ON facts (a, v, e, valid_from);
CREATE INDEX vaet ON facts (v, a, e, valid_from);
""")
with con:
    con.executemany("INSERT INTO facts VALUES (?,?,?,1,?)", ((e, x, v, None if (e, x, v) in b else 2) for e, x, v in a))
    con.executemany("INSERT INTO facts VALUES (?,?,?,2,NULL)", (f for f in b - a))
con.execute("VACUUM")
t1 = time.monotonic()
q = lambda s: con.execute(s).fetchone()[0]
print("rows", q("SELECT count(*) FROM facts"), "bytes", os.path.getsize(db), "load_s %.2f" % (t1 - t0))
print("at tx1", q("SELECT count(*) FROM facts WHERE valid_from<=1 AND (valid_to IS NULL OR valid_to>1)"),
      "at tx2", q("SELECT count(*) FROM facts WHERE valid_from<=2 AND (valid_to IS NULL OR valid_to>2)"),
      "added", q("SELECT count(*) FROM facts WHERE valid_from=2"), "removed", q("SELECT count(*) FROM facts WHERE valid_to=2"))
