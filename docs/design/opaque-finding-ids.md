# Opaque finding ids: design review and plan

**Date:** 2026-09-25
**Author seat:** arcaven-architect-g5-0, on an operator commission relayed by
director (brief `2026-09-25-kos-opaque-finding-ids`)
**Status:** design and plan only. Nothing here is built. Where this note
departs from the 2026-09-16 ruling, it says **PROPOSAL** and the ruling stands
until the operator rules again.
**Decision of record:** `_kos/nodes/bedrock/elem-kos-artifact-id-scheme.yaml`.
**Study:** `_kos/findings/finding-173-kos-artifact-id-allocation-study.md`.
**Implementation ticket:** `aae-orc-wmna4`.

Finding numbers still collide, and they collide on main. Today orc `origin/main`
carries two files numbered 175 and two numbered 177, and orc and kos each have
a finding-173. The ruling adopts bd-style ids minted at authoring. This note
checks that design against bd's source and kos's code, fixes the parts that do
not carry over from bd's world to kos's, and orders the work into PRs.

## 1. Premises checked

Each line is one command and what it returned.

| Premise | Command | Result |
|---|---|---|
| bd algorithm matches finding-173 | `git show 0c5fa422d:internal/idgen/hash.go`, `:internal/storage/domain/adaptive.go`, `:internal/storage/domain/issue.go` in `forks/beads` | Matches: input `title\|description\|creator\|UnixNano\|nonce`, sha256, 2 to 5 leading bytes by length, base36, first L in 3..8 with `1-exp(-n²/2·36^L) <= 0.25`, ten nonces per length through 8 against `Exists`. |
| No minting code in kos | `grep -n next_finding_number kos/src/process.rs` | `kos finding` still does max+1 over the local `findings/` (`process.rs:141`, `:283-303`). |
| Duplicates exist on orc main now | `git ls-tree --name-only origin/main _kos/findings/` | Two files at 175, two at 177. |
| The orc runs no validate | `ls .github/workflows lefthook.yml` at orc root | Neither exists. kos's own `graph-validate.yml` runs validate on the kos graph only, advisory (kos#106). |
| Most findings are hand-written prose | `ls _kos/findings \| sed 's/.*\.//' \| uniq -c` | orc: 168 `.md`, 19 `.yaml`. kos: 7 `.md`, 44 `.yaml`. `kos finding` writes `.yaml` only. |
| bd ids already appear as finding names | `ls kos/_kos/findings` | `finding-aae-orc-5lbu-serena-live-lsp-baseline.md` and `finding-aae-orc-msqx-scip-indexing-cost.md`, named after bd tickets. |
| `finding_key` reads a leading digit run | `sed -n 124-137p kos/src/findings.rs` | `take_while(is_ascii_digit)`: an opaque suffix such as `4a7k` keys as `finding-4`. |

The last two rows change the design; sections 2.3 and 2.4 say how.

## 2. Review

### 2.1 Hash inputs, and whether determinism is wanted

bd hashes title, description, creator, created_at in nanoseconds and a nonce.
In bd two of those inputs carry the uniqueness: created_at is set once from a
server-side clock, and the store's `Exists` check catches the rest. In kos,
neither holds up:

- **creator separates nothing.** Every assignee in this fleet is the one human
  principal (`task-workflow.md`, "A claim does not name an actor"), so fan-out
  arms share it.
- **title and description are often identical across arms.** A fan-out gives
  N arms one prompt; the 2026-08-09 arms were all told to take "the next free
  number" (ul2h notes), and a shared topic brings a shared title.
- **created_at is the only separator left**, and a timestamp is a weak one:
  arms started by one workflow can mint within the same clock tick, and clock
  resolution differs by platform.

Determinism buys nothing here. bd needs the hash to be stable across its own
retry loop, which is why it fixes `CreatedAt` before minting. kos writes the id
into the file once and never recomputes it, so nobody ever needs to derive the
same id twice.

**Recommendation (PROPOSAL, changes the ruling's reference inputs):** draw 16
bytes from the OS random source for each attempt and encode them with bd's
base36 and byte widths. Keep bd's attempt loop (ten tries per length, then grow
toward 8) so exhaustion behaves the same. Record `created_at` in the file's
frontmatter as data, not as a hash input. The algorithm keeps bd's shape and
drops the inputs that separate nothing in kos. The new direct dependency is
`getrandom`, which is already in the lock file transitively.

