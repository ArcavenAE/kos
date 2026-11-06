---
id: finding-kos-to4v
type: finding
confidence: frontier
title: "The graph has a probe field and never reads it, so a tested finding and a desk catalog look the same"
created_at: 2026-11-06
finding:
  probe: null
  result: partial
  evidence: mixed
---

# finding-kos-to4v: the graph has a probe field and never reads it, so a tested finding and a desk catalog look the same

**Bead.** aae-orc-37hm (P1). Siblings aae-orc-guy2 (the sweep) and aae-orc-ks34 (the method) follow this finding and are not part of it.

**Status.** MEASURED (code reading and file counts at kos origin/main 12cd440), plus a judged sample. By this finding's own rubric it is mixed: it measured, and it ran no brief with success signals. Its `finding:` block above is written in the shape this finding proposes, which kos does not read today.

**Provenance.** Researched by the research-supervisor role at the operator's request. The chair re-checked `src/findings.rs:346-355`, `src/findings.rs:561` and `src/promote.rs:11` at origin/main.

## Summary

1. The kos schema already has a field that separates tested from untested work: `finding.probe` / `finding.result`, plus `brief.success_signal`. kos never checks it. validate ignores it, and the default markdown shape drops it at parse.
2. A sample of 8 findings from the kos and marvel graphs splits 3 probe-run, 2 desk, 3 mixed. The probe field is readable on 3 of the 8 (one probe-run, one mixed, one desk), so today's field does not track the class.
3. No kos command reads anything that tells the three classes apart. `kos promote` says outright that evidence prompts are "not part of this command yet".
4. Recommended first step: have the markdown frontmatter decoder read the existing `finding:` block, and add one `evidence:` enum to it. validate gates only on a malformed value or an unresolvable probe, and reports a missing value.
5. A document-level label would not have caught marvel finding-007, which was probe-run. That class needs a claim-level "named alternative", as the second step.

## Question

What do the kos schema and tooling record today that could separate a tested finding from an untested one, and what is the smallest addition that would make the difference machine-readable?

## Method

- Read kos `origin/main` (after fetch): `schema/node.schema.yaml`, `src/model.rs`, `src/findings.rs`, `src/validate.rs`, `src/process.rs`, `src/reflect.rs`, `src/promote.rs`, `src/doctor.rs`.
- Ran `kos --help`, `kos finding --help` and `kos validate --help` on the installed build (alpha c4f93c2).
- Counted structured fields across all finding files on `origin/main` of two public graphs: kos (54) and marvel (87 md and yaml).
- The sample is 8 findings: marvel finding-007 (required), then evenly spaced picks by sorted filename, 4 from kos at positions n/5 to 4n/5 and 3 from marvel at n/4 to 3n/4. A private graph was also sampled; it is left out of this public record.
- Limits: the sample is small. It was classified by one reader from each file's opening sections and a keyword grep, not a full read.

The rubric:
- **probe-run:** names a probe, states its success signals and reports against them.
- **desk:** reading or cataloging only, with nothing executed.
- **mixed:** executed or reproduced something without a named probe and its signals, or puts probe-run claims beside desk claims.

A tooling note: the first count used `\s` in `git grep -E` and returned 0 everywhere, since that engine has no `\s` in extended mode. It was re-run with `[[:space:]]`.

## Results

### 1. What the schema records [measured]

- `type: finding` is defined as "Result of a probe. Evidence, not opinion." (`schema/node.schema.yaml:40`). The 37hm bar is already in that definition, as prose only.
- `confidence` is per node and epistemic (`:48-66`). There is no per-claim confidence.
- `brief.success_signal` is "What would confirm or disconfirm the hypothesis." (`:272-274`). Beside it sits `brief.predicted_confidence` (`:278-289`).
- `finding.probe` is "The brief that generated this finding." (`:311-313`).
- `finding.result` is one of confirms | disconfirms | partial | redirects (`:314-320`). `finding.surprise_magnitude` is at `:321-335`.
- The `derives` and `discovered_from` edges exist (`:93-121`). Neither means "tested by".
- The model mirrors the schema: `FindingSection { probe, result, surprise_magnitude }` (`src/model.rs:305-311`), loaded at `src/model.rs:31`.
- There is no `deny_unknown_fields` anywhere in `src/`, so unknown keys are dropped silently.

