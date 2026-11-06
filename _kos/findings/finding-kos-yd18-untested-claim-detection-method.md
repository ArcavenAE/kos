---
type: finding
confidence: frontier
title: "Detecting mislabeled findings and claims with no named alternative: mechanical checks find the class, judgement finds the degenerate claim"
created_at: 2026-11-06
edges:
  - type: supports
    target: finding-kos-to4v
finding:
  probe: null
  result: partial
  evidence: mixed
---

# finding-kos-yd18: detecting mislabeled findings and claims with no named alternative

**Bead.** aae-orc-ks34. Follows finding-kos-to4v (aae-orc-37hm). Feeds aae-orc-guy2, the sweep.

**Status.** MEASURED on a pilot of six public findings at kos `12cd440` and marvel `ba0ea65`. By finding-kos-to4v's rubric this is mixed: it piloted a method and named no brief with success signals.

**Provenance.** Researched by the research-supervisor role at the operator's request. The chair re-checked marvel finding-016 lines 276-281, the detached caveat behind M7, at origin/main.

## Summary

1. A 14-step method (10 mechanical checks, 4 judgement steps) was piloted on 6 public findings: 3 from kos and 3 from marvel.
2. Five of the six carry at least one load-bearing claim with no named alternative. Two are degenerate in the finding-007 sense: marvel finding-007 has two, and marvel finding-016 has one.
3. The mechanical checks got the document class right in 6 of 6 and pointed at the bad claim in 4 of 6. Spotting a degenerate or confounded claim still takes judgement.
4. The highest-yield mechanical check is caveat detachment. A claim's key token appears unqualified in the headline and again beside "cannot be separated", "unverified" or "suggestive" in a gaps section. It fires on marvel finding-016 and kos finding-kos-wre7.
5. Placement: a report-only kos subcommand runs the mechanical checks, and a kos repo skill holds the judgement steps. Only the field and reference checks may gate in `kos validate`.

## Question

What repeatable, agent-runnable method detects (1) a finding mislabeled at the document level, and (2) a load-bearing claim with no named alternative, cheap enough to run often?

## Method

For each finding the probe read the whole file at origin/main, the brief it names, the finding's `git log`, and grep hits for its id and key tokens across the repo. Each read was timestamped. Public graphs only.

The public corpus uses three conventions for probe links, which M1 must handle [measured, crude regex]:
- kos (54 findings): 2 bold-header probe lines, 3 frontmatter, 34 body-only brief mentions, 15 with no probe reference.
- marvel (87 findings): 11 bold-header, 22 frontmatter, 4 body-only, 50 none.

The frontmatter column overcounts. finding-007 matched on `question:`, which is not a probe link, and finding-047 matched on `**Probe:** incidental`. M1 must read the value, not just find the key.

## Pilot results

**marvel finding-007** (`marvel/_kos/findings/finding-007-context-pressure-extraction.md`)
- **Class:** probe-run against `_kos/probes/probe-context-pressure-extraction.md`, SP1 to SP4.
- **Signals:** artifact-shaped ("a filled table" at brief :62, "a recommended seam" at :88), so they could not fail on content [measured].
- **Claims:**
  - Claude's numerator as the three-way sum (:66-78, "This is decisive") names one alternative and excludes it (`input_tokens` alone). It does not name the second, cumulative total versus per-request level; every turn was a single-request `--resume`, so the two read the same. The addendum at :268-288 found this. Degenerate [measured].
  - opencode's total excluding cache (original :60, :91-95) names no alternative, and the row had both cache classes at zero. Degenerate; corrected at :97-132.
  - Codex `input_tokens` including cached tokens (:87-89) rests on one turn, with the additive layout as the implied alternative. Weak [inferred].
  - Shim not required (:183-193): the alternative is named, and the claim is reasoned [inferred].
  - Claude's displayed percentage (:197-211) comes from docs, not measurement, and its caveat comes later, at :217-221.
- **Blast radius:** 26 files mention finding-007 [measured].
- **Mechanical checks alone:** M4 (artifact-shaped signals) and M9 (spread into source) would have fired. Neither degenerate claim would have been caught.

