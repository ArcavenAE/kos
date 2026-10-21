"""Minimal SCIP index decoder (protobuf wire format, no dependencies).

Usage: python3 -I scipdec.py <index.scip> <out.json>
Writes {docs: [{path, defs, refs}], symbols: {sym: {kind, rels:[{s,impl,ref,typedef}]}},
        ext: [sym], occ_symbols: {sym: [[path, line, roles]]}}
"""
import json
import sys


def varint(b, i):
    r = s = 0
    while True:
        c = b[i]
        i += 1
        r |= (c & 0x7F) << s
        s += 7
        if c < 0x80:
            return r, i


def fields(b):
    i, n = 0, len(b)
    while i < n:
        k, i = varint(b, i)
        f, t = k >> 3, k & 7
        if t == 0:
            v, i = varint(b, i)
        elif t == 2:
            ln, i = varint(b, i)
            v = b[i:i + ln]
            i += ln
        elif t == 1:
            v = b[i:i + 8]
            i += 8
        elif t == 5:
            v = b[i:i + 4]
            i += 4
        else:
            raise ValueError(f"wire type {t}")
        yield f, t, v


def packed(v):
    out, i = [], 0
    while i < len(v):
        x, i = varint(v, i)
        out.append(x)
    return out


def rel(b):
    r = {"s": "", "ref": 0, "impl": 0, "typedef": 0, "def": 0}
    for f, t, v in fields(b):
        if f == 1:
            r["s"] = v.decode()
        elif f == 2:
            r["ref"] = v
        elif f == 3:
            r["impl"] = v
        elif f == 4:
            r["typedef"] = v
        elif f == 5:
            r["def"] = v
    return r


def syminfo(b):
    s = {"symbol": "", "kind": 0, "rels": []}
    for f, t, v in fields(b):
        if f == 1:
            s["symbol"] = v.decode()
        elif f == 4:
            s["rels"].append(rel(v))
        elif f == 5:
            s["kind"] = v
    return s


def main():
    data = open(sys.argv[1], "rb").read()
    out = {"docs": [], "symbols": {}, "ext": [], "occ": {}}
    for f, t, v in fields(data):
        if f == 2:
            path, defs, refs = "", 0, 0
            occs, syms = [], []
            for g, tt, w in fields(v):
                if g == 1:
                    path = w.decode()
                elif g == 2:
                    o = {"range": [], "sym": "", "roles": 0}
                    for h, t3, x in fields(w):
                        if h == 1:
                            o["range"] = packed(x) if t3 == 2 else [x]
                        elif h == 2:
                            o["sym"] = x.decode()
                        elif h == 3:
                            o["roles"] = x
                    occs.append(o)
                elif g == 3:
                    syms.append(syminfo(w))
            for o in occs:
                if o["roles"] & 1:
                    defs += 1
                else:
                    refs += 1
                line = o["range"][0] + 1 if o["range"] else 0
                out["occ"].setdefault(o["sym"], []).append([path, line, o["roles"]])
            for s in syms:
                out["symbols"][s["symbol"]] = {"kind": s["kind"], "rels": s["rels"], "doc": path}
            out["docs"].append({"path": path, "defs": defs, "refs": refs})
        elif f == 3:
            out["ext"].append(syminfo(v)["symbol"])
    json.dump(out, open(sys.argv[2], "w"))


if __name__ == "__main__":
    main()
