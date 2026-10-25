# Finding: the kos status Wizard-of-Oz ran one day, not three cycles

Status: final
Date: 2026-10-24
Brief: `_kos/probes/brief-kos-status-woz.md` (opened 2026-07-04)
Issue: kos#64 (roadmap#29, `kos status`), open
Instrument: `tools/kos-status` in the orchestrator repo, added 2026-07-04 and unchanged since

## Headline

The trial produced one day of evidence, and that day was positive: the status
verb was run voluntarily as a pre-harvest orient and surfaced the harvest
surface that session then worked. The brief's timebox was three working
cycles, and the trial never reached a second one. The registered kill
criterion was therefore never evaluated, in either direction. The R1 inversion
is neither confirmed nor falsified by this probe.

## What was recorded

- **The one day.** The orchestrator's session log entry for 2026-07-04..05 records
  that tools/kos-status shipped with this brief's kill criterion, was used
  voluntarily as a pre-harvest orient, and correctly surfaced the harvest
  surface. The session log describes voluntary use and one surfaced
  obligation acted on; it does not count invocations. The brief's success
  signal reads "at least one session where the surfaced obligation (e.g.
  unharvested finding) was acted on and would plausibly have been missed
  otherwise". The session log shows the first half. It does not say the
  harvest would have been missed without the verb, so the single-session
  signal is met only in part, and the habit signal not at all.
- **The telemetry.** The script appends every invocation to
  `~/.kos-status-woz/<date>.jsonl`. On the host the 2026-07-04 session ran
  on, the directory holds one file, `2026-07-04.jsonl`, with five lines, per
  the review of 3ab80438 on kos#151 (read on that host, not re-read for this
  revision). Four parse as invocations, at 2026-07-04T22:31Z and
  2026-07-05T03:08Z, 03:57Z and 04:42Z; a fifth line at 05:20Z names no
  graph. So the verb ran four or five times over about seven hours in that one
  session. The log does not record which runs were voluntary, so the
  per-session voluntary count the kill criterion uses is not settled by it.
  On the host this finding was written on, the directory does not exist
  (checked 2026-10-24).
- **No later use on record.** The log has nothing after 2026-07-05, no
  session log entry after that date names kos-status or the WoZ, and
  `tools/kos-status` has no commit after the one that added it.

## What it means for the brief

- Hypothesis ("consulted in >=2/3 of sessions once wired into session-start
  habit"): untested. The verb was never wired into a session-start habit, so
  the condition the hypothesis names did not arise.
- Kill criterion (<=1 voluntary invocation per session after three cycles):
  not evaluated. The trial stopped by disuse, not by a recorded decision.
- The one-day result is consistent with the hypothesis and is too small to
  carry it.

## Closing the brief

The brief is closed with this finding as its result: the trial as designed
is not running and has no data past its first day. A new trial would need the
instrument wired into session start first (the brief's own precondition) and
the telemetry read from every host that runs it. Whether to run one is a
question for kos#64, which stays open.
