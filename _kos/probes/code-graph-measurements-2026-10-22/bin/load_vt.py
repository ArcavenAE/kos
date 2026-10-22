"""Probe C arm S-vt: incremental valid-time load, one commit at a time.
Usage: python3 -I load_vt.py <db> <index.jsonl> <facts-dir> <out.jsonl>
Reads the ok commits in order; tx = the commit's position (1..200)."""
import json, sqlite3, sys, time
db, idx, fdir, out = sys.argv[1:5]
con = sqlite3.connect(db, isolation_level=None)
con.execute("PRAGMA journal_mode=WAL"); con.execute("PRAGMA synchronous=NORMAL")
con.executescript("""
CREATE TABLE facts (e TEXT NOT NULL, a TEXT NOT NULL, v TEXT NOT NULL, valid_from INTEGER NOT NULL, valid_to INTEGER);
CREATE INDEX eavt ON facts (e, a, v, valid_from, valid_to);
CREATE INDEX aevt ON facts (a, e, v, valid_from, valid_to);
CREATE INDEX avet ON facts (a, v, e, valid_from, valid_to);
CREATE INDEX vaet ON facts (v, a, e, valid_from, valid_to);
CREATE TABLE txs (tx INTEGER PRIMARY KEY, commit_id TEXT NOT NULL);
""")
fo = open(out, "w")
for row in (json.loads(l) for l in open(idx) if l.startswith('{"i"')):
    if row["status"] != "ok": continue
    i, c = row["i"], row["commit"]
    t0 = time.monotonic()
    new = {(r["e"], r["a"], r["v"]) for r in map(json.loads, open(f"{fdir}/{c[:8]}.jsonl"))}
    t1 = time.monotonic()
    live = set(con.execute("SELECT e,a,v FROM facts WHERE valid_to IS NULL"))
    t2 = time.monotonic()
    rem, add = live - new, new - live
    con.execute("BEGIN")
    con.executemany("UPDATE facts SET valid_to=? WHERE e=? AND a=? AND v=? AND valid_to IS NULL", ((i, *f) for f in rem))
    con.executemany("INSERT INTO facts VALUES (?,?,?,?,NULL)", ((*f, i) for f in add))
    con.execute("INSERT INTO txs VALUES (?,?)", (i, c))
    con.execute("COMMIT")
    t3 = time.monotonic()
    fo.write(json.dumps({"i": i, "commit": c, "facts": len(new), "added": len(add), "removed": len(rem),
                         "read_file_s": round(t1-t0, 3), "read_live_s": round(t2-t1, 3), "write_commit_s": round(t3-t2, 3),
                         "total_s": round(t3-t0, 3)}) + "\n"); fo.flush()
con.execute("PRAGMA wal_checkpoint(TRUNCATE)"); con.close()
