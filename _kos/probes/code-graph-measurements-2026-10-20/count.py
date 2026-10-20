"""Count candidate kos facts in a decoded SCIP index.

Usage: python3 -I count.py <decoded.json> [...]
The input is scipdec.py output. Documents whose path starts with "../" lie
outside the repository (scip-go emits files from the Go build cache, such as
generated test mains), and are counted separately from tracked files.

DEFINES     non-local definition occurrences
REFERENCES  distinct (document, non-local symbol) pairs among references
IMPLEMENTS  relationships with the implementation flag set
"""
import json
import sys


def count(d, keep):
    occ = {s: v for s, v in d["occ"].items() if not s.startswith("local ")}
    docs = [x for x in d["docs"] if keep(x["path"])]
    raw_defs = sum(x["defs"] for x in docs)
    raw_refs = sum(x["refs"] for x in docs)
    defines = sum(1 for v in occ.values() for o in v if o[2] & 1 and keep(o[0]))
    refs = {(o[0], s) for s, v in occ.items() for o in v if not o[2] & 1 and keep(o[0])}
    impl = sum(1 for x in d["symbols"].values() if keep(x["doc"]) for r in x["rels"] if r["impl"])
    return len(docs), raw_defs, raw_refs, defines, len(refs), impl


def main():
    for n in sys.argv[1:]:
        d = json.load(open(n))
        for label, keep in (("tracked", lambda p: not p.startswith("../")), ("all", lambda p: True)):
            docs, rd, rr, de, re_, im = count(d, keep)
            print(f"{n} {label}: documents={docs} raw_defs={rd} raw_refs={rr} "
                  f"DEFINES={de} REFERENCES={re_} IMPLEMENTS={im} total={de + re_ + im}")


if __name__ == "__main__":
    main()
