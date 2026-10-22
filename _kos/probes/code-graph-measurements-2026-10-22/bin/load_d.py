"""Probe C arm D: one Dolt commit per marvel commit, via the dolt CLI as Probe A.
Usage: python3 -I load_d.py <dolt-repo> <index.jsonl> <facts-dir> <work> <out.jsonl>
tx = the first-seen marvel commit (sha8), kept for unchanged facts."""
import csv, io, json, subprocess, sys, time
repo, idx, fdir, work, out = sys.argv[1:6]
def dolt(*a, cap=False):
    r = subprocess.run(["dolt", *a], cwd=repo, capture_output=True)
    if r.returncode: raise RuntimeError(f"dolt {a[0]} rc={r.returncode}: " + r.stderr.decode(errors="replace").strip().splitlines()[-1])
    return r.stdout
fo = open(out, "w")
for row in (json.loads(l) for l in open(idx) if l.startswith('{"i"')):
    if row["status"] != "ok": continue
    i, c = row["i"], row["commit"]; s = c[:8]
    try:
        t0 = time.monotonic()
        new = {(r["e"], r["a"], r["v"]) for r in map(json.loads, open(f"{fdir}/{s}.jsonl"))}
        t1 = time.monotonic()
        prev = {}
        for r in csv.reader(io.StringIO(dolt("sql", "-r", "csv", "-q", "SELECT e,a,v,tx FROM facts", cap=True).decode())):
            if r and r != ["e", "a", "v", "tx"]: prev[(r[0], r[1], r[2])] = r[3]
        t2 = time.monotonic()
        p = f"{work}/load-{s}.csv"
        with open(p, "w", newline="") as fh:
            w = csv.writer(fh); w.writerow(["e", "a", "v", "tx"])
            for f in sorted(new): w.writerow([*f, prev.get(f, s)])
        dolt("table", "import", "-r", "facts", p); dolt("add", "facts"); dolt("commit", "--allow-empty", "-m", f"facts at marvel {c} i={i}")
        t3 = time.monotonic()
        h = dolt("sql", "-r", "csv", "-q", "SELECT HASHOF('HEAD'), count(*) FROM facts", cap=True).decode().split()[-1].split(",")
        fo.write(json.dumps({"i": i, "commit": c, "facts": len(new), "dolt_hash": h[0], "rows": int(h[1]),
                             "added": len(set(new) - set(prev)), "removed": len(set(prev) - set(new)),
                             "read_file_s": round(t1-t0, 3), "read_prev_s": round(t2-t1, 3), "import_commit_s": round(t3-t2, 3),
                             "total_s": round(t3-t0, 3)}) + "\n")
    except Exception as e:
        fo.write(json.dumps({"i": i, "commit": c, "status": "LOAD-ERROR", "err": str(e)[:300]}) + "\n")
    fo.flush()