### 2. Where the field is lost [measured]

- **The markdown shape drops it.** The markdown frontmatter struct reads only `id, type, confidence, title, created_at, date, edges` (`src/findings.rs:346-355`), and `finding_node` hard-sets `finding: None` (`src/findings.rs:561`). `src/reflect.rs:703-705` states the consequence: "Markdown findings carry no `finding:` calibration block".
- **Markdown is the default shape.** `kos finding` defaults to `--md` (help text), and the markdown template emits no `finding:` block (`src/process.rs:212-222`).
- **The YAML template pre-fills a result.** It emits `probe: null` with `result: partial` (`src/process.rs:205-208`). A finding with no probe starts life carrying a result.

Field use across the fleet:

| graph | finding files | lines with `probe:` set | lines with `probe:` null |
|---|---|---|---|
| kos | 54 | 31 | 8 |
| marvel | 87 (84 md) | 4 | 0 |

All four marvel lines are top-level `probe:` keys, which neither decoder reads. In kos, 13 of the 31 set values are `brief-mvg-bootstrap`.

**Authors already write an evidence status, in prose kos does not parse.** Counts of the leading word on `Status:` lines:
- marvel: FRONTIER 6, MEASURED 5.

A kos test fixture uses the same convention (`src/findings.rs:779`, `**Status:** OBSERVED`).

### 3. The sample [measured counts; the classification is judgment]

| finding | class | evidence (one line) | `finding.probe` readable by kos |
|---|---|---|---|
| marvel 007 context-pressure | probe-run | SP1 to SP4 mirror `probe-context-pressure-extraction.md`; "Claude, Codex, and OpenCode were driven live" (:35) | no; frontmatter carries `question:` only, which is dropped |
| marvel 022 heartbeat binding | mixed | "The defect, confirmed on `main` before the fix" (:19); no probe | no |
| marvel 044 auto-shift | mixed | "Measured on f601b6e against a live interactive claude" (:41); no probe | no |
| marvel 066 identity line | desk | "I verified the claim against marvel origin/main b003cd3" (:4), by code reading | no |
| kos 010 Python PEPs | mixed | "Answer: PARTIAL" against brief-mvg-bootstrap (:14, :133); the kos-method claim was tested, the PEP-content claims are desk | yes |
| kos 021 executive empirical | probe-run | "Results against success criteria: 5/5 met." (:13) | yes, `result: confirms` (:252-253) |
| kos 032 rd-bridge | probe-run | "## Success signals (from brief-rd-bridge)" with YES per signal (:54-59) | **no**; no `finding:` block at all |
| kos 043 beads deep dive | desk | "Deep source-code analysis of gastownhall/beads" (:8) | yes, `probe: brief-seed-scan` (:473), whose brief is "Survey SDD systems" |

Counts: probe-run 3, desk 2, mixed 3. Of the 3 probe-run findings, 1 has a field kos reads. Of the 2 desk findings, 1 has the same field set. [measured]

So "probe set" fails in both directions:
- kos 032 is probe-run with no field.
- kos 043 is desk research with a field, and that field resolves to a desk-survey brief.
- kos 010 shows a brief whose success signals test the kos method on documents, while the PEP claims ride along under the same `result: partial`.

[inferred] Mixed ties for the largest class here. It is also the tier 37hm names as missing ("measured but not probe-verified"): the fleet produces it most and has no word for it.

### 4. What kos commands read [measured]

- **validate** checks, per node: filename matches id, confidence directory, edge targets known, edge type vocabulary, and a graveyard-section warning (`src/validate.rs:497-544`).
  - It has no finding-specific check. Its findings pass checks duplicate ids, id/stem drift and unloadable files only (`src/validate.rs:258-262`).
  - It walks `nodes/`, while briefs live in `probes/`, so a `finding.probe` reference would not resolve under it anyway.
