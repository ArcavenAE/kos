"""Write G3 query files and the outside truth. Usage: g3q.py <draws-dir> <facts-dir> <d-load.jsonl> <outdir>"""
import json, sys
dd, fdir, dload, od = sys.argv[1:5]
pairs = [l.split() for l in open(f"{dd}/g3-pairs.txt") if l.strip()]
hashes = {r["i"]: r["dolt_hash"] for r in map(json.loads, open(dload)) if "dolt_hash" in r}
load = lambda c: {(r["e"], r["a"], r["v"]) for r in map(json.loads, open(f"{fdir}/{c[:8]}.jsonl"))}
live = lambda n: f"SELECT e,a,v FROM facts WHERE valid_from<={n} AND (valid_to IS NULL OR valid_to>{n})"
truth, vt, d = [], [], []
for i, ci, j, cj in pairs:
    a, b = load(ci), load(cj)
    truth.append({"pair": f"{i}-{j}", "added": len(b - a), "removed": len(a - b)})
    vt.append(f"{i}-{j}\tSELECT (SELECT count(*) FROM ({live(j)} EXCEPT {live(i)})), (SELECT count(*) FROM ({live(i)} EXCEPT {live(j)}))")
    if int(i) in hashes and int(j) in hashes:
        d.append(f"{i}-{j}\tSELECT CAST(COALESCE(SUM(diff_type='added'),0) AS SIGNED), CAST(COALESCE(SUM(diff_type='removed'),0) AS SIGNED) FROM dolt_diff('{hashes[int(i)]}','{hashes[int(j)]}','facts')")
open(f"{od}/g3-truth.jsonl", "w").write("".join(json.dumps(t) + "\n" for t in truth))
open(f"{od}/g3-vt.tsv", "w").write("\n".join(vt) + "\n")
open(f"{od}/g3-d.tsv", "w").write("\n".join(d) + "\n")
print("pairs", len(pairs), "d queries", len(d))
