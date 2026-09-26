# KOS Roadmap v3 — Literature-Hardened Edition

> Supersedes roadmap v2. Integrates three inputs: (1) the v2 primary-source
> assessment of the repo, (2) the nine beyond-vision improvements, (3) the
> literature survey's failure-mode findings across design rationale,
> traceability, KM, MDE, Semantic Web, and lessons-learned systems.
>
> Organizing principle, drawn from the survey's central lesson: **every
> knowledge-capture system in the literature died from the same wound —
> asymmetric cost/benefit [@grudin1996evaluating]. The people who pay the
> capture cost are not the people who reap the retrieval benefit.** KOS
> survives only if every milestone either reduces capture cost, increases
> retrieval benefit, or moves benefit to the moment of capture. Items that
> do none of these are cut or deferred, regardless of intellectual appeal.

---

## Part 0 — What the literature changed

Five findings that reshaped the roadmap's structure:

**L1. The Grudin asymmetry is the master risk.** Design rationale capture
(gIBIS/QOC) failed on intrusiveness and deferred benefit
[@grudin1996evaluating; @shipman1997design]. Traceability fails the same
way — links decay because maintainers pay and downstream readers benefit
[@gotel1994analysis; @arkley2005overcoming]. KM initiatives died on
incentive misalignment [@walsh1991organizational]. **Roadmap consequence:**
every issue below is tagged with its cost/benefit posture; a new Milestone
R (Retrieval-first) is promoted ahead of most capture discipline.

**L2. Reflexion models, not full traceability.** Murphy & Notkin's
Reflexion approach succeeded where DOORS-class traceability failed by
checking a *lightweight declared model against extracted reality* and
reporting only divergences [@murphy1995software]. This is exactly what
`kos drift` + seam detection already do. **Consequence:** the
correspondence layer ambition is re-scoped from "maintain typed spec↔code
links" (the DOORS death march) to "declare little, extract much, report
divergence" — cheaper and validated by prior success.

**L3. The graveyard is the genuinely novel contribution.** Lessons-learned
systems failed on retrieval-at-need, not on capture — NASA's LLIS
accumulated entries nobody consulted at decision time [@weber2001intelligent].
No prior system combines append-only negative knowledge with typed
*reopener* conditions checked mechanically. **Consequence:** graveyard
research is elevated from hygiene chore to the project's flagship claim,
with reopener-triggering as the differentiator (Issue 24).

**L4. Confidence half-life has an existing formalization.** ACT-R
base-level activation already models memory decay with retrieval-frequency
reinforcement [@anderson1991reflections]. Don't invent decay curves —
adopt the rational-analysis form. **Consequence:** Improvement #3 gains a
concrete mechanism and loses its "arbitrary horizons" risk.

**L5. Goodhart applies to KOS's own metrics.** Measurement dysfunction
literature [@austin1996measuring] predicts that node counts, finding
counts, and calibration scores will be gamed the moment they carry status.
**Consequence:** all process metrics become *diagnostic* (visible in
reports, never in gates), and canary detection-rate becomes the only
health metric that resists gaming — you can't game detection of
corruption you don't know exists.

---

## Part 1 — Milestone structure (revised)

```
M0  Repair & Truth          (unchanged from v2, 3 issues)
M1  Enforcement Inversion   (unchanged core, +instrument tests)
MR  Retrieval-First         (NEW — promoted above old M2; Grudin's cure)
M2  Query & Context         (absorbed into MR where overlapping)
M3  Science                 (rebuilt around literature-validated methods)
M4  Substrate               (gated, unchanged)
M5  Mission & Positioning   (expanded: publishable claims identified)
```

The single largest change: **retrieval before discipline.** v2 sequenced
gates → repairs → queries. The literature says systems die when capture
discipline arrives before retrieval value [@grudin1996evaluating;
@marshall2003semantic]. Gates stay early only where they are *zero-cost to
authors* (CI validation catches what authors never see); everything that
taxes the author at write time moves after retrieval proves its worth.

