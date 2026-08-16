# finding-132: kos has no way to move, rename, or rescope a node without orphaning its references

**Date:** 2026-08-15
**Status:** OBSERVED (anticipatory). No relocation was performed; this records a
capability gap that the marvel-node placement audit made concrete. A finding
**for kos**: relocation is a kos-tool concern.
**Scope:** kos-tool id and reference integrity. Sibling of finding-131 (id
collision) and finding-133 (cross-graph awareness); the three are facets of one
federation problem.

## The gap

Nodes get misfiled, rescoped, redefined, and moved. It is not a hypothetical:
the placement audit this session found nodes whose home is arguable and will
likely change, and the whole point of the audit was that some belong elsewhere.
Relocation **will** happen, repeatedly, as the graph grows and as scope
understanding sharpens. kos has no primitive for it that preserves reference
integrity.

What kos supports today:

- **Moving a node between confidence tiers** (bedrock / frontier / graveyard) is
  a `git mv` within one graph. The id stays the same, so edges still resolve.
  This case is handled.

What kos does not support:

- **Renaming a node** (changing its id). Every `edges:` target, every charter
  reference, and every doc mention pointing at the old id becomes a dangling
  reference. Nothing updates them and nothing leaves a trail from the old id to
  the new one.
- **Moving a node to another graph** (orc to a subrepo, subrepo to orc, subrepo
  to subrepo). The node leaves its graph entirely; every in-graph reference to
  it is orphaned, and the destination graph has no record of where it came from.
  This is the case the placement audit will trigger.
- **Rescoping or redefining a node in place.** The id and location stay, but the
  meaning changes. References that were correct now point at something that
  means something different, with no signal that the target shifted under them.

## Why it matters

Every relocation without forwarding is silent reference rot. The reader who
follows an old edge, charter line, or doc link lands on nothing, or worse, lands
on a node that has quietly come to mean something else. At current scale this is
occasional. As graphs multiply and scope is refined, relocation becomes routine,
and routine silent rot is how a knowledge graph stops being trustworthy.

## What is needed (shape open, for study)

A relocation primitive that does one of two things, and probably both depending
on the case:

1. **Update references atomically.** On rename or move, find and rewrite every
   reference (edges, and ideally charter and doc mentions) to the new id or
   location. Requires kos to know its full reference set, which today it computes
   only within a graph.
2. **Leave a forwarding path.** A tombstone or redirect at the old id that points
   to the new location, so an old reference resolves through one hop instead of
   dangling. Followable, and (per finding-133) tolerantly resolvable when the
   destination is in an unreachable graph.

The primitive must distinguish **moved** (same concept, new home or new id, a
redirect is correct) from **redefined or superseded** (the concept changed, a
`supersedes` edge with an `error` or `evolution` signal is correct). kos has
`supersedes` today; it has no **moved-to** or **relocated** relationship, and the
two must not be conflated: a redirect says "the same thing lives here now," a
supersession says "this was replaced by a different thing."

## Prior art to study

- **HTTP 301 / 308** permanent redirect: the canonical "moved, follow this"
  pattern, and the model for a tombstone that forwards.
- **DOI and other persistent identifiers**: the identifier survives relocation of
  the resource; resolution is indirected through a registry. The strongest model
  for "the id never dies even when the thing moves."
- **git**: `mv` with rename detection, and refs as movable human names over
  stable objects.
- **filesystem symlinks** and **URL permalinks**: the cheap redirect.
- **Datomic** (kos's stated long-term substrate): entities keep a stable id while
  attributes and values change over time; history is the record. Relocation as an
  append, not a destructive move.

## Update 2026-08-15: a third shape, split-on-move

The fleet placement audit surfaced a case the two-way move/rename framing above
does not cover: a single node whose content serves TWO subjects.
`question-session-bootstrap` (orc) bundles forestage's own launch layering
(retired) with how marvel bootstraps an autonomous agent (live). Its correct
resolution is not a move but a **split**: divide the node, relocate the live
subject (to marvel), drop or graveyard the dead one (forestage, retired). kos
has no split primitive, so today this is a manual rewrite that loses the
original's identity and edges. The relocation primitive study should treat
split-on-move as a first-class shape beside rename and cross-graph move: one
source node becomes two (or more) target nodes, with forwarding from the old id
to the primary successor and a recorded derivation to the others.

The same audit also surfaced the reverse-reference need (an orc node that three
subrepo graphs must be able to point AT); that is a resolution concern and is
recorded in finding-133, not here.

## Recommended probe

Fold into the finding-131 id-scheme study, or run beside it: design the
relocation primitive (rename, cross-graph move, rescope) with a forwarding
record and a reference-update pass, and a clear split between moved and
superseded. Output a kos-side node and a finding.

## Cross-references

- finding-131 (id collision across graphs): a stable, collision-resistant id
  makes forwarding tractable; the two studies are joined.
- finding-133 (cross-graph awareness): cross-graph moves need tolerant
  resolution of the forwarding target.
- `question-cross-repo-knowledge` (F10), `question-charter-management` (charter
  references are one of the reference sets that rot on rename).
- kos schema: `kos/schema/node.schema.yaml` ("never reused" ids and the
  `supersedes` edge are the nearest existing pieces; neither covers move).

## bd tracking

- aae-orc-0ges9 (kos: no move/rename/split with forwarding; relocation
  primitive study). bd work-queue anchor; this finding is the authoritative
  record.