**kos finding-032** (`kos/_kos/findings/finding-032-rd-bridge-probe.yaml`)
- **Class:** probe-run. The brief `brief-rd-bridge.yaml` has a threshold (:61) and a falsifier (:52).
- **Claims:**
  - "100% (48/48)" (:11) uses a denominator fixed in the brief (:61) independently of the regex, so it is not circular [measured].
  - Success signal 2 is graded "YES" (:57). The brief (:61-62) requires confidence among the extracted fields, while the finding's own F2 (:30-31) says confidence was captured for 10 of 48 (21%). It was graded against a weaker criterion than written [measured].
  - "avoids false negatives" (:39-40) has no alternative and no count.
- **Git:** committed with code in `f71239c`, a feat commit. M8 fires, but this was a build probe, so it is a false positive for the "written to rationalize" heuristic.
- **Blast radius:** 3 citing files.
- **Mechanical checks alone:** they flag the conflict between the reported number and the threshold. Calling it a misgrade takes reading.

**kos finding-131** (`kos/_kos/findings/finding-131-kos-node-ids-collide-across-graphs.md`)
- **Class:** mixed. No frontmatter, `**Status:** OBSERVED` (:4), a filename scan with no named probe, and a prior-art survey from reading only (:60-86). The self-label is honest.
- **Claims:**
  - The collisions exist (:20-27). The scan could have returned none, so this claim discriminates [measured].
  - "guarantees collisions" (:56-58) is inferred, with no alternative.
  - "exact-id collision is the detectable minority" (:156-165) rests on four examples; divergent-id duplicates were never counted, so "minority" is not measured. This claim shapes the design of a prevalence tool.
- **Blast radius:** 8 citing files.
- **Mechanical checks alone:** M1 and M6 (on "guarantees") fire. The "minority" claim is not caught.

**kos finding-kos-wre7** (`kos/_kos/findings/finding-kos-wre7-serena-live-lsp-baseline.md`)
- **Class:** probe-run. The brief was locked before install (:7), the prediction registered (:27), and the finding flags its own inflated score (:34-47). The best-behaved of the six.
- **Claims:**
  - W7 fails with a hard error (:28-32). This claim discriminates.
  - "Cold latency tracks the dependency graph, not line count" (:121-123). The two repos also differ in language and language server, so the claim is confounded and names no alternative.
  - "Rust index does not survive a restart" (:127-133) rests on one pair of runs (7.22 s against 6.91 s) under load 4.83 to 9.06 (:321). The finding calls latency its least trustworthy measure (:327-330), but that caveat sits away from the claim. M7 fires.
  - The native client still pays rust-analyzer's cost (:263-270): the PID-attributed split was measured, and the native client itself is inferred.
- **Reproduction:** the scripts lived outside the tree (:339-343). M10 fires.
- **Blast radius:** 6 citing files.
- **Mechanical checks alone:** M7 and M10 fire. The confound needs judgement.

**marvel finding-016** (`marvel/_kos/findings/finding-016-effective-autocompact-window-is-the-predictive-denominator.md`)
- **Class:** probe-run with pre-registered falsifiers (:4-5, :219). It retracts a wrong draft claim (:12-38), which puts a negative result on the record.
- **Claims:**
  - "93.5% of the effective window" (:48) and "both compacting at about 93.5% of whatever window" (:58-62). All 63 events ran at one setting, and at one setting "a fraction of the window" and "the window minus a ~32k constant" are the same arithmetic. The finding says so itself (:276-281), but in its gaps section, not at the headline. Degenerate, caveat detached [measured].
  - The claim has spread to `_kos/nodes/frontier/question-interactive-context-pressure.yaml:128`, `question-stream-attachment.yaml:137` and `_kos/probes/probe-context-operating-points-and-axes.md:33` [measured].
  - The model-change bullets (:125-138) are vendor claims from reading only.
- **Blast radius:** 33 citing files, the largest of the six.
- **Mechanical checks alone:** M7 and M9 fire, and M7 points straight at the degenerate claim.

**marvel finding-047** (`marvel/_kos/findings/finding-047-flaky-bus-test-port-offset-overflow.md`)
- **Class:** mixed. `**Probe:** incidental` (:4); the failure was reproduced (:8).
- **Claims:**
  - "macOS hands out ephemeral ports SEQUENTIALLY" (:25-29) is an OS-behavior claim the finding does not test. Eight binds on one host returned 61406 through 61413 in order, so it holds there [measured]. The 12-of-12 failures over four runs (:31-32) also argue against uniform random picks (about 0.24 per bind) [inferred], but the finding does not make that argument.
  - The claim now sits verbatim in a source comment, `internal/bus/supervisor_test.go:40-41`.