---

## Part 2 — Issues

### M0 — Repair & Truth (v2 issues 1, 2, 19 stand)

Unchanged. Schema drift repair (observation edges, graveyard type
conflation, dangling founding refs), charter-content classification, and
the README truth pass. One addition to Issue 1:

**Issue 1 addendum — rename maps become migration infrastructure.**
Ontology-evolution literature shows identity churn is the norm, not an
incident [@noy2004ontology]. The one-time rename map becomes
`_kos/migrations/` with a `kos migrate` stub — schema v0.4 ships with the
migration path it will need, not after the next breakage.
*Cost/benefit: author-neutral, system-paid.*

---

### M1 — Enforcement Inversion (v2 issues 4, 18 stand; 5, 6 modified)

**Issue 4 (unchanged, still first):** full-scope validate, CI job,
pre-commit hook, dangling targets = error. This is Grudin-safe: the
machine pays, authors only see failures they caused.
*Cost/benefit: author-neutral.*

**Issue 18 (unchanged):** test the instrument (validate/model/drift/charter
≥20 tests). The validator must be more trustworthy than the graph it
judges.

**Issue 5 (modified — measure only):** context budget accounting ships;
the *pruning* of imports waits for MR telemetry (Issue 20) to show what's
actually read. Cutting before measuring repeats the KM pattern of
optimizing unobserved behavior.

**Issue 6 (modified — softened per Goodhart):** the artifact-close rule
becomes a *dashboard signal* (sessions closed with/without artifacts,
visible in `kos doctor`), not a hard gate. Austin's measurement-dysfunction
finding [@austin1996measuring]: gate it and sessions will produce
token commits to satisfy the gate. Surface it and let the human pawl
judge. Two consecutive analysis-only sessions still trigger the mandatory
graveyard review — that part is judgment-preserving.

---

### MR — Retrieval-First (NEW milestone; the Grudin cure)

The survey's verdict: capture systems survive only when retrieval benefit
arrives *before* capture discipline is demanded. Everything here increases
what the graph gives back per token of authoring already spent.

