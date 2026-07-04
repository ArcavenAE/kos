# Exploration Brief — kos status Wizard-of-Oz (roadmap#29 / kos#64)

**Date opened:** 2026-07-04
**Question:** Does a session-entry status verb — "where am I in the
cycle, what's the next verb" — actually get consulted and change
operator behavior, when it costs nearly nothing to run?
**Hypothesis:** The R1 inversion holds: given a free status verb, the
operator (human or agent) consults it at session start and the next
verb it suggests is followed more often than not. If the operator
won't consult a nearly-free status verb, no amount of engineering
makes the tool-drives-process inversion real (party round 2, Maya).

**predicted_confidence:** 0.7 that status is consulted in >=2/3 of
sessions once wired into session-start habit; 0.5 that the suggested
next verb is followed without modification.

**Instrument:** `aae-orc/tools/kos-status` (bash, deliberately not
Rust). Sections: open probe loops, unharvested findings, stale
frontier, validate + uncommitted _kos state, bd ready count, single
next-verb suggestion. Every invocation appends to
`~/.kos-status-woz/<date>.jsonl` — usage telemetry is the experiment's
own measurement (per-session file family; no shared writer).

**Timebox:** three real working cycles (per round-2 consensus:
promote to Rust only after surviving three cycles).

**Success signal:** invocation log shows habitual use; at least one
session where the surfaced obligation (e.g. unharvested finding)
was acted on and would plausibly have been missed otherwise.

**Kill criterion (registered now, per Quinn's round-1 demand):** if
after three cycles the log shows <=1 voluntary invocation per session,
or the next-verb suggestion is systematically ignored, the WoZ dies
and roadmap#29's design is rethought before any Rust ships — the
failure would falsify R1's core mechanism, not just this script.

**Non-goals:** enforcement of any kind; fixture-workspace AC (that's
the Rust implementation's AC in kos#64); hooks (gated on the
environment contract, kos#65).
