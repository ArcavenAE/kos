# kos ask evaluation corpus

Scoring sheet for `kos ask` (bd `aae-orc-jajn3`, Phase 0), pre-registered by
`kos/_kos/nodes/frontier/question-kos-ask-retrieval-value.yaml`. The node fixes
the ordering before the verb ships: adoption dominates benchmark quality, and
the corpus must be real session questions captured before anyone knows what the
verb can do, so it cannot be gamed later.

I assembled this on 2026-08-16 against the two graphs as they stood that day:
`/Users/michael.pursifull/work/aae-orc/kos/_kos/` (the kos graph) and
`/Users/michael.pursifull/work/aae-orc/_kos/` (the orc graph). A later session
runs `kos ask <question>` against each entry and scores it on three axes the
node names: ranking (did the right id come up near the top), freshness (did it
prefer a bedrock or shipped answer over a superseded one), and provenance (did
it hand back the id and tier, not a summary that hides its source).

## How the questions were sourced

Fifteen questions distilled from real session activity, not idealized. Sources,
in order of weight:

1. Verbatim user asks and agent self-directed searches mined from the eight
   most recently modified session transcripts under
   `/Users/michael.pursifull/.claude/projects/-Users-michael-pursifull-work-aae-orc/`.
   The richest were `b9280faf` (identifier-collision work plus the `kos ask`
   build itself), `f651f189` (one-surface guidance), and `e63284fc`
   (bd/dolt/callbook topology).