**Issue 20: Retrieval telemetry (Improvement #7, promoted).**
`_kos/.telemetry/reads.jsonl` logging node IDs served by orient/query/
doctor. Ten-session baseline. Hypothesis pre-registered: retrieval follows
a power law; ≥40% of nodes never read post-creation. This is the
instrument every later cost/benefit decision depends on — the metric
Walsh & Ungson's organizational-memory program never had
[@walsh1991organizational].
*Cost/benefit: author-neutral; produces the benefit ledger.*
**Deeper-treatment flag from survey honored: this is the empirical core.**

**Issue 21: Subgraph context format + query commands (v2 issues 7+8
merged).** Unchanged in content; re-justified by the survey: this is the
"retrieval at the moment of need" that LLIS lacked [@weber2001intelligent].
Format spec first, three hand-built ThreeDoors examples, then
frontier/bedrock/recent commands emitting it.

**Issue 22: Task-conditional orient (v2 issue 9).** Unchanged. The
empirical test of charter-as-cache. Token cost measured against full
charter load; either outcome is a finding.

**Issue 23: MCP server (Improvement #8, promoted into MR).** Read-only
epistemic API serving the Issue-21 format. Rationale upgraded by the
survey: mixed-initiative literature [@horvitz1999principles] and the
platform's own F10 failure mode both say the benefit must arrive *inside
the agent's task loop*, not in a separate consultation step. Gated on 21.
*Cost/benefit: pure retrieval-side; zero author tax.*

**Issue 24: Reopener triggers — the graveyard's differentiator (NEW).**
Graveyard reopeners are currently prose. Make them machine-checkable:
structured `trigger` conditions (dependency-hash change, date horizon,
external-fact assertion) evaluated by `kos stale`. When a trigger fires,
the graveyard entry surfaces in orient output. This is the feature no
lessons-learned system ever shipped — negative knowledge that *re-presents
itself at the moment it becomes relevant* [@weber2001intelligent]. Flagged
by the survey as the project's most publishable novelty; treat
accordingly.
*Cost/benefit: small structured-authoring cost at graveyard time (already
the cheapest moment — the finding is fresh), large deferred retrieval
benefit delivered automatically.*

---

### M3 — Science (rebuilt)

Ordering within M3 now follows credibility-per-effort from the survey.

**Issue 25: Retrodiction engine (Improvement #5, promoted to first
science item).** `kos asof <ref>` + historical seam replay on the four
ThreeDoors incidents, operator blinded to incident selection, prediction
registered before running (≥2 of 4 detectable at T-minus-one-sprint).
Survey hardening: adopt backtesting hygiene against look-ahead bias
[@bailey2014pseudo] and hindsight-bias controls from accident-investigation
methodology [@fischhoff1975hindsight; @dekker2002field] — the blinding
protocol is not optional, it is the experiment.
*This is the project's highest-credibility experiment and its best paper.*

**Issue 10 (v2, corrected premise stands): calibration pipeline.**
Survey addition: adopt Brier scoring and the forecasting-literature
apparatus wholesale [@tetlock2015superforecasting; @brier1950verification]
rather than inventing surprise taxonomies. `surprise_magnitude` maps to
calibration-curve residuals. Pre-registration framing now cites the
registered-reports model [@nosek2014registered] — briefs *are* registered
reports for engineering knowledge; say so in the docs and inherit the
discipline's norms (no post-hoc hypothesis editing, `post_hoc: true` flag
for probe-null findings, HARKing named as the failure mode [@kerr1998harking]).

**Issue 11 (v2 stands): controlled baseline** — naive LLM-over-docs vs.
graph, finding-023 rubric, two projects. Unchanged.

**Issue 17 (v2 stands, sharpened): typed-edge thesis test.** Survey
sharpening: the 92%-derives monoculture mirrors the Semantic Web's
ontology-cost failure — rich taxonomies go unused because authors default
to the cheapest link [@marshall2003semantic; @shipman1999formality].
The probe now tests Shipman & Marshall's "formality tax" hypothesis
directly: retype 50 edges, measure traversal value, and *also* measure
authoring cost per typed edge. If value < cost, prune the taxonomy to
derives/contradicts/supersedes and treat that as confirmation of the
incremental-formalization literature, not defeat.

**Issue 26: Canary nodes (Improvement #2, admitted to M3).**
Epistemic mutation testing [@demillo1978hints] + chaos-engineering framing
[@basiri2016chaos]. Three canaries across tiers, sealed hash-committed
registration, five normal sessions, reveal. Goodhart-resistant health
metric per L5. Hawthorne mitigation: random delays, low base rate.

**Issue 12, 13, 14 (v2 stand):** inter-rater reliability (Cohen's kappa on
30-finding sample), external-operator probe (now trivially runnable via
Issue 23's MCP endpoint — dependency updated), scheduled graveyard audit
every 3 sessions with the 10-session sunset rule.

**Deferred from the nine improvements, with reasons:**
- *Source Reliability Ledger (#1):* n-of-sources too small until the
  platform has more operators; also Goodhart-exposed (grades become
  status). Revisit when ≥4 distinct sustained sources exist. The Admiralty
  two-axis insight (source vs. information) is adopted *conceptually* now:
  provenance and confidence stay separate fields, never merged.
- *Confidence half-life (#3):* adopt ACT-R base-level form per L4, but
  implement only after Issue 20's telemetry exists — decay reinforced by
  retrieval requires retrieval counts. Sequenced behind MR by construction.
- *Dissent nodes (#6):* premature at one human; parked with a trigger —
  reopens when a second sustained human operator joins.
- *Toulmin fields (#4):* survey warning is direct — argumentation
  formalisms are the canonical Grudin victim [@shipman1997design;
  @conklin1988gibis]. Admit only the single cheapest field:
  `rebuttal_conditions` on bedrock (it doubles as Issue 24's trigger
  vocabulary). Grounds/warrant structure stays graveyarded until someone
  demonstrates demand via telemetry.
- *Chronicle (#9):* admitted as a *projection* (read-side, Grudin-safe)
  but sequenced after Issue 21's traversal machinery, which it reuses.
  Becomes Issue 27, M5-adjacent, paired with the external-operator probe
  as its comprehension benchmark.

---

### M4 — Substrate (v2 issue 15 stands, gated on M0–MR)

One survey addition: the MDE failure literature [@whittle2014state;
@hutchinson2011empirical] is the checklist for any substrate migration —
tool lock-in, round-trip breakage, and skill asymmetry killed MDE in
industry. Each substrate probe's brief must state its answer to all three
before the probe runs.

---

### M5 — Mission & Positioning (expanded)

**Issue 16 (v2 stands):** mission split; graveyard
continuity-via-infrastructure with model-level reopener.

**Issue 28: Publishable claims register (NEW).** The survey identifies
where KOS sits: CSCW/CHI (human-AI knowledge work), ICSE/FSE (drift and
seam detection), and the emerging LLM-agent-memory literature. Three
claims are paper-shaped once their issues close:
(a) *Retrodictive seam detection* — Issue 25's blinded replay result;
(b) *Reopener-triggered negative knowledge* — Issue 24, positioned against
the lessons-learned failure literature;
(c) *Calibrated engineering hypotheses* — Issue 10 at n≥30, positioned as
registered reports for design decisions.
The register tracks each claim's evidentiary status; nothing is drafted
until its evidence closes. This converts "improve rigor" from aspiration
into named, falsifiable, externally-judgeable outputs.

---

## Part 3 — Sequencing

```
n:    Issue 4 (gate on) ── failures enumerate the M0 worklist
n+1:  Issues 1(+migrations), 19
n+2:  Issues 2, 3 (charter classification, cutover)
n+3:  Issues 18, 5(measure-only), 20 (telemetry starts NOW — 10-session clock)
n+4:  Issue 21 (format + queries)
n+5:  Issues 22, 24 (task-orient; reopener triggers)
n+6:  Issue 23 (MCP), Issue 14 first graveyard audit
n+7:  Issue 25 (retrodiction — the flagship experiment)
then: 10, 11, 17, 26 as the science block · 12, 13 · 27 · 15 gated · 16, 28
```

Standing rules, now three:
1. **Each issue closes with a commit** (unchanged).
2. **No new author-side capture requirement ships before MR demonstrates
   retrieval value** — the Grudin rule, enforced by milestone ordering.
3. **Process metrics are diagnostic, never gates** — the Goodhart rule.
   The only gated checks are machine-payable (schema, hashes, dangling
   refs).

---

## Part 4 — What v3 removes from consideration entirely

Named cuts, with the literature that killed them:

- **Full spec↔code traceability matrices** — DOORS-class maintenance decay
  [@gotel1994analysis; @arkley2005overcoming]. Reflexion-style divergence
  reporting only [@murphy1995software].
- **Rich argumentation ontologies** — gIBIS/QOC intrusiveness
  [@conklin1988gibis; @shipman1997design]. One field survives
  (`rebuttal_conditions`).
- **Comprehensive upfront ontology work on edge types** — Semantic Web
  authoring-cost failure [@marshall2003semantic]. Taxonomy earns each type
  via Issue 17 or loses it.
- **Auto-summarization of the charter or nodes** — "industrialized
  forgetting"; also the round-trip failure MDE documented
  [@hutchinson2011empirical]. Projections are authored or deterministic,
  never lossy-automatic.
- **Gamified process metrics** (node counts, finding velocity, coverage
  percentages as targets) — Goodhart/Austin [@austin1996measuring].

---

## Bibliography

```bibtex
@article{grudin1996evaluating,
  author = {Grudin, Jonathan},
  title = {Evaluating Opportunities for Design Capture},
  journal = {Design Rationale: Concepts, Techniques, and Use},
  publisher = {Lawrence Erlbaum},
  year = {1996},
  pages = {453--470}
}
@article{shipman1997design,
  author = {Shipman, Frank M. and McCall, Raymond J.},
  title = {Integrating Different Perspectives on Design Rationale: Supporting the Emergence of Design Rationale from Design Communication},
  journal = {Artificial Intelligence for Engineering Design, Analysis and Manufacturing},
  volume = {11},
  number = {2},
  year = {1997},
  pages = {141--154}
}
@article{gotel1994analysis,
  author = {Gotel, Orlena C. Z. and Finkelstein, Anthony C. W.},
  title = {An Analysis of the Requirements Traceability Problem},
  journal = {Proceedings of the First International Conference on Requirements Engineering},
  year = {1994},
  pages = {94--101}
}
@inproceedings{arkley2005overcoming,
  author = {Arkley, Paul and Riddle, Steve},
  title = {Overcoming the Traceability Benefit Problem},
  booktitle = {13th IEEE International Conference on Requirements Engineering},
  year = {2005},
  pages = {385--389}
}
@article{murphy1995software,
  author = {Murphy, Gail C. and Notkin, David and Sullivan, Kevin},
  title = {Software Reflexion Models: Bridging the Gap between Source and High-Level Models},
  journal = {ACM SIGSOFT Software Engineering Notes},
  volume = {20},
  number = {4},
  year = {1995},
  pages = {18--28}
}
@article{walsh1991organizational,
  author = {Walsh, James P. and Ungson, Gerardo Rivera},
  title = {Organizational Memory},
  journal = {Academy of Management Review},
  volume = {16},
  number = {1},
  year = {1991},
  pages = {57--91}
}
@article{weber2001intelligent,
  author = {Weber, Rosina and Aha, David W. and Becerra-Fernandez, Irma},
  title = {Intelligent Lessons Learned Systems},
  journal = {Expert Systems with Applications},
  volume = {20},
  number = {1},
  year = {2001},
  pages = {17--34}
}
@article{anderson1991reflections,
  author = {Anderson, John R. and Schooler, Lael J.},
  title = {Reflections of the Environment in Memory},
  journal = {Psychological Science},
  volume = {2},
  number = {6},
  year = {1991},
  pages = {396--408}
}
@book{austin1996measuring,
  author = {Austin, Robert D.},
  title = {Measuring and Managing Performance in Organizations},
  publisher = {Dorset House},
  year = {1996}
}
@inproceedings{horvitz1999principles,
  author = {Horvitz, Eric},
  title = {Principles of Mixed-Initiative User Interfaces},
  booktitle = {Proceedings of the SIGCHI Conference on Human Factors in Computing Systems},
  year = {1999},
  pages = {159--166}
}
@article{marshall2003semantic,
  author = {Marshall, Catherine C. and Shipman, Frank M.},
  title = {Which Semantic Web?},
  journal = {Proceedings of the Fourteenth ACM Conference on Hypertext and Hypermedia},
  year = {2003},
  pages = {57--66}
}
@article{shipman1999formality,
  author = {Shipman, Frank M. and Marshall, Catherine C.},
  title = {Formality Considered Harmful: Experiences, Emerging Themes, and Directions on the Use of Formal Representations in Interactive Systems},
  journal = {Computer Supported Cooperative Work},
  volume = {8},
  number = {4},
  year = {1999},
  pages = {333--352}
}
@inproceedings{conklin1988gibis,
  author = {Conklin, Jeff and Begeman, Michael L.},
  title = {gIBIS: A Hypertext Tool for Exploratory Policy Discussion},
  booktitle = {Proceedings of the 1988 ACM Conference on Computer-Supported Cooperative Work},
  year = {1988},
  pages = {140--152}
}
@article{fischhoff1975hindsight,
  author = {Fischhoff, Baruch},
  title = {Hindsight Is Not Equal to Foresight: The Effect of Outcome Knowledge on Judgment under Uncertainty},
  journal = {Journal of Experimental Psychology: Human Perception and Performance},
  volume = {1},
  number = {3},
  year = {1975},
  pages = {288--299}
}
@book{dekker2002field,
  author = {Dekker, Sidney},
  title = {The Field Guide to Human Error Investigations},
  publisher = {Ashgate},
  year = {2002}
}
@article{bailey2014pseudo,
  author = {Bailey, David H. and Borwein, Jonathan M. and L{\'o}pez de Prado, Marcos and Zhu, Qiji Jim},
  title = {Pseudo-Mathematics and Financial Charlatanism: The Effects of Backtest Overfitting on Out-of-Sample Performance},
  journal = {Notices of the AMS},
  volume = {61},
  number = {5},
  year = {2014},
  pages = {458--471}
}
@book{tetlock2015superforecasting,
  author = {Tetlock, Philip E. and Gardner, Dan},
  title = {Superforecasting: The Art and Science of Prediction},
  publisher = {Crown},
  year = {2015}
}
@article{brier1950verification,
  author = {Brier, Glenn W.},
  title = {Verification of Forecasts Expressed in Terms of Probability},
  journal = {Monthly Weather Review},
  volume = {78},
  number = {1},
  year = {1950},
  pages = {1--3}
}
@article{nosek2014registered,
  author = {Nosek, Brian A. and Lakens, Dani{\"e}l},
  title = {Registered Reports: A Method to Increase the Credibility of Published Results},
  journal = {Social Psychology},
  volume = {45},
  number = {3},
  year = {2014},
  pages = {137--141}
}
@article{kerr1998harking,
  author = {Kerr, Norbert L.},
  title = {HARKing: Hypothesizing After the Results Are Known},
  journal = {Personality and Social Psychology Review},
  volume = {2},
  number = {3},
  year = {1998},
  pages = {196--217}
}
@article{demillo1978hints,
  author = {DeMillo, Richard A. and Lipton, Richard J. and Sayward, Frederick G.},
  title = {Hints on Test Data Selection: Help for the Practicing Programmer},
  journal = {Computer},
  volume = {11},
  number = {4},
  year = {1978},
  pages = {34--41}
}
@article{basiri2016chaos,
  author = {Basiri, Ali and Behnam, Niosha and de Rooij, Ruud and Hochstein, Lorin and Kosewski, Luke and Reynolds, Justin and Rosenthal, Casey},
  title = {Chaos Engineering},
  journal = {IEEE Software},
  volume = {33},
  number = {3},
  year = {2016},
  pages = {35--41}
}
@article{whittle2014state,
  author = {Whittle, Jon and Hutchinson, John and Rouncefield, Mark},
  title = {The State of Practice in Model-Driven Engineering},
  journal = {IEEE Software},
  volume = {31},
  number = {3},
  year = {2014},
  pages = {79--85}
}
@inproceedings{hutchinson2011empirical,
  author = {Hutchinson, John and Whittle, Jon and Rouncefield, Mark and Kristoffersen, Steinar},
  title = {Empirical Assessment of MDE in Industry},
  booktitle = {Proceedings of the 33rd International Conference on Software Engineering},
  year = {2011},
  pages = {471--480}
}
@article{noy2004ontology,
  author = {Noy, Natalya F. and Klein, Michel},
  title = {Ontology Evolution: Not the Same as Schema Evolution},
  journal = {Knowledge and Information Systems},
  volume = {6},
  number = {4},
  year = {2004},
  pages = {428--440}
}
```

---

*v3 verdict in one line: the literature's graveyard is now KOS's map —
retrieval before discipline, divergence-reporting before traceability,
machine-payable gates only, and three named experiments (retrodiction,
reopener triggers, calibrated briefs) that no prior system ran and this
one can.*