### 2.2 The existence check, the threshold, and the minimum length

The two tools check different sets:

- **bd's `Exists` sees every id**, so the 0.25 birthday bound over the
  per-prefix count is a retry-budget rule, not a safety rule. A 25 percent
  chance that some pair would clash only means some mints take a second try.
  No duplicate is ever stored.
- **kos's local check sees only merged findings on the arm's base.** Those can
  never be duplicated. The blind set is the findings minted and not yet merged
  anywhere: sibling fan-out arms, other sessions' open branches, and the
  unpushed worktree on another host that `aae-orc-dt8ur` records.

So the count that governs risk is the in-flight count m, not the graph's
population n. The chance of any clash among m blind mints at length L is
`1 - exp(-m(m-1) / (2·36^L))`:

| L | m = 6 | m = 11 | m = 20 | m = 50 |
|---|---|---|---|---|
| 3 | 0.032% | 0.118% | 0.406% | 2.59% |
| 4 | 0.0009% | 0.0033% | 0.0113% | 0.073% |
| 5 | under 0.0001% | 0.0001% | 0.0003% | 0.002% |

At L=3 the space is also smaller than it looks: bd consumes 2 bytes (65,536
values) mod 46,656, so the effective space is about 41,600.

**Recommendation (PROPOSAL, changes bd's MinLength of 3):**

- Set the floor to 4 in every graph. Keep bd's adaptive growth over n, because
  it still sizes the retry cost against merged findings.
- The orc graph, with 187 numbered findings, already comes out at 4 under
  bd's rule (163 is the L=3 boundary). The floor changes only small graphs such
  as kos (51) and gives one length across the fleet.
- The duplicate check in `kos validate` stays the backstop for the residue,
  which the table puts under 0.1 percent per fan-out even at 50 arms.

### 2.3 Prefix

wmna4 proposes the graph_id from `kos.yaml` as the prefix, giving
`finding-aae-orc-k3m9-slug.md`. That shape is already taken:

- For the orc graph, `aae-orc-k3m9` is exactly the shape of a bd ticket id.
- The two kos findings named after bd tickets (`finding-aae-orc-5lbu-...`,
  `finding-aae-orc-msqx-...`) would read as minted ids of the orc graph while
  sitting in the kos graph.
- `parse_md_edges` already treats any `aae-orc-` token as a citation, so every
  new finding id would also look like a ticket reference to the edge extractor.

The graph already has a namespace mechanism: `graph::slug`, ratified the same
day for exactly this job.

**Recommendation (PROPOSAL, changes "a graph prefix plus a short hash
suffix"):**

- The id is the kind word plus the suffix: `finding-k3m9`. The filename is
  `finding-k3m9-<slug>.md`.
- Inside its own graph a finding is cited as `finding-k3m9`. Across graphs it
  is cited as `aae-orc::finding-k3m9`.
- One namespacing mechanism, not two. In filenames and commit subjects the id
  is ten characters, the same as `finding-173`.
- The ruling's purpose for the prefix, keeping ids apart across graphs, is met
  by `graph::`, which is what bd's prefix does too (finding-173 section 4:
  "the prefix does all cross-project namespacing; the hash never does").

If the operator keeps a graph prefix in the id, it should be a short code that
is not a bd prefix, recorded in `kos.yaml`. `graph_id` itself collides with
bd for the orc.

### 2.4 Coexistence with numbered findings

No migration. bd's live mix of 3, 4 and 5 character suffixes is the precedent.
What must change is classification:

- **Key rule.** The key is the segment between `finding-` and the next `-`:
  - All digits: numbered, compared as an integer (so `019` and `19` are one
    key).
  - Anything else: opaque, compared lowercase.
  - A minter never emits an all-digit suffix. It treats one as a collision and
    tries again, which happens with probability (10/36)^4, about 0.6 percent at
    L=4. Without that rule `finding-0173` could mint and mean 173.
- **The bug this fixes first.** `finding_key` today takes the leading digit
  run, so `finding-4a7k-x` and `finding-4zzz-y` both key as `finding-4` and
  fail validate as duplicates of each other and of a numbered finding-4. About
  28 percent of suffixes start with a digit. The validate change has to land
  before the first minted finding, or the first fan-out after adoption produces
  false duplicate failures.
- **Legacy odd names.** `finding-aae-orc-5lbu-...` keys as the opaque `aae`,
  and so does its sibling `...msqx...`: a false duplicate under the new rule.
  Either rename those two files to their minted shape with a forwarding note,
  or teach the key rule that a segment equal to a bd prefix takes the next
  segment too. I recommend the rename: two files, one PR, and the special case
  never enters the code.
- **Resolution.** Every reader resolves the same four forms through one
  function in `findings.rs`:
  - the key (`finding-173`, `finding-k3m9`);
  - the full stem;
  - the bare slug, when unique in the graph;
  - any of those after `graph::`.

  Callers: `kos ask` edge walking, validate's edge-target check (which today
  warns "may be a finding or probe" for every short citation, numbered ones
  included), charter render, and `aq finding`/`aq harvest` in the orc.
  - `aq`'s slug fallback globs `finding-*-*${key}*`, which never matches a key
    in the first segment, so `aq finding k3m9` would miss.
  - A bare slug that matches two files is an error that lists both, git's
    ambiguous-abbreviation model.
  - A bare citation resolves in the citing file's own graph first, which
    keeps orc `finding-173` and kos `finding-173` apart without edits.