2. The shapes the pre-registration node itself names ("what was decided about
   X, what was ruled out about Y, what is the open question on Z, where is the
   finding about W").
3. The standing frontier and graveyard nodes, which tell me what a session
   genuinely needs to look up.

Two real session shapes drove the design. The user's identifier-collision ask
in `b9280faf` (L508, verbatim: "don't we have some work about dealing with
identifier collusions ... Check our kos docs around this, and our findings and
orc findings and most important bd ticket ...") is the canonical
"point me at the prior decision, it spans findings and a bd ticket" shape. The
"who is working finding-136?" ask appears verbatim in two separate sessions
(`f651f189` L1003 and `e63284fc` L922); it looks like a graph lookup but is a
work-assignment question the graph cannot answer, so it is one of my
false-confidence probes.

## What the baselines are

**grep baseline.** The ripgrep an agent actually runs. I ran `rg -il <pattern>`
over the graph roots and recorded the file count (the triage cost: how many
files the reader opens to find the answer) and whether the right id is surfaced
or buried among unrelated hits. Commands are recorded per question so the run
is reproducible.

**orient baseline.** `kos orient` (Homebrew `kos alpha-20260809-030901-3495ed1`).
Two properties bound this baseline hard, and both were measured, not assumed:

- orient takes no query. It dumps the whole graph for the current directory:
  269 lines for the orc graph, 222 for the kos graph. The reader triages every
  line by eye. Section sizes (orc / kos): open questions 68 / 43, bedrock 26 /
  33, kos-findings 19 / 44, graveyard 7 / 10.
- orient is cwd-scoped to one graph and reads only `.yaml` findings. The orc
  graph has 19 `.yaml` findings and 124 `.md` findings; **orient surfaces none
  of the 124 `.md` findings**. I confirmed finding-044 and finding-136 (both
  `.md`) are absent from orient output. finding-128 in the graph documents this
  same blindness for `kos reflect`. So for any question whose answer is a `.md`
  finding, orient cannot reach it at all, and grep is the only baseline that
  does.

A note on freshness scoring (sub-question 4 of the node): orient groups by tier
(bedrock / frontier / graveyard sections), which is a weak freshness signal the
verb must at least match. grep gives no tier signal; it lists graveyard and
bedrock hits undifferentiated. The verb has to beat orient's tier-grouping, not
just grep's flat list.

---

## Kind (a): point lookups

### Q1. What is the decision on the kos read path, how do sessions retrieve knowledge from the graph?
- **Kind:** point lookup (spans both graphs)
- **Session provenance:** the whole `kos ask` build in `b9280faf`; the read-path
  gap is the subject of the pre-registration node.
- **Ground truth:** `question-kos-ask-retrieval-value` (kos, frontier),
  `question-read-telemetry-decision-value` (kos, frontier),
  `question-read-surface-sequencing` (kos, frontier),
  `val-retrieval-before-capture` (kos, **bedrock**, the load-bearing rule that
  no capture ships before retrieval value is shown), `kos-read-path-gap.md`
  (orc, idea), vision.md Gap 0.
- **grep:** `rg -il 'read.path|read telemetry|kos ask' kos/_kos _kos` -> 12 files.
  The three frontier questions and the idea surface, mixed with edge-vocabulary
  and taxonomy noise (finding-044-edge-vocabulary, finding-033, finding-133,
  finding-136). **Buried, and it misses the bedrock node**:
  `val-retrieval-before-capture` does not contain the literal phrase, so grep
  never returns the one bedrock answer that governs the whole area.
- **orient:** answer is split across both graphs, so orient must run twice. The
  frontier questions appear in the kos open-questions section (43 items) and the
  bedrock value in the kos bedrock section (33 items); the orc idea is in a
  section orient prints as 0 ideas for the orc, so orient does not list it.
  Present but unranked, cross-graph, and the reader scans two full dumps.

### Q2. What was decided about running bd on a Dolt server?
- **Kind:** point lookup
- **Session provenance:** `e63284fc` (bd/dolt topology session).
- **Ground truth:** `question-bd-dolt-server-architecture` (orc, frontier),
  finding-039-bd-localhost-dolt-server-phase-0-spike (orc, `.md`; Phase 0
  shipped, bedrock promotion gated on soak), charter F26.
- **grep:** `rg -il 'dolt server|dolt sql-server' _kos` -> **27 files**. The two
  right answers sit among two dozen bd-friction findings that mention "dolt
  server" in passing (finding-081, finding-101, finding-105, finding-115,
  finding-124, and more). **Badly buried**; the highest-triage-cost point lookup
  in the set.
- **orient:** the frontier question appears in the orc open-questions section
  (68 items). finding-039 is a `.md` finding, so **orient cannot surface it at
  all**. The reader gets the question node but not the shipped-state finding.

### Q3. Where is the finding about the bd versus bv ecosystem and the export-scope problem?
- **Kind:** point lookup (find-the-finding)
- **Session provenance:** the bd/bv confusion is live in `e63284fc` and
  `b9280faf`.
- **Ground truth:** finding-044-bd-bv-ecosystem-and-export-scope (orc, `.md`).
- **grep:** `rg -il 'beads_viewer|bd.{0,3}bv|export scope' _kos` -> 9 files.
  finding-044 is present but not distinguished from finding-081, finding-087,
  finding-134 and three frontier nodes. Reader must open several to confirm.
- **orient:** finding-044 is a `.md` finding, so **orient cannot surface it**.
  This question is unanswerable from orient. grep is the only baseline that
  reaches the target, and only by brute force.

### Q4. What is the agent taxonomy, the five primitives?
- **Kind:** point lookup
- **Session provenance:** the taxonomy (persona/theme/identity/role/process)
  recurs across sessions as the canonical vocabulary.
- **Ground truth:** finding-019-agent-taxonomy (orc, `.md`), charter B14,
  `question-persona-model` (orc, frontier, resolved to B14).
- **grep:** `rg -il 'five primitive|agent taxonomy' _kos` -> 9 files. finding-019
  surfaces (distinctive phrase), alongside idea files and finding-047. Reasonable
  but not ranked; `question-persona-model` does not surface (different wording).
- **orient:** B14 appears in the orc charter-items section (matched via the
  bedrock node), which is a clean hit. But finding-019 (the full map) is `.md`,
  so orient shows the charter line and not the finding behind it. Partial.

### Q5. According to kos, what predicts whether a knowledge system survives?
- **Kind:** point lookup (single authoritative node)
- **Session provenance:** `elem-who-pays-survival-predictor` is cited directly in
  the pre-registration node as the grounding for adoption-dominates.
- **Ground truth:** `elem-who-pays-survival-predictor` (kos, **bedrock**).
- **grep:** `rg -il 'who pays|survival predictor' kos/_kos` -> 3 files, the
  bedrock node first. **Clean surface.** This is grep at its best: a distinctive
  phrase with few decoys. The verb has to at least match this, and to add the
  tier signal (that this is bedrock) that grep does not give.
- **orient:** the node is in the kos bedrock section (33 items). Present,
  tier-labeled, but the reader scans 33 lines to find it.

---

## Kind (b): ruled-out / negative

### Q6. What was ruled out about pack management?
- **Kind:** ruled-out (this is a literal `kos ask` test query the build session used)
- **Ground truth:** `grv-skillshare-for-packs` (orc, **graveyard**), charter G1.
  Positive counterpart for contrast: `elem-packs-in-marvel` (orc, bedrock).
- **grep:** `rg -il 'pack management' _kos` -> 8 files. The graveyard node is
  present but sits beside `elem-packs-in-marvel` (the opposite, positive
  decision) and four sideshow idea files. **grep gives no signal that the
  graveyard node is the ruled-out answer**; the reader infers it from the path.
- **orient:** the orc graveyard section is only 7 items, so orient's tier
  grouping lets the reader jump straight to graveyard and find
  `grv-skillshare-for-packs` fast. **This is where orient beats grep**, and it
  is the bar the verb must clear on negative questions.

### Q7. Did we rule out using kos as a task tracker, and why?
- **Kind:** ruled-out (spans both graphs)
- **Session provenance:** the kos-versus-bd boundary is a recurring theme.
- **Ground truth:** `grv-kos-as-task-tracker` (exists in **both** graphs, both
  graveyard). Related: `question-kos-task-tracker-relationship` (orc, frontier).
- **grep:** `rg -il 'task tracker' kos/_kos _kos` -> 10 files. The graveyard node
  appears twice (once per graph, an id collision across graphs), among
  `elem-kos`, ideas, and a probe. The duplicate is itself a hazard: the two
  files are different content under the same id.
- **orient:** each graveyard node shows in its own graph's graveyard section (7
  and 10 items). Two runs, but tier grouping makes each easy. orient handles the
  collision honestly by keeping them in separate graph dumps; grep flattens them
  into one list.

