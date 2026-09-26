# CLI Maturity for External Use

*Idea — pre-hypothesis brainstorming. Generative, uncommitted.*

## The observation

kos works from its own directory. To be useful on other repos, an
entire class of problems needs solving — not just orient, but the
full surface area of "what does a user need when they run kos
somewhere that isn't aae-orc/kos?"

The orient standalone fix is one instance. But the real work is
getting the initial binary into a state where it can be used beyond
this directory. That means: install, init, orient, day-to-day
workflow, docs, and process scaffolding all need to work for someone
who has never seen the kos source.

## What's missing

### 1. Process subcommands

The process cycle is Orient → Ideate → Question → Probe → Harvest →
Promote. Only `orient` is a subcommand. The rest are manual file
operations. Scaffolding commands would reduce ceremony and errors:

- `kos idea "title"` — scaffold idea in `_kos/ideas/`
- `kos question "title"` — scaffold frontier question node with
  edges and provenance
- `kos probe "question-id"` — scaffold exploration brief linked to
  a question, with hypothesis/timebox/success-signal template
- `kos harvest "probe-id"` — scaffold finding linked to probe,
  prompt for result (confirms/partial/redirects)
- `kos promote "node-id" bedrock` — move node between confidence
  tiers, validate edges, prompt for evidence

These enforce naming conventions, edge wiring, and provenance.
They're the difference between "read CLAUDE.md and manually create
YAML" and "run a command."

### 2. User documentation

Two audiences, two formats:

**Humans** want:
- Getting started (install, init, first orient, first idea)
- Day-to-day workflow (the process cycle in practice)
- Different circumstances (standalone repo, subrepo, orchestrator,
  brownfield onboarding, code-only project)
- Reference (all subcommands, all flags, examples)

**LLMs** want:
- Everything at once — every subcommand, flag, workflow pattern,
  and process cycle doc in a single structured dump
- Optimized for context injection, not human scanning
- Always current (generated from binary, not maintained separately)

### 3. Built-in docs vs. external docs

Proposal: docs built into the binary, extractable.

- `kos help --full` — dumps complete reference as markdown.
  All subcommands, flags, examples, process cycle, getting-started.
  Generated from clap metadata + embedded workflow docs.
- `kos help getting-started` — targeted topic help
- `kos help process` — the cycle explained with examples
- `kos help --full > docs/reference.md` — generate repo docs from
  binary, always current, can't drift

The README stays conceptual (what is kos, why). Reference docs are
always `kos help --full` away. Staleness problem solved.

### 4. Standards for LLM-readable CLI help

No established standard for "dump all your usage for an LLM." The
closest are man pages and --help, designed for humans scanning.
An LLM wants structured, complete, single-document output with
semantic context and examples.

Could this be a convention? `--help-llm` or `--help-full` that
outputs markdown with:
- Every subcommand with all flags and defaults
- Usage examples per subcommand
- Workflow examples (multi-command sequences)
- Concept explanations where needed
- Machine-parseable sections (## Subcommands, ## Workflows, etc.)

This would work as a built-in skill equivalent — instead of a
`.claude/commands/kos.md` that gets stale, the binary IS the skill.

### 5. Orient for standalone repos

Orient currently searches the aae-orc charter for target mentions.
Standalone repos have no aae-orc charter. Orient needs to detect
standalone context and read the local charter, display local graph
state (nodes by confidence, findings, probes, ideas), and surface
useful orientation without orchestrator context.

## The framing

This isn't "fix orient" plus "add docs" plus "add subcommands."
It's one problem: **make kos usable outside its own directory.**
The orient fix, the process subcommands, and the docs are all
instances of the same maturity gap.

The work order follows the process cycle itself:
1. Install (brew/update — done)
2. Orient (standalone fix — next)
3. Init (works, tested on switchboard/ftc)
4. Ideate/Question/Probe/Harvest (process subcommands)
5. Help/Docs (built-in reference)

Each step makes the next more useful. You can't orient without
install. You can't ideate without orient. You can't follow the
process without scaffolding. You can't onboard new users without
docs.

## Tensions

- How much CLI scaffolding before the process subcommands exist?
  The manual YAML approach works. Is it a barrier or just ceremony?
- Built-in docs add binary size and maintenance surface. Is the
  staleness problem real enough to justify embedding?
- The LLM help format — is this a kos concern or a broader tool
  convention that should live elsewhere (spectacle? a standard)?
- Process subcommands assume the process is stable. Is it? Or will
  the first few external uses change the cycle?
