"""C1 oracle for the SI-12 re-run: expected code/ facts per version, computed from
the index files with the probe's own decoder (scipdec.py), independent of the
kos ingest, compared with the store's intervals from `kos facts export`.

usage: python3 -I oracle.py <run dir> <export.jsonl>
"""
import json
import re
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from scipdec import fields, syminfo  # noqa: E402



def split_symbol(sym):
    # <scheme> <manager> <package> <version> <descriptors>; spaces inside fields are doubled
    parts = sym.split(" ", 4)
    if len(parts) < 5:
        return None
    return parts


def extract(path, repo):
    data = open(path, "rb").read()
    docs = []
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
        docs.append((doc, occs, syms))
    leaves = lambda d: d.startswith("/") or d == ".." or d.startswith("../")
    # own packages: those the index defines in documents under the root
    own = set()
    for doc, occs, syms in docs:
        if leaves(doc):
            continue
        for sym, roles in occs:
            if roles & 1 and not sym.startswith("local ") and sym:
                p = split_symbol(sym)
                if p:
                    own.add((p[1], p[2]))

    def norm(sym):
        p = split_symbol(sym)
        if p and (p[1], p[2]) in own:
            return " ".join([p[0], p[1], p[2], ".", p[4]])
        return sym

    out, deps = set(), set()

    def note(sym):
        p = split_symbol(sym)
        if p and (p[1], p[2]) not in own:
            deps.add(p[2])

    for doc, occs, syms in docs:
        if leaves(doc):
            continue
        for sym, roles in occs:
            if not sym or sym.startswith("local "):
                continue
            note(sym)
            out.add((f"{repo}:{doc}", "code/defines" if roles & 1 else "code/references", norm(sym)))
        for s in syms:
            for r in s["rels"]:
                if not r["impl"]:
                    continue
                if s["symbol"].startswith("local ") or r["s"].startswith("local "):
                    continue
                note(s["symbol"])
                note(r["s"])
                out.add((norm(s["symbol"]), "code/implements", norm(r["s"])))
    for d in deps:
        out.add((repo, "code/depends_on", d))
    return out




def main(RUN, EXPORT):
    expected = {}
    for n in range(1, 51):
        s = set()
        for repo in ("kos", "marvel"):
            s |= extract(f"{RUN}/idx-{repo}/{n}.scip", repo)
        expected[n] = s

    # the store's intervals
    iv = []
    for line in open(EXPORT, encoding="utf-8"):
        o = json.loads(line)
        if "e" in o and o["a"].startswith("code/"):
            iv.append((o["e"], o["a"], o["v"], o["valid_from"], o["valid_to"]))


    def state(n):
        return {(e, a, v) for e, a, v, f, t in iv if f <= n and (t is None or n < t)}


    bad_state, bad_diff = [], []
    prev = set()
    counts = {}
    for n in range(1, 51):
        st = state(n)
        counts[n] = len(st)
        if st != expected[n]:
            bad_state.append((n, len(st - expected[n]), len(expected[n] - st)))
        added = {(e, a, v) for e, a, v, f, t in iv if f == n}
        removed = {(e, a, v) for e, a, v, f, t in iv if t == n}
        if n > 1:
            if added != expected[n] - expected[n - 1] or removed != expected[n - 1] - expected[n]:
                bad_diff.append(n)
        else:
            if added != expected[1]:
                bad_diff.append(n)
    print("versions compared: 50")
    print("state mismatches (n, extra in store, missing in store):", bad_state)
    print("diff mismatches:", bad_diff)
    print("code/ facts at 1, 25, 50:", counts[1], counts[25], counts[50])
    json.dump({str(n): len(expected[n]) for n in expected}, open(f"{RUN}/logs/oracle-counts.json", "w"))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
