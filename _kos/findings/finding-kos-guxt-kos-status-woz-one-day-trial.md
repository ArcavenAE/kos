# Finding: the kos status Wizard-of-Oz ran one day, not three cycles

Status: final
Date: 2026-10-24
Brief: `_kos/probes/brief-kos-status-woz.md` (opened 2026-07-04)
Issue: kos#64 (roadmap#29, `kos status`), open
Instrument: `aae-orc/tools/kos-status`, added in aae-orc 96f09a6 (2026-07-04) and unchanged since

## Headline

The trial produced one day of evidence, and that day was positive: the status
verb was run voluntarily as a pre-harvest orient and surfaced the harvest
surface that session then worked. The brief's timebox was three working
cycles, and the trial never reached a second one. The registered kill
criterion was therefore never evaluated, in either direction. The R1 inversion
is neither confirmed nor falsified by this probe.

## What was recorded

- **The one day.** The aae-orc session log entry for 2026-07-04..05 records
  that tools/kos-status shipped with this brief's kill criterion, was used
  voluntarily as a pre-harvest orient, and correctly surfaced the harvest
  surface. That is one voluntary invocation in
  one session, and one surfaced obligation acted on. It meets the brief's
  success signal for a single session ("at least one session where the
  surfaced obligation was acted on"), not its habit signal.
- **No later use on record.** No session log entry after 2026-07-05 names
  kos-status or the WoZ, and `tools/kos-status` has no commit after the one
  that added it.
- **The telemetry is not on this host.** The script appends every invocation
  to `~/.kos-status-woz/<date>.jsonl` and creates that directory on each run.
  On corporate the directory does not exist (checked 2026-10-24), so the script
  has never run there. The 2026-07-04 run's log, if it survives, is on the host
  that session ran on; it was not read for this finding.

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