### Q8. Why did we not adopt speckit or the bmad document format?
- **Kind:** ruled-out
- **Ground truth:** `grv-speckit-adoption` (orc, graveyard),
  `grv-bmad-format` (orc, graveyard), charter G2/G3.
- **grep:** `rg -il 'speckit|bmad.{0,6}format' _kos` -> 5 files. Both graveyard
  nodes surface among a probe and two ideas. **Reasonable**; distinctive terms
  keep the decoys down.
- **orient:** both in the 7-item orc graveyard section. Clean tier hit. Both
  baselines do acceptably here; the verb must not regress.

### Q9. Was git considered a sufficient substrate for kos, and why was that ruled out?
- **Kind:** ruled-out
- **Ground truth:** `grv-git-as-sufficient-substrate` (kos, graveyard),
  `grv-git-semantic-layer` (kos, graveyard). Positive contrast:
  `elem-storage-model` (kos, bedrock, "immutable fact store over git").
- **grep:** `rg -il 'git.{0,12}substrate|sufficient substrate' kos/_kos` -> 11
  files. The graveyard node is present but buried among substrate-hypothesis
  findings (finding-036, finding-040) and the bedrock storage model, which is
  the near-opposite conclusion. **The reader risks reading the positive node as
  the answer.** Freshness/polarity matters and grep gives none.
- **orient:** graveyard section (10 items) holds both ruled-out nodes; bedrock
  section holds the storage model. Tier grouping separates ruled-out from
  adopted, which is exactly the distinction grep loses.

