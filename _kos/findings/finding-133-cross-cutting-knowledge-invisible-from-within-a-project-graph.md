# finding-133: cross-cutting knowledge is invisible from within a project's graph, and kos has no tolerant cross-graph reference

**Date:** 2026-08-15
**Status:** OBSERVED. The more serious of the federation findings. A finding
**for kos**, and the graph-level form of F10 (cross-repo knowledge continuity).
**Scope:** kos-tool federation. Sibling of finding-131 (id collision) and
finding-132 (move with forwarding); together they are the federation problem.

## The problem

A person or agent working inside marvel, using kos, sees marvel's graph and only
marvel's graph. The cross-project learnings, tests, probes, questions, and ideas
that are not fully scoped within marvel are invisible from there. The most
valuable knowledge, the composition-layer knowledge, is exactly the knowledge a
single project's graph cannot show, because by definition it does not live in
that project.

The failure is not only that you cannot read it. It is that **you do not know it
exists.** A marvel session has no signal that there is a cross-project finding, a
platform-wide test, or an orc-level question bearing directly on what it is doing.
This is F10 stated at the graph level, and it is the same read-path hole that
F19 (active surfacing) and F25 (backdrops) keep circling.

## Requirements (the shape is open; this sets up the study)

1. **A cross-graph reference.** A marker inside marvel's graph that says "there
   is more about this topic in another graph." The shape is undecided: a
   reference node, an edge with a cross-graph target, or a lightweight stub.
   Deciding it is part of the study, and it is entangled with the id scheme
   (finding-131): a cross-graph reference needs a target id that is unambiguous
   across graphs.

2. **Unreachable-reference tolerance (hard requirement).** kos must not hang,
   freeze, error out, or conclude "bad reference" when the referenced graph is
   not reachable. Reachability is a real boundary, not an error: a user may have
   access to marvel's graph and no access to aae-orc at all. The correct behavior
   is informational degradation: "there is a reference here to node X in aae-orc,
   which cannot currently be reached," surfaced as a fact, never as a validation
   failure or a hang. Today kos treats an unresolved edge target as a warning
   within one graph; a cross-graph target that is simply out of reach is a
   different case and must be handled as absence-of-access, not
   absence-of-node.

3. **Level-of-detail projection across graphs.** A summary, lower-detail node MAY
   live in marvel: the marvel-relevant slice of a cross-cutting topic, explicitly
   marked as **not the authority**, pointing to the authoritative detailed node
   in aae-orc. This is the F25 stage-rig distinction across a graph boundary: the
   local summary is a backdrop (authored, shape-preserving, not the record); the
   remote authoritative node is the prop (verbatim, full fidelity). The local
   reader gets oriented; the record stays single and authoritative.

4. **Backref-aware revision.** The authoritative detailed node needs to know its
   backref links, where its summaries live across graphs, so that when it is
   updated it can flag or revise the projections that depend on it. Bidirectional
   awareness: the authority knows its projections; each projection knows its
   authority and is marked non-authoritative. Without this, LoD projections
   silently drift from the record, which is the finding-131 divergence hazard in
   a new place.

## This needs a prevalence analysis, not only a design

Before designing the mechanism, measure how common the problem already is.
finding-131 found three cross-graph id collisions, each a same-topic pair. Those
are the visible tip: any node whose knowledge is cross-cutting but which lives in
only one graph is a silent instance, and there is no current way to count them.
The study needs tooling that scans the graphs and estimates how much
cross-cutting knowledge is currently siloed, so the fix is sized against real
prevalence rather than a guess.

## Update 2026-08-15: fleet audit sharpens three of the requirements

The placement audit over all 20 graphs produced concrete evidence for this
finding.

**The reference direction runs both ways.** The requirements above assume a
subrepo needs to see cross-cutting knowledge held elsewhere. The audit found the
reverse too: `question-persona-model` is correctly held at orc (its subject is
the B14 taxonomy, a composition primitive), and marvel, sideshow, and forestage
each need a reference TO it. So the cross-graph reference is not only
subrepo-reaching-up; it is also orc-reaching-down to several subrepos at once.
The mechanism must be symmetric.

**Unreachable-tolerance is already deployed, unmanaged.** Requirement 2 is not
speculative. Subrepo graphs already carry `aae-orc::`-prefixed edge targets
(marvel, forestage, curtain, switchboard), and `kos validate` already degrades
them to warnings ("edge target 'aae-orc::...' not found in nodes/") rather than
failing. The tolerant behavior this finding asks for exists by accident. The job
is to make it intentional: distinguish out-of-reach (absence of access) from
not-found (absence of node), which the current warning does not.

**The prevalence pass must compare subjects, not ids.** finding-131's fleet
sweep (updated 2026-08-15) shows exact-id collision is the detectable minority;
the larger mass is the same concept living under DIFFERENT ids in two graphs
(orc `question-runtime-adapter` vs marvel `elem-runtime-adapter-framework`, and
others). A prevalence tool that counts id collisions will therefore report a
small number and miss most of the siloed cross-cutting knowledge. The pass this
finding calls for has to cluster by subject and content similarity, not by id,
or it measures the wrong thing.

## Prior art to study

- **DNS** delegation and referrals: authoritative-elsewhere with a followable
  pointer, and graceful behavior when a zone is unreachable.
- **Federated and distributed systems** with partition tolerance: a reference to
  an unreachable partition is a known state, not a crash. (CAP framing: choose
  availability of the local read over consistency with the unreachable graph.)
- **Wikipedia interwiki links and hatnotes** ("see also, on another wiki"): the
  human pattern for "more exists elsewhere," including how it degrades when the
  target is missing.
- **Symlinks and DOIs again** (shared with finding-132): the followable pointer
  whose target may be absent.
- **HippoRAG / graph-retrieval work** and the F25 stage-rig literature: LoD
  projection with a preserved authoritative source.

## Recommended probe

A study, with two outputs: (1) the cross-graph reference shape (ref node vs edge
vs stub), the tolerance contract for unreachable targets, and the LoD
projection-plus-backref model; and (2) a prevalence pass over the existing
graphs. Joined with finding-131 (ids) and finding-132 (movement) as the
federation study. Output: kos-side nodes and a finding, plus a prevalence number.

## Cross-references

- F10 (`question-cross-repo-knowledge`): this is F10 at the graph level.
- F19 (active knowledge surfacing) and F25 (`question-stage-rig-information-
  architecture`): the LoD-projection requirement is the stage rig across a graph
  boundary; the "you do not know it exists" failure is the F19 hole.
- finding-131 (id collision) and finding-132 (move with forwarding): the same
  federation problem; a cross-graph reference needs a cross-graph-unique id and a
  tolerant resolver.
- `question-marvel-service-provider-shape`: a live example of a cross-cutting
  node whose knowledge a marvel-only reader cannot currently see.
