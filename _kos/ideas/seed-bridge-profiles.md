# Extensible seed via pack-shipped bridge profiles

*Idea — pre-hypothesis brainstorming. Generative, uncommitted.*

## The observation

kos seed needs to understand project structure semantically — not just
what files exist, but what they mean. Different SDD systems (bmad,
spectacle, gitspec, pennyfarthing, multiclaude, gastown, chainlink,
openclaw, ThreeDoors BOARD) organize knowledge differently. kos can't
have built-in knowledge of every system. But each system knows itself.

## The design

Each content pack ships a **bridge profile** — a manifest that describes
the system's artifacts to kos:

- **Fingerprints**: file/directory patterns that identify the system
  (e.g., `bmad-agent/**/*.md`, `master-prompt.md`, `BOARD/`)
- **Artifact map**: what each pattern means in kos terms (element,
  question, value, graveyard) and what edges to expect between them
- **Version ranges**: different layouts for different versions of the
  system. v5 bmad has different artifact paths than v6.
- **References**: where to learn more — repo URL, docs, changelog
- **Migration hints**: what changed between versions, deprecated
  patterns, renamed artifacts

sideshow distributes packs. Bridge profiles ship with packs. When you
install bmad via sideshow, kos gains the ability to recognize bmad
artifacts. When you update bmad, the recognition gets richer.

## The three-layer architecture

1. **`kos seed scan`** (mechanical, Rust) — loads installed bridge
   profiles from sideshow's install path, matches patterns against
   the repo's filesystem, outputs annotated JSONL manifest. Fast,
   deterministic, extensible via profiles.

2. **Agent workflow** (semantic, LLM) — consumes the annotated scan,
   reads the actual sources, understands the project's conventions,
   creates nodes via process subcommands. The bridge profile gives
   the agent a head start — it knows what it's looking at.

3. **Process subcommands** (mechanical, Rust) — `kos idea`, `kos
   question`, `kos finding`, `kos probe`. The output mechanism.
   Already built (session-014).

## The versioning insight

Bridge profiles evolve with their packs:
- bmad v5 → v6: profile updates with new fingerprints, old ones
  deprecated. `kos seed scan` sees the repo's actual layout, matches
  against the installed profile's version range, reports which version
  it's looking at.
- spectacle v1 has no profile. spectacle v2 ships one. Installing v2
  makes kos recognize spectacle artifacts it was blind to. The repo
  didn't change — your understanding of it got richer.
- Profile improvements are decoupled from kos releases. sideshow
  syncs the profile; kos reads it.

## The prediction connection

The delta between "what kos knows about this repo" and "what kos
could know given installed profiles" is measurable. A newly installed
bridge profile is a prediction: "these files mean something you didn't
know about." The scan output before and after installing a profile is
the surprise signal — connecting this to the predictor layer (F6).

## The meta-knowledge angle

Bridge profiles are themselves knowledge about knowledge systems.
Each profile is a small graph: artifacts have types, artifacts have
relationships to each other, artifacts map to kos node types. This
is kos's cognitive architecture applied one level up — the graph
holds knowledge about the systems that produce knowledge.

## Open questions

- What's the bridge profile format? YAML to match kos convention, or
  TOML to match marvel convention?
- Where does sideshow put profiles so kos can find them? Same path
  as pack artifacts, or a separate registry?
- How does version detection work? Parse a version file, infer from
  artifact layout, or both?
- Should profiles describe just fingerprints (what to recognize) or
  also extraction templates (how to create nodes from what's found)?
- Can the profile itself be a kos graph fragment? Nodes about the
  system's artifacts, edges about their relationships?
- How does this interact with the universal-bridge-intermediary idea?
  The bridge profile IS the per-format bridge adapter that idea
  describes.

## Relationship to existing ideas

- **universal-bridge-intermediary** — the bridge profile is the
  per-format adapter that idea calls for
- **composable-process-variants** — seed is a process variant
  (bootstrap chain: orient → bulk-read → extract → validate → harvest)
- **kos-as-lifecycle-knowledge-layer** — seed is how kos becomes
  the knowledge layer for projects that already exist
- **finding-037-bootstrap-seeding-gap** — this directly addresses
  the "no seed step" gap

## Tensions

- Is the bridge profile too much abstraction before we've seeded more
  than 2 repos manually? The ThreeDoors bootstrap was one session of
  human+agent work without any profile system. Would profiles have
  helped, or would they have been overhead?
- sideshow is MVP. Does kos depend on sideshow, or does kos have its
  own profile discovery that sideshow happens to populate?
- The LLM is already a universal parser. Does a structured profile
  add value over "here are the files, figure it out"? The hypothesis:
  yes, because the profile gives the LLM domain knowledge it wouldn't
  otherwise have (version history, migration paths, artifact semantics
  specific to that system).
