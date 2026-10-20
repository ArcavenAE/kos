# Fact count: candidate kos facts per commit, 2026-10-20

The measurement aae-orc-isie lacked. Counts are derived from the 2026-08-16 SCIP indexes; see the code-graph-measurements-2026-08-16 directory for the indexing cost.

The counts come from the 10-16 indexes. A "candidate kos fact" is one of:
- DEFINES: one per non-local definition occurrence;
- REFERENCES: one per distinct (document, symbol) pair among non-local
  references, so repeated references inside one file count once;
- IMPLEMENTS: one per SCIP implementation relationship.

Symbols local to a function are left out.

| repo, commit | documents | raw occurrences (defs / refs) | DEFINES | REFERENCES (dedup) | IMPLEMENTS | candidate facts |
|---|---:|---:|---:|---:|---:|---:|
| marvel ac67c078 (Go, 144k lines incl. tests) | 699 | 43,246 / 223,360 | 10,139 | 36,855 | 312 | **47,306** |
| kos 829c392 (Rust, 11k lines) | 30 | 3,234 / 17,568 | 929 | 3,455 | 0 | **4,384** |

What it means:
- One commit of one mid-size repo is about 47k facts. That is two orders of
  magnitude past the flat-file ceiling kos already ruled
  (grv-flat-file-context-scaling: about 20 epics, a few hundred nodes).
- Kept per commit, it grows with history unless facts are stored as diffs.
- The isie ceiling argument, which was an assertion, now holds as a
  measurement for marvel alone, before any other fleet repo is counted.
- isie's own estimate (10^5 to 10^6 facts) is the right order for the
  fleet at one commit: marvel plus kos is about 52k, and the fleet had 25-plus
  indexed workspaces on 08-16. This is INFERRED; the other repos were not
  counted today.

Limits:
- rust-analyzer emits no implementation relationships for kos, so 0 there
  means not reported, not none.
- DEPENDS_ON (package-level) was not derived. It is small next to
  REFERENCES.
- scip-go output is not byte-reproducible (10-16), but the counts do not
  depend on byte order.
