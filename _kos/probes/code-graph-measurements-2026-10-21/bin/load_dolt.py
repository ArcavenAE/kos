"""Probe A loader: SCIP index -> facts(e, a, v, tx) in a scratch Dolt repository, one Dolt commit per marvel commit.

Usage: python3 -I load_dolt.py <dolt-repo-dir> <index.scip> <workdir>
The table holds the fact set at the latest loaded commit. tx is the marvel
commit at which the fact was first asserted (kept for unchanged facts, so
Dolt's row diff between two Dolt commits is exactly the facts added and
removed). Times SCIP bytes to a committed Dolt commit, end to end.
"""
import csv
import io
import subprocess
import sys
import time

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from facts import extract  # noqa: E402


def dolt(repo, *args, capture=False):
    r = subprocess.run(["dolt", *args], cwd=repo, capture_output=True)
    if r.returncode != 0:
        sys.stderr.write(r.stderr.decode(errors="replace"))
        raise SystemExit(f"dolt {args[0]} rc={r.returncode}")
    return r.stdout if capture else None


def main():
    repo, scip, work = sys.argv[1], sys.argv[2], sys.argv[3]
    commit = scip.rsplit("-", 1)[1].split(".")[0]
    t0 = time.monotonic()
    facts = extract(scip)
    t1 = time.monotonic()
    prev = {}
    out = dolt(repo, "sql", "-r", "csv", "-q", "SELECT e, a, v, tx FROM facts", capture=True)
    for row in csv.reader(io.StringIO(out.decode())):
        if row and row != ["e", "a", "v", "tx"]:
            prev[(row[0], row[1], row[2])] = row[3]
    t2 = time.monotonic()
    path = f"{work}/load-{commit}.csv"
    with open(path, "w", newline="") as fh:
        w = csv.writer(fh)
        w.writerow(["e", "a", "v", "tx"])
        for f in sorted(facts):
            w.writerow([*f, prev.get(f, commit)])
    t3 = time.monotonic()
    dolt(repo, "table", "import", "-r", "facts", path)
    dolt(repo, "add", "facts")
    dolt(repo, "commit", "-m", f"facts at marvel {commit}")
    t4 = time.monotonic()
    head = dolt(repo, "sql", "-r", "csv", "-q", "SELECT count(*), sum(tx = '%s') FROM facts" % commit, capture=True)
    print(f"commit={commit} facts={len(facts)} kept_from_prev={len(set(prev) & facts)} "
          f"extract_s={t1 - t0:.2f} read_prev_s={t2 - t1:.2f} csv_s={t3 - t2:.2f} import_commit_s={t4 - t3:.2f} "
          f"total_s={t4 - t0:.2f} head={head.decode().split()[-1]}")


if __name__ == "__main__":
    main()
