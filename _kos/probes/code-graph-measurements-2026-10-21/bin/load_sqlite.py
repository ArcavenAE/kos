"""Probe B loader: SCIP index -> append-only facts(e, a, v, tx) in one SQLite file.

Usage: python3 -I load_sqlite.py <db> <tx> <index.scip>
tx is an integer; the commit it stands for is kept in table txs.
Each tx holds the full fact set at that commit (a snapshot per tx), so the
diff between two commits is a set query over tx.
Times the whole step, SCIP bytes to committed rows, and prints it.
"""
import sqlite3
import sys
import time

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from facts import extract  # noqa: E402

SCHEMA = """
CREATE TABLE IF NOT EXISTS facts (e TEXT NOT NULL, a TEXT NOT NULL, v TEXT NOT NULL, tx INTEGER NOT NULL);
CREATE INDEX IF NOT EXISTS eavt ON facts (e, a, v, tx);
CREATE INDEX IF NOT EXISTS aevt ON facts (a, e, v, tx);
CREATE INDEX IF NOT EXISTS avet ON facts (a, v, e, tx);
CREATE INDEX IF NOT EXISTS vaet ON facts (v, a, e, tx);
CREATE TABLE IF NOT EXISTS txs (tx INTEGER PRIMARY KEY, commit_id TEXT NOT NULL, loaded_at TEXT NOT NULL);
"""


def main():
    db, tx, scip = sys.argv[1], int(sys.argv[2]), sys.argv[3]
    commit = scip.rsplit("-", 1)[1].split(".")[0]
    t0 = time.monotonic()
    facts = extract(scip)
    t1 = time.monotonic()
    con = sqlite3.connect(db)
    con.executescript(SCHEMA)
    with con:
        con.executemany("INSERT INTO facts VALUES (?, ?, ?, ?)", ((e, a, v, tx) for e, a, v in facts))
        con.execute("INSERT INTO txs VALUES (?, ?, strftime('%Y-%m-%dT%H:%M:%SZ','now'))", (tx, commit))
    t2 = time.monotonic()
    n = con.execute("SELECT count(*) FROM facts WHERE tx = ?", (tx,)).fetchone()[0]
    print(f"tx={tx} commit={commit} facts={n} extract_s={t1 - t0:.2f} insert_s={t2 - t1:.2f} total_s={t2 - t0:.2f}")


if __name__ == "__main__":
    main()
