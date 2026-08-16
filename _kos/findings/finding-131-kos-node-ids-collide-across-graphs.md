# finding-131: kos node ids collide across graphs, and the readable-slug scheme has no defense against it

**Date:** 2026-08-15
**Status:** OBSERVED. A specimen surfaced during a placement audit of the
marvel-related nodes. No change applied to either graph (a marvel session may
be live); this file records the problem and sets up the study.
**Scope:** kos-tool id design. The observation is cross-repo (the orc graph
against the marvel graph), so the finding lives at the composition layer per
the orc rule for cross-repo findings, but it is a finding **for kos**: kos
assigns and resolves node ids, and kos is where the fix belongs. When kos next
takes up id scoping, mirror or reference this from `kos/_kos`.

## What was observed

kos node ids are human-readable slugs (`question-permission-model`,
`val-uncertainty-first`). A slug is unique **within one graph**. It is not
unique **across graphs**, and nothing in the scheme or the tooling enforces or
even detects cross-graph uniqueness.

Three ids exist in both `aae-orc/_kos` and `marvel/_kos` right now, all in the
frontier tier:

| id | orc title | marvel title |
|---|---|---|
| `question-multi-host` | Multi-Host Scheduling | Multi-Host Scheduling via Switchboard |
| `question-permission-model` | Permission Model: Environment Construction + Internal Capabilities | What is the permission model: environment construction + internal RPC capabilities? |
| `question-stream-attachment` | Stream Attachment Strategy | Which stream-attachment strategy does each runtime adapter use? |

These are not accidental clashes of unrelated concepts. Each pair is the **same
question at two altitudes**: the orc holds the composition-layer framing, marvel
holds the implementation framing. That is the intended summary-and-detail split
(orc charter scope rule, session-016). The problem is that the split is
expressed by **reusing one id in two graphs**, and there is no link, edge, or
marker saying "these are the orc view and the marvel view of one question."

## Why it is a defect, not a curiosity

- **Validation is blind to it.** `kos validate` runs per graph. Running it in
  the orc validated 100 nodes and never saw marvel; running it in marvel
  validated 83 and never saw the orc. Neither run can report that two copies of
  `question-permission-model` exist, or that they have drifted.
- **Edge references are ambiguous across graphs.** An `edges:` target of
  `question-permission-model` resolves to a **different node** depending on
  which graph the reader is standing in. Cross-graph edges (which the platform
  will want as knowledge federates) have no unambiguous target.
- **Drift is silent and bidirectional.** The two copies can diverge with no
  signal. The titles above already differ; content divergence would be
  invisible until a human happened to read both.

## Why this will get worse (the reason to solve it now)

There are 16-plus subrepos, and the direction is for more of them to grow their
own `_kos` graph (marvel, kos, sidestep, and others already have one). Slugs are
allocated independently per graph, from the same small vocabulary of obvious
words (`question-permission-model`, `question-multi-host`, `elem-runtime-*`).
Independent allocation from a shared vocabulary guarantees collisions, and the
collision rate rises with the number of graphs. The scheme that reads best at
one graph fails hardest at many.

## The design tension, and prior art to study

The tension is **readability against collision-resistance**, and other systems
have already paid for both horns:

- **bd (beads)** solved collision with a project prefix plus a short
  collision-resistant suffix (`aae-orc-dqhf`, `aae-orc-b56ja`). It buys global
  uniqueness at the cost of readability: `aae-orc-dqhf` tells a human nothing
  about what the issue is without a lookup. This is the "difficult for humans to
  identify references" cost. bd accepts it because its ids are handles, not
  descriptions. Study: exactly how bd generates the suffix (length, alphabet,
  collision handling), how the prefix namespaces, and how it stays stable across
  the Dolt server.
- **git** solved it with a **two-layer** model: a content-addressed SHA is the
  canonical, collision-resistant, opaque identifier, and humans work through
  refs (branch and tag names) that point at SHAs. Readability and uniqueness are
  separated into two layers rather than traded off in one string. This is the
  closest match to the combine-hypothesis below. Study: how refs resolve to
  objects, how ambiguous short SHAs are handled, how tags provide stable human
  names.
