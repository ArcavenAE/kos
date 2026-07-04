# The Nine Improvements, Re-Read Through the Survey and Roadmap v3

> Cross-document assessment: `kos-nine-improvements.md` evaluated against
> the literature survey's failure-mode findings and roadmap v3's
> integration decisions. Citations use the survey's citekeys; full BibTeX
> entries are in `kos-roadmap-v3.md`.

**Verdict distribution: 2 strengthened, 3 modified, 3 correctly deferred,
1 partially killed.**

---

## Strengthened

### #5 Retrodiction

The survey's biggest gift to any single proposal. The backtesting
literature [@bailey2014pseudo] and hindsight-bias work
[@fischhoff1975hindsight; @dekker2002field] convert the "graveyard risk"
the nine-doc named (hindsight leakage) from a caveat into a designed
control: blinding isn't mitigation, it *is* the experiment. v3 correctly
promoted this to first science item.

One thing the nine-doc got right that deserves note: it pre-registered a
prediction (≥2 of 4 incidents detectable at T-minus-one-sprint) before
knowing the method literature — that instinct is exactly the
registered-reports discipline [@nosek2014registered] the survey later
validated.

### #8 MCP Server

Horvitz's mixed-initiative principle [@horvitz1999principles] plus the
LLIS retrieval-at-need failure [@weber2001intelligent] upgrade this from
"integration convenience" to "the delivery mechanism the entire literature
says is mandatory." Benefit must arrive inside the agent's task loop, not
in a separate consultation step. v3's promotion into milestone MR is
right.

---

## Modified

### #3 Confidence Half-Life

The nine-doc invented decay classes (durable/stable/volatile) with
admittedly arbitrary horizons. ACT-R base-level activation
[@anderson1991reflections] replaces invention with a validated
formalization where decay is *retrieval-reinforced* — which creates a
dependency the original missed: you cannot compute retrieval-weighted
decay without retrieval counts. Hence v3 sequencing it behind #7's
telemetry. The original's "graveyard risk: arbitrary horizons" is now
resolved, but the improvement gains a hard prerequisite.

### #7 Knowledge Economics

Survives intact mechanically, but the survey adds a warning the nine-doc
lacked: Austin's measurement dysfunction [@austin1996measuring].
Retrieval counts as *diagnostic* telemetry are fine; the moment they
become status metrics ("your nodes are never read"), authors optimize for
retrieval-bait. The original's Hawthorne acceptance ("bias toward more
graph use is desired") was too casual — Goodhart cuts deeper than
Hawthorne. v3's diagnostic-never-gate rule is the fix.

### #2 Canary Nodes

Strengthened *and* constrained. Mutation testing [@demillo1978hints] and
chaos engineering [@basiri2016chaos] give it lineage; Goodhart gives it a
promotion — v3 names canary detection-rate as the *only* game-resistant
health metric. But the survey implies a constraint the original missed:
canaries test detection of *corruption*, not detection of *staleness or
gaps* — different failure classes need different canary templates. The
template library needs a taxonomy matching the six signal types before
planting.

---

## Correctly Deferred

### #1 Source Reliability Ledger

The Admiralty two-axis insight survives conceptually (provenance ≠
confidence; never merge the fields), but with one human and few agent
versions, grades cannot discriminate — and worse, the
intelligence-analysis literature the survey touched (structured analytic
techniques' mixed empirical validation) suggests reliability grading is
precisely the kind of SAT whose real-world efficacy is *asserted more
than measured*. The deferral trigger (≥4 sustained sources) is right.

### #6 Dissent Nodes

The historiography instinct is sound, but at n=1 human the mechanism
records nothing the graveyard doesn't. The parked trigger (second
sustained operator) is correct. Note for when it reopens: the two
backfill candidates identified (Q4-vs-Q5 prioritization;
charter-freeze-vs-query-cited-edits) remain valid test material.

### #9 Chronicle

Grudin-safe by construction (pure read-side), which is why it survives at
all. But it reuses Issue 21's traversal machinery, so sequencing behind
the query layer isn't deferral — it's dependency. The
citation-per-sentence discipline in the original anticipated the
grounded-generation requirement correctly.

---

## Partially Killed

### #4 Toulmin Fields

The survey's most direct hit on the nine-doc. Argumentation formalisms
are the *canonical* Grudin victim: gIBIS [@conklin1988gibis], QOC, and
the whole design-rationale program died on exactly this intrusiveness
[@shipman1997design] — "Formality Considered Harmful"
[@shipman1999formality] reads as a pre-written obituary for the proposal.
The original's own graveyard risk ("if authors skip it, another soft
constraint") understated the case: the literature says authors *will*
skip it, universally, across four decades of attempts.

What survives is instructive: **one field, `rebuttal_conditions`, on
bedrock only** — and it survives not as argumentation but because it
doubles as Issue 24's machine-checkable reopener vocabulary. The lesson
generalizes: **formal structure earns admission only when a machine
consumes it.** Structure for human readers alone is dead weight;
structure the tooling acts on is infrastructure.

---

## Two Cross-Cutting Observations

**The nine-doc's implicit theme was validated.** Its closing note flagged
#1/#2/#5 as "testing the graph's epistemics, not just its yield." The
survey independently converged: the three paper-shaped claims named in v3
(retrodiction, reopener triggers, calibrated briefs) are all
epistemics-testing; none are yield features. The instinct preceded the
literature.

**The nine-doc's blind spot was cost accounting.** Every proposal carried
a "graveyard risk," but only #4 and #7 seriously reckoned with
*author-side cost* — and Grudin says that is the only risk that has
historically killed anything [@grudin1996evaluating]. Re-scored on
cost/benefit posture:

| Posture | Improvements | Outcome |
| --- | --- | --- |
| Author-neutral (read-side) | #5, #8, #9 | All survive strong |
| Machine-paid | #2, #7 | Survive |
| Author-light | #3 | Survives with prerequisite |
| Author-taxed at write time | #1, #4, #6 | One killed, two deferred |

The correlation is perfect. The single sharpest lesson the research adds
to the improvements document: **the survival predictor was never novelty
or rigor — it was who pays.**
