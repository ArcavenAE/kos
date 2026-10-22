"""Probe C arm S-snap: one full fact set per tx; stops once the store passes 2 GB.
Usage: python3 -I load_snap.py <db-dir> <index.jsonl> <facts-dir> <out.jsonl>"""
import json, os, sqlite3, subprocess, sys, time
ddir, idx, fdir, out = sys.argv[1:5]
os.makedirs(ddir, exist_ok=True)
con = sqlite3.connect(f"{ddir}/snap.sqlite", isolation_level=None)
con.execute("PRAGMA journal_mode=WAL"); con.execute("PRAGMA synchronous=NORMAL")
con.executescript("""
CREATE TABLE facts (e TEXT NOT NULL, a TEXT NOT NULL, v TEXT NOT NULL, tx INTEGER NOT NULL);
CREATE INDEX eavt ON facts (e, a, v, tx);
CREATE INDEX aevt ON facts (a, e, v, tx);
CREATE INDEX avet ON facts (a, v, e, tx);
CREATE INDEX vaet ON facts (v, a, e, tx);
CREATE TABLE txs (tx INTEGER PRIMARY KEY, commit_id TEXT NOT NULL);
""")
fo = open(out, "w"); n = 0
for row in (json.loads(l) for l in open(idx) if l.startswith('{"i"')):
    if row["status"] != "ok": continue
    i, c = row["i"], row["commit"]
    t0 = time.monotonic()
    new = [(r["e"], r["a"], r["v"], i) for r in map(json.loads, open(f"{fdir}/{c[:8]}.jsonl"))]
    con.execute("BEGIN"); con.executemany("INSERT INTO facts VALUES (?,?,?,?)", new)
    con.execute("INSERT INTO txs VALUES (?,?)", (i, c)); con.execute("COMMIT")
    t1 = time.monotonic(); n += 1
    kb = int(subprocess.run(["du", "-sk", ddir], capture_output=True, text=True).stdout.split()[0])
    fo.write(json.dumps({"i": i, "commit": c, "facts": len(new), "total_s": round(t1-t0, 3), "du_kb": kb}) + "\n"); fo.flush()
    if kb * 1024 > 2 * 1024**3:
        fo.write(json.dumps({"stopped": True, "commits_loaded": n, "at_i": i, "du_kb": kb}) + "\n"); break
con.execute("PRAGMA wal_checkpoint(TRUNCATE)"); con.close()
