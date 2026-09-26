# kos-read-path-gap

*Criticism received and accepted: kos seeks to encode knowledge, but
query/reading is non-existent. Cache writes are expensive — and more
so if you never read. Captured 2026-07-15; an area Michael was
encouraged to work on, now grounded by the organizing-inference thesis
(the library institution: judged by circulation, not acquisition) and
by two same-day pieces of evidence.*

Captured: 2026-07-15
Source: external criticism relayed by Michael + session evidence.
Tags: [atelier]
Related: F19 (active surfacing — the PUSH half; this is the PULL
half), F25 (stage rig / backdrops), F10 (cross-repo continuity),
flyloft charter F2 (the designed retrieval substrate),
finding-044 + `bd-orient-backdrop-missing.md` (s045 — the empirical
failure), `organizing-inference-at-scale.md` (the crosswalk method
needs this to run against our own record), `.claude/rules/agent-tools.md`
(aq — the embryonic read path, see direction 2).

## The criticism, precisely

kos's verb set is almost entirely producer-side: `idea`, `question`,
`probe`, `finding` (scaffolds), `harvest`/`promote` (conventions),
`validate`, `drift`, `reflect`, `compact`, `charter render` — all
write-path or write-hygiene. The read path is `kos orient` (bulk
per-cwd dump at session start) and nothing else. There is:

- no query verb ("what do we know / what did we decide / what did we
  rule out about X?")
- no ranked or scoped retrieval into a working context mid-session
- no read telemetry — nothing records whether a node has EVER been
  consulted

A knowledge system with writes and no reads is a warehouse, not a
library. Worse than zero ROI: unread knowledge still bills —
maintenance, drift management, schema migrations, and orient-dump
context tokens are all paid on the write side regardless of whether
any read ever occurs.

## Evidence (all local, two from today)

1. **s045 (finding-044):** four architectural pivots in one session;
   every prop needed existed in the graph; nothing surfaced them. F25
   filed it as "backdrops missing." This file names the complementary
   read: even with backdrops, there was no verb the agent could have
   used to ASK.
2. **Today, the orient bug nobody noticed:** `kos orient` in marvel
   has been silently skipping 14 of its node files on schema drift
   (unknown edge types). Nobody noticed the reads were broken — the
   defining tell of a write-only system. (Broken reads on a read-heavy
   system page someone in minutes.)
3. **Today, the synthesis path:** the vsdd→marvel→thesis arc worked
   because a human curated the reading list (issues, review corpus,
   code). "What do we know about marvel's resource model?" was
   unanswerable by any kos verb, though the answer existed in
   fragments across five documents.

## The institutional frame

Libraries are judged by circulation. Citation is the academy's read
telemetry — an artifact's authority is a count of *reads that
mattered*. A records office that only files is dead weight; the
circulation desk is what makes the archive an asset. (See the library
row in organizing-inference-at-scale.md's checklist.)

## Candidate directions (pre-hypothesis, cheapest first)

1. **Read telemetry via the existing wrappers.** `aq finding/node/
   idea/charter` are already the human-and-agent read path — one
   logline each and the graph gets circulation counts for ~free.
   Downstream: a "cold nodes" report (never-read bedrock is suspect;
   a frontier node read 20 times is signaling priority). Reads become
   evidence in promotion decisions — the missing complement to
   `reflect`'s harvest-debt (which measures unwritten knowledge but
   not unread knowledge).
2. **`kos ask <question>`** — scoped, ranked retrieval over
   nodes/findings/ideas. Interim implementation can be lexical +
   graph-proximity (orient's loader already parses everything);
   flyloft is the designed substrate when it exists. This must NOT
   grow into flyloft-inside-kos (composition-analysis item 4 warns
   exactly this) — the verb is the contract; the substrate swaps.
3. **The crosswalk as query pattern.** The re-discovery preventer:
   before designing, query the record — ours and civilization's.
   Making `kos ask` answer "what did we rule out about X and why" is
   the graveyard finally paying rent.
4. **Read-side acceptance test for writes.** A finding isn't done
   until it's *findable*: the harvest checklist gains "would `kos ask`
   surface this for the question it answers?" — cheap while ask is
   lexical, honest forever.

## What this is not

Not a replacement for F19's predictor (push) — pull and push are the
two halves of active knowledge; pull is buildable now with lexical
tools, push waits on cue semantics. Not flyloft — flyloft is the
retrieval substrate; this is the missing kos *verb* and the missing
telemetry that would tell us whether any of it earns its keep.

## Crystallized (2026-08-16)

Directions 1 and 2 have owners now. The idea stays here as the source;
the testable questions live in kos's own graph, since kos is their
subject.

- Frontier questions, `kos/_kos/nodes/frontier/`:
  `question-read-telemetry-decision-value` (does measuring circulation
  change a later decision, or does the instrument go unread like the
  graph it measures), `question-kos-ask-retrieval-value` (does the verb
  beat grep on real session questions, and does it get used), and
  `question-read-surface-sequencing` (does verb-first-then-MCP hold, or
  does the benefit only arrive inside the agent task loop). Each carries
  a success signal and a kill condition.
- bd: `aae-orc-2qlx0` (read-telemetry collection slice, carrying the
  pre-registration amendment) and `aae-orc-jajn3` (`kos ask` Phase 0).
- Proposal: `kos/docs/proposals/read-path-first-steps.md`.
- ADR-009 records the single-query-engine decision behind direction 2.
