import json, sys, collections
def load(p): return {(r["e"], r["a"], r["v"]) for r in map(json.loads, open(p))}
a, b = load(sys.argv[1]), load(sys.argv[2])
add, rem = b - a, a - b
print("outside: added", len(add), "removed", len(rem))
print("added_by_a", sorted(collections.Counter(x[1] for x in add).items()))
print("removed_by_a", sorted(collections.Counter(x[1] for x in rem).items()))
