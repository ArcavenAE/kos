# finding-136: operator chat is a probe surface with no harvest step, and repo sessions have started citing its ghosts

**Date:** 2026-08-16
**Status:** OBSERVED. Small finding; no new rule this wave.
**Scope:** orc-level, the collaboration process itself (not any one component).
**Trigger:** reconciling the read-path record surfaced two references to a
"substrate report" that no repo session could open.

## The finding

Planning conversations in the operator chat are a probe surface. They produce
the same class of output the kos cycle expects from a probe (a report, a
verdict, a design with pre-registered confidences), but they carry no harvest
obligation, so the output exists in the operator's context and in whatever
ticket text quotes it, and nowhere on disk. For a repo session, an
uncommitted chat artifact does not exist. The gap is structural, not
incidental: nothing in the cycle notices that a probe happened somewhere a
`_kos/` directory could not see.

## Evidence

Two references in the current record pointed at the same missing document.

1. **bd `aae-orc-1od3`** ("substrate report not in graph") records design-field
   claims inherited from ticket `isie`'s lineage. The claims read as settled
   findings; their source was a chat-produced report with no committed prop
   behind it.
2. **The initial-analysis section 2** cited the same substrate report as if it
   were a graph artifact.

The report was real. It was produced in conversation, and it was committed on
2026-08-16 as `docs/kos-substrate-alternatives.md`. Until that commit,
both citations were pointing at a ghost, and neither citation looked like a
gap from inside a repo session; each looked like a normal reference to prior
work.

## Why it recurs

Every session where substantive work happens in chat reproduces the
conditions. The cost is asymmetric in the way that makes a failure durable:
the chat participant pays nothing to leave the artifact uncommitted, and the
next repo session pays the whole cost of discovering that a cited document is
not there. Downstream, an uncommitted artifact can be cited confidently
enough that its absence is read as a filing error rather than a missing
source.

## Disposition

Recorded, no rule this wave. Two things would raise it:

- A third instance of a repo session citing an uncommitted chat artifact.
- Any case where the uncommitted artifact's absence changed a decision rather
  than merely delaying one.

The adjacent rule family is `.claude/rules/tooling-friction.md` (capture
before workaround); this failure has the same shape, with the capture point
moved from "before applying a workaround" to "before the chat that produced
the artifact ends." If it recurs, it graduates to a frontier question node in
the orc graph and, if the shape holds, a behavior-trigger rule matching the
existing family.

## Cross-references

- `docs/kos-substrate-alternatives.md` (the artifact, committed 2026-08-16)
- bd `aae-orc-1od3` (report located; remaining work is naming the two
  "five properties" lists distinctly and tracing `isie`'s design-field claims
  to props or tagging them asserted-unverified)
- `.claude/rules/tooling-friction.md` (adjacent rule family)
- `docs/proposals/read-path-first-steps.md` (the wave that surfaced this)
