# Brief: fact store Probe A, Dolt as a versioned EAV fact log (aae-orc-isie)

Status: DRAFT, written before any measurement. Lock it before the probe runs.
Date: 2026-10-20
Question node: question-kos-fact-store-substrate, sub-questions A, C and D

## Question

Can Dolt hold one repository's code facts at two commits, as an append-only
EAV log? It must give a commit-over-commit diff natively (AS OF), and let a
promoted subset be harvested into kos without carrying the bulk along.

## Decision criterion

Probe A passes when all four hold on marvel's measured scale (about 47k
candidate facts per commit):
1. **Load.** Both commits' facts load in under 60 s each, from a SCIP index to
   rows.
2. **Diff.** An AS OF query lists the facts added and removed between the two
   commits, and its counts equal a set difference computed outside Dolt.
3. **Harvest.** A chosen subset of 100 facts is cherry-picked to a branch and
   exported to kos YAML, and the bulk does not travel with it.
4. **Traverse.** After load, a two-to-three-hop traversal (callers of a
   function's callers) runs in Rust over the loaded rows, not in SQL, in under
   1 s.

If any fails, the finding names which one and by how much.

## The three MDE killers, answered before the probe runs (kos roadmap v3, M4)

- **Tool lock-in.** Dolt is one vendor's engine.
  - Answer to test: the whole fact log exports to plain JSONL and re-imports
    with identical counts and set equality.
  - Exit cost: one dump, measured.
  - Fleet context: bd already runs on a Dolt server here, so the operating
    cost is not new.
- **Round-trip breakage.** kos's source of truth stays YAML in git.
  - Answer to test: the harvested subset goes to YAML and back with semantic
    equality, the same facts and attributes.
  - Any loss is a fail.
- **Skill asymmetry.**
  - Answer to test: every query the probe needs is plain SQL, except the AS OF
    clause and `dolt_diff`.
  - Record each Dolt-specific verb used, and whether a fresh session wrote it
    from the brief alone.

## Method

- **Inputs:**
  - the 10-16 marvel index (ac67c078);
  - one fresh scip-go index at current marvel main.
- **Run conditions for the fresh index:**
  - built from a scratch clone, under isolated run conditions: a scratch
    HOME, no MARVEL_SOCKET, `sandbox-exec` localhost-only with the
    refused-connection control, and the sandbox write-deny proof;
  - `GOPROXY=off`.
- **Schema:** `facts(e, a, v, tx)`, one row per fact, attributes as rows.
- **Clock:** one Dolt repository in scratch, one server on a random localhost
  port, killed by its pid at the end and checked with `kill -0`.
- **Timebox:** one working day.

## Prediction (registered)

0.7 that the layered answer holds: Dolt carries the bulk, and kos takes only
promoted facts. The prediction is carried from isie and is the recommending
analysis's, not new.

## Falsifier

If a two-hop traversal cannot run in Rust without pulling the whole log into
memory, or the load exceeds 60 s per commit, the versioned-store-below shape
fails at this scale.

## Outputs

- A finding in kos `_kos/findings/`, with the id minted by
  `kos id finding <slug>`.
- The measured table, and the four criteria pass or fail.
- The MDE answers, as measured.