- **Other projects worth a look:** kubernetes namespaced names
  (`namespace/name`, uniqueness scoped by namespace, readable and unique
  together); ULID and UUIDv7 (sortable, unique, opaque); Sqids and nanoid
  (short, url-safe, tunable collision resistance); Nix store paths
  (hash-plus-human-name in one path, `hash-name`); DOI and ISBN (registered
  namespaced identifiers for a federated corpus). Each is a different point on
  the readable-versus-unique curve.

## Hypothesis to test

Do not replace the slug. **Combine the two layers, the way git does.** Keep the
human-readable slug as the reading and authoring surface, and issue a
collision-resistant canonical id alongside it. Candidate shapes to evaluate:

1. **Namespace prefix** (k8s-style): the graph name scopes the slug, so
   `marvel:question-permission-model` and `aae-orc:question-permission-model`
   are distinct canonical ids while both keep their readable slug. Cheapest;
   readable; makes cross-graph edges unambiguous.
2. **Two-layer with an opaque canonical id** (git-style): a short
   collision-resistant id (bd-style suffix or a short hash) is the canonical
   identity; the slug is a human alias that resolves to it. Most robust; adds a
   resolution step.
3. **Hybrid** (Nix-style): `slug-<shortid>` in one string, readable and unique
   at once, at the cost of longer ids.

A related need surfaces from the specimen: the three collisions want an
**explicit cross-graph relationship** (orc-summary-of / marvel-detail-of), not
only a unique id. Collision-resistant ids and cross-graph linking are two parts
of the same federation problem and should be studied together.

## Recommended probe

A study (not yet filed as a work item; proposed for the next step): survey bd,
git, and the projects above; state where each lands on readability versus
uniqueness; and recommend one of the three candidate shapes for kos, with a
migration path that does not break existing slugs or existing edges. Output: a
kos-side node and a finding. This is a kos-tool change, so it also needs a home
in `kos/_kos`.

## The immediate specimen (handled separately)

The three colliding ids are a live cleanup question independent of the general
fix. That decision (which copy is authoritative, whether to namespace them now,
or whether to express the summary-detail link some other way) is being taken up
directly rather than waiting on the id-scheme study. This finding records the
class; the specimen is the immediate work.

## Update 2026-08-15: fleet-wide sweep revises the prevalence and the shape

The placement audit widened from marvel to all 20 graphs. The original three
collisions were a marvel-only sample. The real map:

**13 exact-id collisions across two axes this finding did not capture.**

Axis 1, orc against a subrepo (8): `question-multi-host`, `question-permission-model`,
`question-stream-attachment` (the original three, orc/marvel), plus
`question-marvel-service-provider-shape` (orc/marvel, created this session),
`question-kos-bridge` (orc/spectacle), `question-pack-format` (orc/spectacle),
`question-session-bootstrap` (orc/forestage), and `grv-kos-as-task-tracker`
(orc/kos).

Axis 2, subrepo against subrepo (5), a class this finding never named:
`question-marvel-integration` (curtain/switchboard), and a four-id cluster
`elem-vendored-spec`, `grv-raw-http-escape`, `question-audit-mining`,
`question-distribution` (bloomctl/sidestep).

