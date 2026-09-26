# Idea: the code graph as a second lens

Pre-hypothesis capture of the 2026-08-08 code-graph research input. Generative,
possibly contradictory, no commitment. This file holds the readings of the
briefing while they are still readings, before any of them is a claim I would
defend. When one becomes testable it leaves here for a frontier question and a
probe brief; the question node `question-code-graph-correspondence` is where
that migration lands, and it already carries the first two (sub-questions A and
B, now measured).

The props stay verbatim elsewhere: the two research documents
(`docs/research/2026-08-08-code-graph-briefing.md` and its
`-initial-analysis.md`) are committed and are the primary source. This file
does not digest them. It records what I think they might mean, which is a
different kind of object from what they say.

## The input, in one paragraph

The briefing surveys six families of source-code graph tooling, distinguished
by how each resolves a symbol reference to its definition (compiler front end,
tree-sitter plus ranking, external graph DB, local MCP wave, live LSP, pre-AI
single-language). Its argument: extraction is the solved part; ranking,
freshness, and provenance are not; the star leaderboard is inverted, with the
deep work low-star or archived and the high-star work months old and packaged
for an audience that cannot evaluate it. Its recommendation for this stack:
read the deep lineage for design, adopt SCIP as the only format, take Serena as
one measured baseline, implement Aider's PageRank ranking, and make kos the
sink rather than adopting a competing graph store. The load-bearing structural
fact it leans on is that kos is already a typed-edge graph with a three-state
confidence model, so adopting any tool that ships its own persistence means
running a second substrate with no shared confidence semantics.

## Reading 1: a correspondence layer is arriving

The briefing keeps circling a thing kos already has a word for. It wants edges
that say DEFINES, REFERENCES, IMPLEMENTS, DEPENDS_ON between symbols, tagged by
how they were derived (compiler-resolved, tree-sitter-inferred, LLM-guessed).
kos already carries typed edges between spec nodes and code nodes as a
first-class node type (`correspondence`), and it already separates how-sure-we-
are into bedrock, frontier, graveyard. So one reading of the whole briefing is:
the field is re-deriving, badly and per-vendor, the correspondence layer kos was
built to hold. Under this reading the code graph is not a new system to stand up.
It is a second population of correspondence edges, sourced from a compiler
instead of from a human writing a spec, landing in the same store beside the
spec-to-code correspondences already there.

This is the reading that connects to the kos-graph node `elem-correspondence-
layer` (typed correspondence between spec and code) and to the briefing's own
observation in section 7 that Graphify's confidence-tagging design was "a good
idea independently arrived at in KOS." If two projects independently arrive at
the same structure, that is weak evidence the structure is real, which is the
criterion the kos-graph finding-023 (signal-to-noise) tried to make precise.
Weak evidence, held as such.

The contradiction inside this reading: a spec-to-code correspondence is authored
and load-bearing; a symbol-to-symbol edge from a compiler is derived and
disposable, regenerated on every index. Calling both "correspondence" may be
lumping two things that behave differently. Whether they belong in one node type
or two is exactly sub-question D (source grade is not tier) and G (one mission
or two), and I do not want to answer it here by choosing a word.

## Reading 2: this forces the substrate decision

If the code graph lands in kos as facts, then kos is now being asked to hold a
class of data it was not sized for: one SCIP index per commit, per repo, across
25-plus repos, regenerated on a cadence. The measurement in `finding-aae-orc-
msqx-scip-indexing-cost` puts a number on the artifact side (60.83 MB for one
warm sweep of the live fleet), but the recurring, versioned, per-commit shape is
the thing kos-graph finding-036 (the substrate hypothesis: YAML-in-git proved
the graph model, now the graph needs a substrate that is not files) and
finding-040 (three graveyard entries measure the same ceiling) already said the
current substrate cannot carry. The kos-graph graveyard node
`grv-flat-file-context-scaling` measured the flat-file ceiling at roughly 20
epics. A per-commit code graph is orders of magnitude past that.

So one reading is that the code-graph arc is not really about code graphs. It is
the workload that forces the substrate move kos has been circling: the thing
heavy enough that YAML-in-git stops being tenable and Dolt (or whatever wins
question-kos-multi-writer-concurrency and the dolt-service-plane work) becomes
load-bearing rather than nice-to-have. Under this reading the right sequencing
is substrate-first, and building the ingest before the substrate is building on
the ceiling we already measured.

The contradiction: the briefing's R1 says the ingest is "non-trivial but
bounded, and entirely in Rust," as if it could proceed on the current store. It
might, at small scale. The tension between "bounded, do it now" and "this is the
thing that breaks the substrate" is unresolved and is sub-question C. I am not
choosing.

## Reading 3: the methodology is escaping the tool

The briefing is ostensibly about which code-graph tool to adopt. But its most
transferable content is not a tool recommendation at all. It is a method: mark
verified apart from reported; treat agreement across sources as possible
contamination rather than corroboration; ask what resolves a reference before
believing a category label; price a claim against an executed measurement rather
than a vendor benchmark. That is the same discipline this fleet already runs
under other names (the upstream-claim-gate, the premise-check rule, F25's
verbatim-props). One reading of the briefing is that it is an outside instance
of our own method, applied to a field we do not own, and that the durable value
is the method surviving the specific tools, all of which will be stale in a year.

This reading has teeth for how the arc itself was filed. The defect that
`aae-orc-aj3c` corrects (research triaged into sixteen work tickets and zero
knowledge artifacts) is the methodology escaping into the wrong container: the
reasoning went into bd, which tracks work, when it belonged in kos, which tracks
knowledge. So "the methodology is escaping the tool" is true in two senses at
once, one about the field and one about us, and the second is the more
uncomfortable.

The contradiction: if the durable value is the method and the tools are
disposable, that argues against building anything heavy now. But the symbol-
workload measurement (`finding-aae-orc-5lbu`) found live LSP genuinely beating
grep on correctness, which is a concrete capability, not a method. Both can be
true. I am recording the tension, not dissolving it.

## Cross-reading tensions worth keeping

- Reading 1 wants the code graph inside kos as correspondence facts. Reading 2
  says that is exactly what the current kos substrate cannot hold at cadence.
  Together they say: yes it belongs in kos, and no the kos we have today cannot
  take it. That is not a contradiction to resolve by picking a side; it is the
  shape of the substrate decision.
- The briefing recommends implementing Aider's PageRank ranking in Forestage.
  Forestage was retired by operator directive on 2026-07-31. The briefing has
  now twice assumed Forestage is a live target. So a high-grade source carries a
  stale premise, which is itself an instance of Reading 1's point that source
  grade and confidence are different axes. Every recommendation naming a
  component needs checking against the current roster. Tracked as sub-question F
  (where ranking lives now) and recorded as a source-calibration fact in the
  question node.
- The briefing's "do not treat 'we have a code graph' as completion" is the same
  warning as F25's "the loft is not the show." Extraction is the loft; ranking,
  freshness, and provenance are the performance. Recording the resonance, not
  claiming it proves anything.

## What has already left this file

- Sub-question A (does live LSP suffice) became `finding-aae-orc-5lbu`. Measured:
  live LSP wins on symbol workloads and cannot serve orientation, history,
  cross-repo scope, or provenance. A split, not a verdict.
- Sub-question B (is compiler-grade indexing affordable) became
  `finding-aae-orc-msqx`. Measured: cheap in time, expensive in memory for Rust,
  near-free for Go, with a silent-degradation failure mode that a zero exit code
  hides.

Both now live as findings and confidence nodes, edged into
`question-code-graph-correspondence`. The remaining readings stay here until
they are testable.
