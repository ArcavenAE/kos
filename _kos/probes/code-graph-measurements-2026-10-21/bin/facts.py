"""Extract candidate kos facts from a SCIP index, as (e, a, v) triples.

Fact model, from fact-count.md (kos 41fccf5), tracked documents only
(paths not starting with "../"), local symbols left out:
  (doc,  "DEFINES",    sym)   a non-local definition occurrence in doc
  (doc,  "REFERENCES", sym)   distinct (doc, sym) among non-local references
  (sym,  "IMPLEMENTS", tsym)  a SCIP relationship with the implementation flag

One normalization, recorded in the finding: the marvel module's own version
token in each symbol (scip-go writes --module-version into every symbol) is
replaced by ".", so a symbol keeps its identity across commits. Dependency
versions are kept; a dependency bump is a real change of fact.

Usage as a script: python3 -I facts.py <index.scip> <out.jsonl>
"""
import json
import re
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from scipdec import fields, packed, syminfo  # noqa: E402

OWN = re.compile(r"^(scip-go gomod github\.com/arcavenae/marvel) \S+ ")


def norm(s):
    return OWN.sub(r"\1 . ", s)


def extract(path):
    data = open(path, "rb").read()
    out = set()
    for f, t, v in fields(data):
        if f != 2:
            continue
        doc, occs, syms = "", [], []
        for g, tt, w in fields(v):
            if g == 1:
                doc = w.decode()
            elif g == 2:
                sym, roles = "", 0
                for h, t3, x in fields(w):
                    if h == 2:
                        sym = x.decode()
                    elif h == 3:
                        roles = x
                occs.append((sym, roles))
            elif g == 3:
                syms.append(syminfo(w))
        if doc.startswith("../"):
            continue
        for sym, roles in occs:
            if not sym or sym.startswith("local "):
                continue
            out.add((doc, "DEFINES" if roles & 1 else "REFERENCES", norm(sym)))
        for s in syms:
            for r in s["rels"]:
                if r["impl"]:
                    out.add((norm(s["symbol"]), "IMPLEMENTS", norm(r["s"])))
    return out


def main():
    facts = extract(sys.argv[1])
    with open(sys.argv[2], "w") as fh:
        for e, a, v in sorted(facts):
            fh.write(json.dumps({"e": e, "a": a, "v": v}) + "\n")
    print(len(facts))


if __name__ == "__main__":
    main()
