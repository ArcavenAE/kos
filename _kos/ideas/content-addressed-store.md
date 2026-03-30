# Content-Addressed Knowledge Store

*Idea — pre-hypothesis brainstorming. Contradictory, uncommitted, generative.*

What if kos's storage substrate moved beyond YAML-in-git to a purpose-built
content-addressed store? Git validates the transport properties (content-addressing,
immutability, cryptographic integrity, distributed sync) but imposes constraints
that limit what kos can do: single-repo boundaries, file-shaped content,
human-paced workflows, no typed edges, no reverse indexes, no graph-native queries.

## Core primitives

**Atom** — content-addressed immutable fact. `sha256(canonical_bytes) → bytes`.
Typed by schema. Has provenance (who/what created it, confidence).

**Edge** — typed directed relationship. `(source_atom, relation, target_atom)`.
Also content-addressed: `sha256(src + rel + tgt) → metadata`. Immutable.

**Frame** — snapshot of a subgraph at time T. Analogous to git commit.
Points to parent frame(s) (DAG of DAGs). Contains an edge set + metadata.

**Lens** — named mutable pointer to a frame. Analogous to git ref/branch.
Examples: `lens:agent/alpha`, `lens:domain/security`, `lens:epoch/sprint-42`.

**Repo** — a namespace boundary. Contains atoms, edges, frames, lenses.
Can declare foreign atom references for cross-repo traversal.

## What this optimizes for that git doesn't

- Graph-native queries — "all facts connected to X within N hops"
- Cross-repo traversal — facts in one repo reference facts in another without copying
- Typed edges as first-class citizens — not hacked into filenames
- Ripple detection — when a fact changes, O(affected subgraph) not O(total graph)
- Multi-agent concurrent writes — CRDTs on edge sets, not merge-after-the-fact
- Temporal range queries — "what did we know about X between dates" without walking commits

## Cross-repo federation

Content-addressed global namespace: same fact in two repos = same hash.
A cross-repo edge references a hash that happens to live elsewhere.
Resolution is lazy — don't need foreign atom content until traversal.

```
enum AtomRef {
    Local(Hash),
    Foreign { repo: RepoId, hash: Hash },
    Unresolved(Hash),
}
```

Repo manifest declares peers for foreign ref resolution.

## Storage engine sketch

```
_kos/
  atoms/          # content-addressed store (packed + loose)
  edges/          # edge objects (also content-addressed)
  frames/         # snapshots (DAG of DAGs)
  lenses/         # named pointers (mutable)
  index/          # query acceleration (redb — pure Rust, ACID, zero-copy)
    by_type.redb
    by_edge.redb
    temporal.redb
    fts.redb
  manifest.toml
```

## Multi-agent concurrency

CRDTs on edge sets instead of git's optimistic-lock-then-merge:
- Adding a fact → add-only, always converges
- Adding an edge → add-only, always converges
- Retracting an edge → tombstone, LWW or vector clocks
- Contradictions → explicit edge type, surfaced not silently merged

## Build phases (if this became a probe)

1. `kos-core` — Atom/Edge/Hash types, serialization, content addressing (~500 lines)
2. `kos-store` — loose object read/write, frame creation, lens management (~1500 lines)
3. `kos-index` — redb-backed query indexes, reverse adjacency, temporal (~1000 lines)
4. `kos-cli` — plumbing commands
5. `kos-federation` — cross-repo atom resolution, peer manifest
6. `kos-ripple` — change propagation on the index layer

Phases 1-4 are single-repo local. Already more useful than git-as-knowledge-store
because queries are native. Federation + ripple = the thing that doesn't exist yet.

## Tensions and open questions

- Is this premature? YAML-in-git works at ~100 artifacts. When does it break?
- Building a storage engine is a massive undertaking. Is the existing Rust ecosystem
  (redb, sled, petgraph) sufficient to assemble from parts?
- Does this conflict with G5 (git as sufficient long-term substrate, ruled out as
  sufficient but retained as current substrate)?
- Could YAML-in-git remain the authoring format while a derived store provides
  the query/traversal/ripple layer? Dual representation: human-readable source,
  machine-optimized index.