### Q10. Did we rule out flat-file context loading, and what was the measured ceiling?
- **Kind:** ruled-out (with a specific fact: the ceiling)
- **Ground truth:** `grv-flat-file-context-scaling` (kos, graveyard, "~20 epics").
  Related bedrock: `elem-context-ceiling`.
- **grep:** `rg -il 'flat.file|context ceiling|20 epics' kos/_kos` -> 12 files.
  The graveyard node is present but buried among four findings and the bedrock
  `elem-context-ceiling` (which states the ceiling positively). Reader must open
  files to get the number.
- **orient:** title-only, so even when it lists `grv-flat-file-context-scaling`
  in the graveyard section, it does not carry the "~20 epics" fact. orient tells
  you which node, never what it says. The verb should return the id and tier and
  let the reader open one file, not twelve.

---

## Kind (c): open-question / frontier

### Q11. What is still open on charter management?
- **Kind:** frontier (spans both graphs)
- **Session provenance:** charter re-inflation is a standing concern; the
  charter-light-touch rule exists because of it.
- **Ground truth:** `question-charter-management` (orc, frontier),
  finding-008-charter-inflation (orc), finding-018-charter-management (orc),
  finding-041-charter-inflation-as-graph-gap (kos), charter F22/F25.
- **grep:** `rg -il 'charter management|charter.{0,4}inflation' _kos kos/_kos` ->
  **28 files**. The question node is present but drowned; and the results include
  the finding-018 id collision (two different orc files share finding-018) and
  finding-068 (which is about a different collision). High triage cost.
- **orient:** the question is in the orc open-questions section (68 items). Two
  of the four ground-truth findings are `.md` and invisible to orient
  (finding-008 and finding-018 are `.yaml`, so those two do show in the
  19-item kos-findings section; finding-073 and the `.md` set do not). Partial
  and split across two graphs.

### Q12. What is the open question on active knowledge surfacing, the push side?
- **Kind:** frontier
- **Ground truth:** `question-active-knowledge-surfacing` (orc, frontier),
  charter F19, finding-004-passive-graph-insufficient (orc, `.yaml`),
  `elem-predictor-layer` (kos, frontier).
- **grep:** `rg -il 'active knowledge|passive graph' _kos kos/_kos` -> **23 files**,
  overwhelmingly idea files (prediction-engine, stage-rig, bd-orient-backdrop,
  and a dozen more). The question node and finding-004 are in there but the
  reader wades through 20-plus ideas first. Buried.
- **orient:** the question node lists in the orc open-questions section;
  finding-004 is `.yaml` so it does show in the 19-item kos-findings section.
  Better than grep here because orient does not surface the idea-file noise, but
  the reader still scans two sections across the dump.

### Q13. What remains open on the director design and the agent communication protocol?
- **Kind:** frontier
- **Ground truth:** `question-director-design` (orc, frontier),
  `elem-director` (orc, frontier), finding-064-director-control-channel-options
  (orc, `.md`), charter F1/F3.
- **grep:** `rg -il 'director' _kos` -> **124 files**. "director" is a common word
  across the marvel and supervisor material, so grep is nearly useless: the one
  question node is one hit in 124. **The strongest grep-buries-it case in the
  set.**
- **orient:** `question-director-design` appears by name in the orc
  open-questions section, and orient also shows `elem-director` and the two
  director briefs. So **orient sharply beats grep here** (scan one section of 68
  vs open 124 files), though finding-064 (`.md`) stays invisible. This question
  is the clearest case for a ranked verb: it must return the director question
  and its `.md` finding, near the top, in one shot.

---

## False-confidence probes (the graph cannot answer these well)

These exist so the corpus can measure whether the verb fabricates confidence.
The success behavior is a miss, a low-confidence answer, or an explicit
"the graph does not track this," not a confidently-ranked wrong node.

