# Brief: fact store Probe B, SQLite fact log with covering indexes (aae-orc-isie)

Status: DRAFT, written before any measurement. Lock it before the probe runs.
Date: 2026-10-20
Question node: question-kos-fact-store-substrate, sub-questions B and C

## Question

Does an SQLite file, with an append-only `facts(e, a, v, tx)` table and four
covering indexes (EAVT, AEVT, AVET, VAET, the Datomic layout the kos charter
names as its ideal at KOS-charter.md:744), do Probe A's job? The job is
holding one repository's facts at two commits, a commit-over-commit diff, a
harvested subset and a fast traversal, built inside one session.

## Decision criterion

The same four criteria as Probe A (load, diff, harvest, traverse), with the
same thresholds, plus a fifth:
- **Build cost.** The whole thing, schema and loader, fits in one session's
  timebox.

Time travel is the `tx` column, not engine versioning, so the diff is a query
over `tx`.

## The three MDE killers, answered before the probe runs

- **Tool lock-in.** SQLite is public domain and ships everywhere.
  - Answer to test: the same JSONL export and re-import as Probe A.
- **Round-trip breakage.** The same YAML round trip as Probe A.
- **Skill asymmetry.**
  - Every query is plain SQL with no engine-specific verbs.
  - Record any recursive CTE the probe needed. isie names those as the cost if
    traversal ever moves into SQL.

## Method

- **Inputs:** the same two indexes as Probe A, under the same run conditions.
- **Store:** one SQLite file in scratch; no server.
- **Timebox:** one session, as isie states. If the timebox runs out, that is
  recorded as a result, not an extension.

## What it adds over Probe A

What the Dolt engine buys:
- branching;
- cell-level merge;
- native AS OF.

Probe B prices these against a file with a `tx` column. If B passes all five
criteria, Dolt has to earn its server with the harvest-by-branch step alone.

## Prediction

None registered beyond Probe A's. A prediction added now would be made after
seeing the fact counts, so it is left out.

## Outputs

A finding in kos `_kos/findings/`, minted with `kos id finding <slug>`, or
one finding covering both probes, written after both run.