**Two sub-classes inside axis 2.** The curtain/switchboard case is one concept
("how do I integrate with marvel") independently and correctly instantiated in
two graphs. The bloomctl/sidestep cluster is the sharper case:
**sibling-replication**, where projects built from a shared design template (the
sidestep pattern) independently coin identical ids for the same recurring
concept. Neither is a within-graph defect. The candidate fix differs from the
orc/marvel summary-detail split: an orc-level shared-parent node ("the sidestep
pattern"), genuinely cross-cutting, that each sibling `instantiates`, rather
than namespacing.

**The larger finding: exact-id collision is the detectable minority.** The
sweep detects collisions by filename. That undercounts, because the same concept
frequently lives under DIFFERENT ids in two graphs, invisible to any id scan.
Measured examples: orc `question-runtime-adapter` vs marvel
`elem-runtime-adapter-framework`; orc `question-agent-protocol` vs marvel
`question-agent-communication-broker`; orc `question-gateway-type` vs marvel
`question-gateway-external-api`; sidestep `elem-primitives-over-composites` vs
bloomctl `elem-primitive-layer`. Exact-id collision is the visible tip; concept
duplication under divergent ids is the mass under the water. Any prevalence tool
(finding-133) must compare subject and content, not ids alone.

**The fleet already runs the unmanaged fixes.** Two of this finding's candidate
shapes are already in use, without tooling: subrepo graphs carry `aae-orc::`
namespaced cross-graph edge targets (marvel, forestage, curtain, switchboard),
which `kos validate` tolerates as warnings (finding-133); and the sibling
projects cross-reference each other in prose ("Same ruling as sidestep G1",
"Threshold prior from sidestep"). Hypothesis 1 (namespace prefix) is not
hypothetical; it is deployed and unmanaged. The design job is to ratify and tool
what the graphs already do, not to invent it.

The immediate specimens are being handled via `_kos/relocations.md` (the
interim forwarding index) and per-node `RELOCATION-PENDING` markers, pending the
id scheme and the move primitive (finding-132).

## Update 2026-08-16: the collision class is intra-graph too (finding numbers)

The orc graph itself produced two specimens of the same allocation
failure on its own FINDING numbers, a surface the original write-up
never named:

- finding-123 is a standing pair:
  `finding-123-harness-invocation-agent-identity-survey` and
  `finding-123-org-owned-fork-silently-disables-maintainer-edits`
  (unresolved at this writing).
- finding-136 was a pair for three hours:
  `finding-136-chat-as-probe-surface-harvest-gap` (committed 15:16,
  PR #225) and `finding-136-channel-selection-is-launcher-side`
  (committed 15:19, PR #226), two concurrent sessions three minutes
  apart. Resolved same day by PR #227: first-committed keeps the
  number; the later file renumbered to finding-137, whose renumbering
  note records the event from its side.

The mechanism and its fix were ALREADY ON THE BOARD before this
collision happened: `aae-orc-ul2h` (finding-number allocation:
prevent parallel-session collisions) and `aae-orc-vxaa` (allocate
the number at merge, not at authoring) are open bd tasks that name
the read-time-allocation race and the reservation-at-commit fix
outright. It is the read-max-plus-one race of finding-105 (session
numbers, Race A in session-close-mutex.md) expressed on a second
counter, and the tickets to close it predate the specimen.

The process observation this update exists to record: three
concurrent sessions handled the 136 collision (detected it,
diagnosed allocation-at-read, converged on first-committed-wins,
executed the renumber) and NONE of them consulted bd first, so all
three re-derived what ul2h and vxaa already said. Every prop existed;
no session ran one search. That is an F19/F25 instance
(active-surfacing gap; props without a backdrop) on the exact subject
matter of this finding, and it is stronger evidence for those
frontier questions than the collision itself.

What the specimens add to the record proper: the original write-up
treated collision as a CROSS-graph property of readable slugs; the
intra-graph pairs show the class is allocation discipline, indifferent
to whether the id is a semantic slug or a sequential integer. The
id-scheme study (aae-orc-hf58k) should take allocation discipline as
an axis beside readability and uniqueness, and the fix design belongs
to ul2h/vxaa, not to this addendum.

Resolution practice demonstrated by the 136 pair (recorded as
practice, not ratified): first-committed keeps the number; the later
entry renumbers itself with a note; history is never rewritten.

Addendum cross-references: aae-orc-ul2h, aae-orc-vxaa (the
pre-existing fix tickets), aae-orc-hf58k (id-scheme study),
finding-105 and `.claude/rules/session-close-mutex.md` (Race A),
`question-kos-multi-writer-concurrency` (multi-writer story),
`finding-137-channel-selection-is-launcher-side` (renumbering note),
`finding-136-chat-as-probe-surface-harvest-gap` (the number-keeper).

## Cross-references

- Placement audit that surfaced the specimen: this session (marvel-service-
  provider design, 2026-08-14/15).
- Frontier node that spans the same graphs: `question-marvel-service-provider-shape`.
- Related kos-tool concerns: `question-kos-multi-writer-concurrency`,
  `question-cross-repo-knowledge` (F10, the composition-layer knowledge
  problem this is one mechanism of).
- kos schema: `kos/schema/node.schema.yaml` (ids are described as "stable,
  unique, never reused" but uniqueness is scoped to one graph by implication,
  never stated as cross-graph).

## bd tracking

- aae-orc-hf58k (kos: node ids collide across graphs; id scheme study). bd
  work-queue anchor; this finding is the authoritative record.