### Q14. Who is working finding-136?
- **Kind:** false-confidence (a work-assignment question wearing a lookup's clothes)
- **Session provenance:** verbatim in two sessions (`f651f189` L1003,
  `e63284fc` L922).
- **Ground truth in the graph:** none for the question as asked. finding-136
  (chat-as-probe-surface-harvest-gap) **exists** as a `.md` node, but it records
  a knowledge gap, not who is assigned to it. Work assignment lives in bd and
  git branches, which the kos graph does not track (see `grv-kos-as-task-tracker`
  and the CLAUDE.md note that a bd claim does not even name an actor).
- **grep:** `rg -il 'finding-136' _kos` -> 3 files (finding-136 itself, plus
  finding-131 and finding-137 that cross-reference it). grep correctly returns
  "here is the finding," which is honest, because grep never claims to answer
  "who is working it."
- **orient:** finding-136 is `.md`, so orient does not list it. orient returns
  nothing, which is honest by omission.
- **Failure mode to catch:** the verb confidently returns finding-136 as if it
  answered "who is working it." The right answer is to surface the finding and
  flag that assignment is a bd/git question the graph does not hold.

### Q15. What did we not ticket this session, and what context will we lose when it compresses?
- **Kind:** false-confidence (absence-detection about live session state)
- **Session provenance:** verbatim `f651f189` L1132 and L1670 ("what did we not
  ticket?").
- **Ground truth in the graph:** none. This is a question about the absence of
  records and about volatile session state; a static graph structurally cannot
  answer it. It is precisely what the F19 predictor layer aspires to and does not
  yet do (`question-active-knowledge-surfacing`, `elem-predictor-layer`).
- **grep:** `rg -il 'did not ticket|not ticket|lose.{0,20}compress|context.{0,15}lost' _kos`
  -> 1 file (finding-087, a false-positive match on "context lost"). grep
  honestly returns near-nothing.
- **orient:** dumps the whole graph, none of which answers the question. Honest
  by irrelevance.
- **Failure mode to catch:** the verb invents a plausible-looking answer (for
  example, returning `question-active-knowledge-surfacing` as though it were the
  answer rather than the reason there is no answer). A ranked, confident hit here
  is worse than grep's empty result.

---

## Kind breakdown

- Point lookup (a): Q1, Q2, Q3, Q4, Q5 (5)
- Ruled-out / negative (b): Q6, Q7, Q8, Q9, Q10 (5)
- Frontier / open (c): Q11, Q12, Q13 (3)
- False-confidence (graph cannot answer): Q14, Q15 (2)
- Total: 15

## Scoring instructions for the later session

For each question, run `kos ask "<question>"` and record:

1. **Rank of first correct id.** Where in the answer list did a ground-truth id
   appear (1 = top, or "absent")? For Q14/Q15 the correct behavior is no
   confident hit, so a top-ranked node is a failure, not a success.
2. **Freshness/polarity.** Did the verb prefer the bedrock or shipped answer over
   a superseded or opposite-polarity one? Q5 (bedrock), Q6 and Q9 (graveyard vs
   the positive counterpart) are the discriminating cases.
3. **Provenance.** Did the answer carry the id and tier so the reader can open
   the source, or did it hand back a summary that hides where it came from? The
   node warns that a confident summary of an untested finding is worse than grep.
4. **`.md` reach.** Q2, Q3, Q13, Q14 have `.md`-finding ground truth that orient
   cannot see. Did the verb reach them? This is the clearest place the verb can
   beat both baselines at once.
5. **Proximity lift (success signal b).** For at least one question, did a
   correct hit come from graph proximity rather than lexical match, that is, a
   hit grep's pattern did not contain? Q1 (the bedrock `val-retrieval-before-
   capture` that grep missed) and Q9 (polarity separation) are the best
   candidates.

The pre-registered success signal, restated: over the ten sessions after the
verb ships, `kos ask` is invoked in at least half the sessions that consult the
graph at all, and at least one session records an answer through a hit the
lexical set did not contain. Both halves required. This corpus measures the
second half directly and gives the first half an honest baseline to be judged
against.
