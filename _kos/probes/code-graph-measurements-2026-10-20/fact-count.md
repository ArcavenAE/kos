# Fact count: candidate kos facts per commit, 2026-10-20

The measurement aae-orc-isie lacked. Counts are derived from SCIP indexes
of marvel ac67c078 and kos 829c392, built on 2026-10-16 and not published.
The indexing cost was measured earlier, on 2026-08-16
(finding-kos-ndz7-scip-indexing-cost and the code-graph-measurements-2026-08-16
directory).

A "candidate kos fact" is one of:
- DEFINES: one per non-local definition occurrence;
- REFERENCES: one per distinct (document, symbol) pair among non-local
  references, so repeated references inside one file count once;
- IMPLEMENTS: one per SCIP implementation relationship.

Symbols local to a function are left out.

| repo, commit | documents | raw occurrences (defs / refs) | DEFINES | REFERENCES (dedup) | IMPLEMENTS | candidate facts |
|---|---:|---:|---:|---:|---:|---:|
| marvel ac67c078, tracked files (Go, 144k lines incl. tests) | 650 | 42,854 / 216,444 | 9,796 | 33,176 | 312 | **43,284** |
| marvel ac67c078, as emitted (adds 49 build-cache documents) | 699 | 43,246 / 223,360 | 10,139 | 36,855 | 312 | 47,306 |
| kos 829c392 (Rust, 11k lines) | 30 | 3,234 / 17,568 | 929 | 3,455 | 0 | **4,384** |

## Documents against files

marvel has 659 tracked `.go` files at ac67c078. The index holds 699
documents:
- 650 are tracked `.go` files.
- 49 lie outside the repository, under the Go build cache (`GOCACHE`). They
  are files the Go toolchain generates for test binaries (the `.test`
  packages' mains). They carry 343 definitions and no implementations.
- The 9 tracked `.go` files not indexed are excluded by build constraints on
  the darwin host: 8 `_linux` or `_other` variants in `internal/childproof`,
  `internal/procargs`, `internal/procstat` and `internal/session`, and one
  `//go:build ignore` probe file under `_kos/probes/`.

650 + 9 = 659 and 650 + 49 = 699. The tracked-file row is the count to use;
the as-emitted row is kept because the earlier draft reported it.

kos has 30 tracked `.rs` files at 829c392, and the index holds those 30.

## How to reproduce

The indexes are not published (22 MB for marvel, and scip-go output is not
byte-reproducible), so the count cannot be checked against public material
without re-indexing. Re-indexing is offline and takes seconds. The counts do
not depend on byte order: two warm marvel runs gave different bytes and
identical counts.

Indexers: scip-go 0.2.7 and rust-analyzer 1.97.1 (with rust-src), run in
clean clones at the stated commits with no network:

```sh
# marvel, at ac67c078
GOPROXY=off GOFLAGS=-mod=readonly GOTOOLCHAIN=local GOCACHE=<scratch>/gocache \
  scip-go index --repository-remote=github.com/ArcavenAE/marvel \
  --module-version=ac67c078 -o marvel.scip
# kos, at 829c392
CARGO_NET_OFFLINE=true rust-analyzer scip .
```

Counting, with the two scripts beside this file:

```sh
python3 -I scipdec.py marvel.scip marvel.json
python3 -I count.py marvel.json
```

`scipdec.py` is a dependency-free protobuf decoder for the SCIP fields the
count reads. `count.py` prints a tracked row (documents whose path does not
start with `../`) and an as-emitted row.

## What it means

- One commit of one mid-size repo is about 43k facts. That is two orders of
  magnitude past the flat-file ceiling kos already ruled
  (grv-flat-file-context-scaling: about 20 epics, a few hundred nodes).
- Kept per commit, it grows with history unless facts are stored as diffs.
- The isie ceiling argument, which was an assertion, now holds as a
  measurement for marvel alone, before any other fleet repo is counted.
- isie's own estimate (10^5 to 10^6 facts) is the right order for the
  fleet at one commit: marvel plus kos is about 48k, and the fleet had 25-plus
  indexed workspaces on 08-16. This is INFERRED; the other repos were not
  counted.

## Limits

- rust-analyzer emits no implementation relationships for kos, so 0 there
  means not reported, not none.
- DEPENDS_ON (package-level) was not derived. It is small next to
  REFERENCES.
- The marvel index is of the darwin build. A linux index would swap in the
  8 build-constrained files.