- **Git:** landed in `9896e61`, whose subject is an unrelated design document. M8 flags the bundling.
- **Blast radius:** 8 citing files.
- **Mechanical checks alone:** M1 (on "incidental") and M9 fire. Whether the claim is true takes one execution.

**Time.** All six were read and judged within about 90 seconds of one agent's tool wall clock, plus side checks [measured, coarse]. That is under a minute per finding, and most of it is judgement. A human reviewer would take longer.

## The method as steps

Inputs: the finding, the brief it names, its edges, `git log --follow` of the file, and repo-wide grep.

Mechanical (parse or grep):
- **M1 Class field.** Read `finding.evidence` from frontmatter, else the bold-header `Probe`, `Status` or `Confidence` line. Read the value: "incidental", "none" or no probe means not probe-run.
- **M2 Probe link.** The named brief exists at origin/main in the same graph.
- **M3 Signals reported.** Each signal id in the brief (SPn, Wn, numbered signals, falsifiers) appears in the finding with a verdict.
- **M4 Signal shape.** Flag a success signal with no number, comparator, falsifier or "if ... then" clause. Such a signal is artifact-shaped and cannot fail on content.
- **M5 Grading against the literal signal.** Extract the numbers next to a "YES" verdict and compare them with the brief's threshold and field list.
- **M6 Lexicon.** Flag "decisive", "guarantees", "always", "exactly" and "confirmed" in summary or verdict sentences. Also flag the hedges "appears", "likely", "should" and "seems" in sentences that M9 finds quoted elsewhere.
- **M7 Caveat detachment.** A key token (a number or identifier) appears unqualified in the summary or verdict, and again within a few lines of "cannot be separated", "unverified", "suggestive", "not established", "inferred" or "single" in another section.
- **M8 Git ordering.** The finding was added in the same commit as code it justifies, or in a commit whose subject does not name it. Exempt briefs whose hypothesis is "build X".
- **M9 Spread.** Grep the claim's key tokens outside the finding (nodes, probes, source comments) and rank by count. This is the blast-radius score for triage.
- **M10 Reproduction.** Referenced artifacts exist in the tree.

Judgement:
- **J1.** List the load-bearing claims: the headline, the verdict, and anything M9 found elsewhere.
- **J2.** For each claim, name the alternative and ask whether the data would have looked different under it. If not, the claim is degenerate. If two variables move together, it is confounded.
- **J3.** For a vendor, harness or OS claim, ask whether it was executed or only read. Where one command settles it, run it.
- **J4.** Assign probe-run, desk or mixed. Record each claim as discriminating, unnamed alternative, degenerate or confounded.

## Placement

- **Both.** A kos subcommand runs M1 to M10 over a finding or a graph and emits a per-finding report. J1 to J4 live in a kos repo skill that consumes that report. The name is the operator's to pick.
- **May gate in `kos validate`, as structural checks:**
  - `evidence` is in the enum when present;
  - `evidence: probe-run` names a brief that resolves;
  - that brief carries `success_signal`.
  Making `evidence` required is a schema change and needs a ruling; until then it is a warning.
- **Report only, never gate:** M3 to M10, the class-mismatch flag and the blast-radius score. They are vital signs about soundness, not well-formedness.

## Recommendation

First step: ship M7 and M9 as a report-only kos subcommand on the existing finding loader. Run it across marvel's and kos's public findings, ranked by citer count, as the seed list for aae-orc-guy2. M7 caught both degenerate-or-confounded cases with a detached caveat, M9 gives the triage order, and neither depends on finding-kos-to4v's frontmatter fix.

## Sell-by

Valid until 2026-11-20. The research-supervisor seat re-checks it then, against kos origin/main and the state of aae-orc-37hm.

## Not read

- PR review threads.
- Any finding beyond the six.
- Whether the claim-site wording behind 37hm's account of finding-007 would have tripped M7. That text predates the correction and was not re-read from history.
