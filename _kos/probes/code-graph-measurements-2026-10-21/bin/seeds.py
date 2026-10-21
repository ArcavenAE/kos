"""Pick 100 seed symbols from DEFINES at one commit, seeded. Usage: seeds.py <facts.jsonl> <seed> <out>"""
import json, random, sys
defs = sorted({r["v"] for r in map(json.loads, open(sys.argv[1])) if r["a"] == "DEFINES"})
pick = random.Random(int(sys.argv[2])).sample(defs, 100)
open(sys.argv[3], "w").write("\n".join(pick) + "\n")
print("distinct DEFINES symbols", len(defs), "picked", len(pick))