- **doctor** skips edges to `finding-*` and `brief-*` targets without checking them (`src/doctor.rs:219-221`).
- **promote** runs validate's node checks on the move. Its header says: "Dissent and evidence prompts are not part of this command yet." (`src/promote.rs:11`)
- **reflect** is the only reader.
  - It uses `finding.probe` to pair predicted with actual confidence (`src/reflect.rs:748-756`, `:891-907`), and to mark a brief with no finding as in flight (`:797`).
  - It does not tell the classes apart.
  - A finding without the field is skipped from calibration silently (`:895-898`).
- **orient** lists findings by id, title and confidence (`src/orient.rs:1125`), with nothing about evidence.

[inferred] This answers the bead's last question with a yes. kos validates structural well-formedness and says nothing about epistemic status. The boundary between structural checks (which may gate) and vital signs (which only report) shows here as a hole, not an overreach: no structural check exists that could even report the class.

## Options, each tried against the sample and against finding-007

**A. `evidence: probe-run | desk | mixed` inside the `finding:` block.**
- Cost: one enum, plus decoder and template changes (`findings.rs:346`, `process.rs:212`). Authors already write this in prose (the Status lines).
- A value outside the vocabulary is structural and may gate. A missing value reports only, until the guy2 sweep backfills about 380 files; gating it on day one would fail almost every file.
- It is self-declared: finding-007 would honestly read `probe-run` and still carry the degenerate claim.

**B. Read `finding.probe` and require it to resolve to a brief that has a `success_signal`.**
- Cost: the decoder change, plus loading `probes/` into validate's id set.
- An unresolvable reference is structural and may gate; absence reports.
- kos 043 passes while being desk research, so B proves linkage, not testing. It is best used with A.

**C. A `candidate` node type for desk research.**
- Cost: a new type, against the schema's own "Type proliferation is a smell" (`schema:44-46`), plus migration and renderer changes.
- It also leaves the mixed class unnamed, and mixed is among the largest classes in the sample. Its cost is higher and its fit lower than A's.

**D. A validate warning when a finding has no probe link.**
- Cost: lowest. Report only, since a count of untested findings is a vital sign and must not gate.
- It fires on 5 of the 8 sampled findings, including 2 mixed ones, which are real measurements. That noise invites a wave-off.

**E. A named alternative per claim** (from the bead's SHARPENED note): `claims: [{text, alternative, discriminated: true|false}]`.
- It is the only option that would have failed finding-007's row, which had no alternative it could exclude.
- Cost: real authoring burden, so it should be optional at first. When present, its shape is structural and may gate.
- It answers the SHARPENED note, not 37hm's document-level question.

## Recommendation

Do A plus the decoder half of B as one kos change:
- teach `md_finding`'s `Frontmatter` to read the `finding:` block (`probe`, `result`, `surprise_magnitude`) and the new `evidence` key;
- emit the block from the markdown template;
- stop pre-filling `result: partial` when `probe` is null;
- validate gates on an unknown `evidence` value and on a `probe` that does not resolve to a brief, and reports a missing `evidence` value.

This makes the three classes machine-readable with no new type, using a vocabulary authors already write. E is the follow-on for the finding-007 subclass, and goes to the operator as a separate proposal, because it changes how claims are written, not only how they are labeled.

**First step:** a kos PR that changes `src/findings.rs:346-355` to deserialize `finding:` from markdown frontmatter, and adds `evidence` to `FindingSection` (`src/model.rs:305`). It carries tests for both the md and yaml shapes, and one reported warning.

**Sell-by:** valid until 2026-11-20. The research-supervisor seat re-checks it then, or sooner if aae-orc-ks34 or aae-orc-guy2 lands a classification method that conflicts with the three-value enum.

## What was not read

- The sampled findings in full; only their opening sections and grep hits.
- `src/charter.rs`, `src/ask.rs` and `src/graph.rs` beyond a grep for `probe` and `success_signal`, which returned no hits.
- The kos CI workflows (`graph-validate.yml`), so whether CI runs validate with warnings as failures is unchecked.
- Findings in other subrepo graphs (forestage, director, sideshow and others).
- Whether every brief named by `finding.probe` in kos exists in `probes/`. Only `brief-mvg-bootstrap`, `brief-seed-scan` and `brief-rd-bridge` were checked.
- The kos `_kos/` for an existing node on this question to add an edge to. `question-brief-as-prediction-record` was found but not read.