**Slug as citation key (PROPOSAL, changes "slug kept ... as the citation
key"):** fan-out arms handed one topic are the case most likely to pick the
same slug. Two graphs picking the same slug is harmless, because `graph::`
separates them. Two findings in one graph picking the same slug is the real
case, and there the slug cannot be the key. I recommend:

- The canonical citation is the id (`finding-k3m9`).
- The slug stays in the filename for readers and resolves as an alias while it
  is unique.
- validate WARNs on a duplicate slug within a graph. It does not fail, because
  the ids are still distinct.

**Closing the numbered sequence.** A hand-written `finding-190-x.md` after
adoption repeats the old failure. `kos.yaml` gains
`findings: { numbered_through: <N> }`, set once per graph at adoption, and
validate WARNs on a numbered finding above it. Ratchets that later need
changing are not the point; the marker records where the old sequence ended.

### 2.5 Authoring path

168 of 187 orc findings are hand-written markdown; `kos finding` writes YAML.
If minting lives only in a verb that writes a file shape agents do not use,
agents keep allocating by hand. The mint has to be reachable on its own:

- `kos finding <slug> --title ...` keeps writing a file, gains `--md`, and
  defaults to `.md` with frontmatter (the shape `findings.rs` calls
  first-class).
- `kos id finding <slug>` prints the id and filename without writing, for an
  agent that authors the file itself.

The authoring guidance moves from "take the next free number" to "run `kos
finding` or `kos id finding`; never hand-allocate" in orc `CLAUDE.md` and
`kos/CLAUDE.md`. That wording is the guidance item on `aae-orc-ul2h`.

### 2.6 Scope

Findings only, now. The other artifact kinds do not have the counter:

- **probes** are `brief-<slug>`, **ideas** are `<slug>.md`, and **nodes** are
  slugs. None allocates a number, so none has the fan-out failure.
- Their failure is same-slug collision, which is detection work
  (`aae-orc-73t0d`) plus `graph::slug` for cross-graph references
  (`aae-orc-qvpr8`).

Order: findings (this plan), then qvpr8, then 73t0d. Opaque ids for the other
kinds only if a same-slug collision is ever measured under fan-out, per
finding-173's (c3) fallback.

### 2.7 The gate

A duplicate finding id is structural, and ADR-007 lets structural validity
gate. kos#106 kept it advisory, following the 2026-08-09 bs28 scoping. Two
facts argue for keeping it advisory until the migration lands:

- orc main carries two standing duplicates (175, 177), and a required check
  would fail every orc PR until someone renumbers them.
- The orc has no CI at all, so for the graph that collides most there is
  nothing to make required yet. Wiring comes first (`aae-orc-bs28`).

**Recommendation, a ruling request, not a change:**

- Once the validate key change ships and 175/177 are renumbered, make only the
  duplicate-id section of `kos validate` a required check, in kos CI now and in
  orc CI when bs28 wires it.
- Everything else in validate stays advisory: warnings, drift, and the
  numbered-above-marker warning.
- Default if the operator does not rule: stays advisory. Nothing ships without
  a ruling. Expiry: revisit at the first fan-out after adoption.

## 3. Plan

Flat tickets with `blocks` edges. wmna4 keeps the minting work. The new tickets
are pieces that land in their own PR.

| # | Work item | Repo, files | Proving test | Blocked by | Ticket |
|---|---|---|---|---|---|
| 1 | Finding key classification: numbered vs opaque, integer compare, duplicate-slug WARN, `numbered_through` WARN, rename the two bd-named kos findings | kos: `src/findings.rs` (`finding_key`), `src/validate.rs`, `src/model.rs` (manifest field), `_kos/findings/` (two renames with forwarding notes) | table test: `finding-4a7k-x` and `finding-4zzz-y` are distinct keys; `finding-019-a` and `finding-19-b` collide; `finding-k3m9-x` and `finding-k3m9-y` collide; same slug under two ids WARNs, does not fail | none | `aae-orc-ottn4` |
| 2 | Mint: `src/id.rs` (random 16 bytes per attempt, bd base36 widths, floor 4, adaptive over n, ten tries per length, all-digit rejected); `kos finding` mints and writes `.md` by default; `kos id finding` prints id and filename | kos: `src/id.rs`, `src/process.rs`, `src/main.rs`, `Cargo.toml` (`getrandom`) | unit: adaptive length at the kos count (51) and orc count (187) gives 4; injected RNG covers the all-digit and existence retries. Integration (wmna4 acceptance): clone one base commit twice, run `kos finding same-slug --title same` in each, assert distinct ids and that `kos validate` passes on the union | 1 | `aae-orc-wmna4` (REMAINING updated) |
| 3 | One resolver for citations (key, stem, unique slug, `graph::` form, own graph first, ambiguity lists candidates) used by ask, validate's edge check, charter render | kos: `src/findings.rs`, `src/ask.rs`, `src/validate.rs`, `src/charter.rs` | resolver table test over a fixture graph with numbered, opaque, duplicate-slug and cross-graph entries; validate no longer warns on a short citation that resolves | 1 | `aae-orc-cnnim` |
| 4 | `aq finding` and `aq harvest` accept opaque keys (match the first segment) | orc: `tools/aq` | fixture dir with `finding-k3m9-x.md`: `aq finding k3m9`, `aq finding finding-k3m9`, `aq finding x` all print it; a numbered key still pads | 1 | `aae-orc-6di8d` |
| 5 | Authoring guidance and adoption: set `numbered_through` per graph, change "next free number" text to the mint verbs | orc: `CLAUDE.md`, `.claude/rules/` where findings are authored; kos: `CLAUDE.md`; each graph's `kos.yaml` | `grep -rn 'next free number\|max.plus.one'` in authoring docs returns only historical findings | 2 | `aae-orc-ul2h` (notes; its guidance item) |
| 6 | Renumber orc 175 and 177 duplicates (the later-merged of each pair), forwarding notes | orc: `_kos/findings/` | `kos validate` in the orc reports 0 duplicate-id failures | none | `aae-orc-rr5h2` |
| 7 | Gate: duplicate-id section required | kos CI now; orc CI via `aae-orc-bs28` | a PR adding a duplicate id fails that one check; a warning-only PR passes | 1, 6, operator ruling | ruling request, no ticket until ruled |

Items 1 and 6 can start now and run in parallel. Item 2 cannot merge before 1
(section 2.4). Items 3 and 4 follow 1 and do not depend on 2, because a fixture
can hold an opaque file.

## 4. Ruling requests for the operator

Each has a default that applies if no ruling comes.

1. **Hash inputs** (2.1): random bytes per attempt in place of
   title|description|creator|created_at. Default: adopt, since the ruling's
   inputs separate nothing in kos beyond the timestamp.
2. **Floor of 4** (2.2). Default: adopt.
3. **No graph prefix in the id; `graph::` namespaces it** (2.3). Default:
   none. This one changes the ruling's wording, so it waits for a ruling.
   Item 2 is written so that the prefix is one constant either way.
4. **Id, not slug, as the canonical citation** (2.4). Default: adopt, slug
   kept as an alias.
5. **Duplicate-id section required** (2.7). Default: stays advisory.

## 5. Not decided here

- Relocation between graphs (`aae-orc-qso2f`): an opaque finding keeps its id
  and changes its namespace, the same rule finding-173 gives slugs.
- Whether charter render displays the slug beside an opaque id. This is a
  rendering choice for item 3's author.
