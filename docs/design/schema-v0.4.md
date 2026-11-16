# Node schema v0.4: design

**Date:** 2026-11-15
**Author seat:** arcaven architect seat, on the plan for aae-orc-e689 accepted
by the arcaven supervisor (2026-11-15).
**Status:** design only. Nothing here is built. Section 6 lists the items still
open, each with the default this note takes.
**Tickets:** aae-orc-e689 (the schema change). This note is aae-orc-eb8nn. The
build is aae-orc-kv2rv (fields) then aae-orc-blj1w (checks). After it come
aae-orc-8j6cc (validator cell rules) and aae-orc-ii3yb (fleet migration).

kos has been recording one confidence tier per node, and that one axis carries
two meanings: how far a claim has been examined, and how much rests on it.
The operator ruled the fix on 2026-11-14 (Desk kdec-1, option 2): a
two-dimensional matrix, examination by commitment. In the same sitting
(kdec-2, option 1), the operator folded two smaller tickets into this schema
change: a provenance kind (aae-orc-didr) and the separation of source grade
from tier (aae-orc-pwnh). This note says what v0.4 adds, and in what order.

## 1. Premises checked

| Premise | Check | Result |
|---|---|---|
| The substrate model is ruled | `aq show aae-orc-e8wn` | closed 2026-11-14, kdec-1 ruled 2, the 2D matrix |
| No v0.4 work has landed | `schema/node.schema.yaml` on kos main 87743bb | its version history ends at v0.3; no examination, commitment, kind, promoted_from, falsifier or conditional_on |
| No v0.4 work is in flight | `gh pr list --repo ArcavenAE/kos --state open`, `git ls-remote` | no PR and no branch for it |
| Old binaries keep unknown fields | `src/model.rs` | `Node` has no `deny_unknown_fields`, so a field the binary does not know is ignored |
| How binaries treat an unknown edge type | `src/model.rs`, `git log 3495ed1` | since 3495ed1 (#89), an unknown type loads as `EdgeType::Unknown(raw)` and kos validate fails the node; a binary from before #89 dropped the whole node. `LEGAL_EDGE_TYPES` has 8 entries |
| How validate treats an edge target | `src/validate.rs` check 3 | a plain target found nowhere warns; a `graph::target` that does not exist fails (#177) |

## 2. The matrix

Two new optional fields carry the coordinate. The names are those of
finding-045's sketch (orc `_kos/findings/finding-045-rd-vs-design-provenance.md`):

- `examination`: `un-examined | questioned | answered | ruled-out`
- `commitment`: `floating | load-bearing | essential | abandoned`

| | floating | load-bearing | essential | abandoned |
|---|---|---|---|---|
| **un-examined** | idle | substrate | (rare) | (rare) |
| **questioned** | scratch | **frontier** | foundation-premise | being-ruled-out |
| **answered** | (rare) | harvested-emergent | **bedrock** | (rare) |
| **ruled-out** | (rare) | (rare) | (rare) | **graveyard** |

The v0.3 tiers are named cells of this table:

- `bedrock` is answered x essential;
- `frontier` is questioned x load-bearing;
- `graveyard` is ruled-out x abandoned.

`confidence` stays required in v0.4, with its v0.3 values. A node that sets
both new fields on a named cell must agree with its `confidence`; that rule
belongs to aae-orc-8j6cc. A node in an unnamed cell keeps the `confidence` its
directory gives it, and its coordinate records the real cell. Where such
nodes live is decided by the migration (aae-orc-ii3yb), not here. The cell
names in the table are labels for people and are not schema values.

## 3. Provenance

Three new optional fields in the existing `provenance` block:

- `kind`: `deliberated | emergent-ratified | inherited | pre-question`. These
  are the values from aae-orc-e689, minus `substrate`, which the matrix now
  carries as a cell (un-examined x load-bearing). One fact in two places would
  drift.
- `promoted_from`: a list of `{target, note}`. `target` is the ADR, finding,
  convention or question node the claim was promoted from.
- `falsifier`: a string naming a condition that someone can check and that
  would unmake the claim.

One new optional field at node level:

- `source_grade`: `authored | compiler | heuristic | inferred`, for how a fact
  was produced (briefing 4.2: compiler-derived, heuristic, inferred by a
  model). Hand-written nodes are `authored`, which is also the meaning of an
  absent field.

Two requirements, carried from aae-orc-pwnh into this ticket by kdec-2, as
written:

1. Absence from a code index is staleness, not graveyard. A compiler fact
   that a later index no longer contains is out of date; it is never moved to
   ruled-out.
2. Source grade must not set the tier. No value of `source_grade` sets or
   implies `confidence`, `examination` or `commitment`. A compiler fact is
   current until it drifts, not bedrock.

## 4. conditional_on

aae-orc-e689 asks for `conditional_on` edges: a decision node names the
premises its argument depends on, and when a premise moves, the dependents
are flagged for re-validation. v0.4 makes it a node field, not a ninth edge
type, and it is a list of `{target, note}`.

The reason is a ruling already on record. aae-orc-gq1c was ratified on
2026-08-09: "the eight edge types stand. No ninth type." It gave the version
skew as the measured reason. Section 1 updates that reason without
changing the conclusion. A binary from before #89 drops a node with an
unknown edge type, and a current binary loads it as `Unknown` and kos
validate fails it. So a ninth type would make every graph that uses it fail
validation under any kos release that predates the type. An unknown field,
by contrast, is ignored: `Node` has never carried `deny_unknown_fields`
(`git log -S deny_unknown_fields -- src/model.rs` is empty), and kos
validate does not flag one. gq1c used the same field shape for
`out_of_scope`. This note applies that ruling to `conditional_on`. If the
operator wants an edge here instead, it needs a ruling that reopens gq1c.

The flag is a warning, never a gate and never an automatic move (SOUL section
8, ADR-007). kos validate keeps no record of an earlier run, so it cannot see a
parent change; it warns on the parent's state. A node whose `conditional_on`
parent is undermined now (confidence graveyard or placeholder, examination
ruled-out, or commitment abandoned) gets one warning naming the parent and the
reason. A questioned or frontier parent is the normal case and does not warn.
It does not change the dependent or fail the run. The warning reads the
parent's state in the graph being checked: a parent that is a finding, or a
node in a sibling graph, is resolved for existence only. Real change detection
(a comparison with the base commit, or a stamp on the dependent) is later
work, and is left to its own ticket. The e689 line "Default: substrate-conditional unless proven
substrate-independent" is read as guidance for authors, not as a value kos
fills in: v0.4 writes nothing that an author did not.

A target that does not resolve is treated as validate treats an edge target
today (section 1): a plain target warns, and a `graph::target` fails. The
same rule applies to `promoted_from`.

## 5. Order of work

1. **aae-orc-kv2rv (PR 1).** `schema/node.schema.yaml` documents v0.4, and
   `src/model.rs` parses every field above. All the new fields are optional,
   so every v0.3 graph parses unchanged. Tests: kos has no Node writer
   (promote and compact edit the text in place), so there is no node-level
   write to round trip. A v0.3 node parses with every new field absent, a v0.4
   node parses with every new field present, promote and compact leave a v0.4
   node's new fields byte-identical, each new enum and the `{target, note}`
   struct serialize and re-parse to equal values, a value outside a
   vocabulary parses as `Unknown(raw)`, and a node carrying a field the
   binary does not know still parses. No directory changes. Size M.
2. **aae-orc-blj1w (PR 2).** kos validate resolves `promoted_from` and
   `conditional_on` targets by the section 4 rule and prints the dependent
   warning. It also checks that each enum value is legal, which is a
   structural check and may fail. Size M.
3. **aae-orc-8j6cc,** the validator's cell rules, after PR 1.
4. **aae-orc-ii3yb,** the migration of every fleet graph, after 8j6cc. It
   makes the new fields required and decides directory layout.

The new verbs (`notice`, `demote`, `recant`, `resurrect`) and the renderer
(aae-orc-qy4j) are outside this change.

## 6. Open items, with this note's defaults

- **r1, names.** Axis and cell names as in finding-045's sketch. Default taken.
- **r2, directories.** The confidence directories are unchanged in v0.4; the
  coordinate is a field, and the migration decides layout. Default taken.
- **r3, optional first.** The new fields are optional in v0.4 and become
  required only after the migration. Default taken.
- **r4, one bump with aae-orc-eym4j.** That ticket adds `checked-on` and
  `replaces` fields with warning-only checks (Desk cr1112-goose-3, Adopt).
  Folding it into v0.4 gives one bump and one migration, which is kdec-2's
  reasoning. Default: fold it in. Not folded yet, because it changes eym4j's
  scope and is with director.
- **r5, placeholder.** `placeholder` has no cell in finding-045's sketch.
  Default: questioned x floating, since a placeholder is an honest marker of
  something unknown with nothing resting on it yet.
- **r6, out_of_scope.** aae-orc-gq1c's ratified field shape is not built. It
  could ride in v0.4 at no extra migration cost, since it is optional and
  needs no backfill. Default: leave it to gq1c, and say so to whoever picks up
  gq1c.
