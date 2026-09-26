# Opaque finding ids: design review and plan

**Date:** 2026-09-25
**Author seat:** arcaven-architect-g5-0, on an operator commission relayed by
director (brief `2026-09-25-kos-opaque-finding-ids`)
**Status:** design and plan only. Nothing here is built. Where this note
departed from the 2026-09-16 ruling, it made a proposal; every one has since
been **RULED 2026-09-25** by the operator (section 4) and is marked so where
it appears.
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
| `finding_key` reads a leading digit run | `sed -n 124-137p kos/src/findings.rs` | `take_while(is_ascii_digit)`, else the full id: a prefixed id such as `finding-aae-orc-k3m9-x` keys on its whole stem, slug included, so two colliding ids with different slugs are never compared. |

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

**RULED 2026-09-25 (amends the 2026-09-16 reference inputs):** draw 16
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

**RULED 2026-09-25 (departs from bd's MinLength of 3):**

- Set the floor to 4 in every graph. Keep bd's adaptive growth over n, because
  it still sizes the retry cost against merged findings.
- The orc graph, with 187 numbered findings, already comes out at 4 under
  bd's rule (163 is the L=3 boundary). The floor changes only small graphs such
  as kos (51) and gives one length across the fleet.
- The duplicate check in `kos validate` stays the backstop for the residue,
  which the table puts under 0.1 percent per fan-out even at 50 arms.

### 2.3 Prefix (RULED 2026-09-25)

**Ruling (operator, 2026-09-25, on kos#110):** keep a prefix. The prefix is
the project that owns the finding, which names the scope of the id namespace:
`marvel`, `kos`, `aae-orc`. This replaces the earlier proposal to drop the
prefix and namespace with `graph::` alone.

**Shape.**

- The prefix is the owning graph's `graph_id` from `kos.yaml`, lowercased.
  Every graph_id in the fleet today is unique, starts with a letter, and uses
  only `[a-z0-9-]` once lowercased (`BetterDials` becomes `betterdials`).
  `kos validate` checks those three properties when it reads a manifest.
- The id is the kind word, the prefix and the suffix:
  `finding-aae-orc-k3m9`, `finding-kos-p2ab`, `finding-marvel-7xqh`.
- The filename appends the slug: `finding-aae-orc-k3m9-bus-review.md`.
- The adaptive length counts the findings in the prefix's namespace, which is
  the owning graph. bd's rule counts per prefix the same way.

**Beside `graph::` references.** The two forms do different jobs and read
differently:

- `graph::` says where to look: `marvel::question-permission-model` is a
  node in the marvel graph. It stays the form for node ids and for legacy
  numbered findings, which are unique only within their graph
  (`kos::finding-173`, `aae-orc::finding-173`).
- The prefix says which namespace minted the id. Owner plus a suffix unique
  within the owner makes `finding-aae-orc-k3m9` unique across the fleet, so it
  needs no `graph::` in any graph.
- `aae-orc::finding-aae-orc-k3m9` is legal and redundant. The resolver accepts
  it. validate WARNs when the `graph::` part and the prefix disagree and no
  relocation record explains it.
- **Relocation** (`aae-orc-qso2f`): an id never changes. A finding moved to
  another graph keeps its prefix, which then records where it was minted, and
  the relocation index gives the forwarding. validate WARNs, not FAILs, on a
  prefix that differs from the owning graph.

**Why it will not be confused with bd ids.** For the orc, the prefix is the
same string as bd's: the kinu db holds 1,638 ids, every one prefixed
`aae-orc-` (`bd sql`, 2026-09-25). The inner part, `aae-orc-k3m9`, is exactly
bd's shape. Three rules keep them apart:

1. **The kind word is part of the id and is never dropped.** Prose, commit
   subjects, citations and filenames all write `finding-aae-orc-k3m9`.
   bd never emits an id starting with `finding-`, so the lead word alone
   decides it: `aae-orc-k3m9` on its own always means a bd ticket, and
   `finding-aae-orc-k3m9` always means a finding. The authoring guidance (plan
   item 5) states this in one line.
2. **Tools classify on the lead word first.** `parse_md_edges` splits tokens on
   characters outside `[A-Za-z0-9-]`, so `finding-aae-orc-k3m9` stays one token
   and is taken as a finding citation by its `finding-` test before the
   `aae-orc-` test applies. The shared resolver (item 3) tries the `finding-`
   form first, and its table test includes a finding id whose suffix equals a
   live bd suffix.
3. **The two bd-named kos findings are renamed.** `finding-aae-orc-5lbu-...`
   and `finding-aae-orc-msqx-...` sit in the kos graph and are named after bd
   tickets. Under the ruled shape they read as findings minted by the orc
   graph, so they take minted `finding-kos-...` names with forwarding notes
   that keep the bd reference (item 1).

What remains is a suffix that happens to equal a bd suffix, which is under 0.1
percent per mint at length 4. A machine never confuses them because the lead
word differs. A human skimming might, which is the reason for rule 1. kos does
not check bd, because it must stay usable without bd (SOUL section 2).

### 2.4 Coexistence with numbered findings

No migration. bd's live mix of 3, 4 and 5 character suffixes is the precedent.
What must change is classification:

- **Key rule.** After `finding-`:
  - All digits up to the next `-`: numbered, compared as an integer (so `019`
    and `19` are one key), unique only within its graph.
  - A known prefix followed by `-`: opaque, and the key is
    `finding-<prefix>-<suffix>`, where the suffix is the next segment. The
    known prefixes are the owning graph's plus, at an orchestrator root, those
    of its included graphs. The longest match wins, so `aae-orc` is tried
    before any shorter prefix.
  - Anything else: WARN "unrecognised finding id". The file keys on its full
    stem and never counts as a duplicate of another file.
  - Because a prefix always precedes the suffix, an all-digit suffix
    (`finding-kos-0173`) cannot be read as numbered, so the minter needs no
    all-digit rejection.
- **Why this lands first.** `finding_key` today takes the leading digit run
  and falls back to the full id. For `finding-aae-orc-k3m9-x` the digit run is
  empty, so the key is the whole stem, slug included. Two minted findings
  that collide on id with different slugs get different keys, and the
  duplicate check, the backstop for the blind set in 2.2, never sees the
  collision. **RULED 2026-09-25:** the key change (`aae-orc-ottn4`) lands
  before any id is minted.
- **Resolution.** Every reader resolves the same forms through one function in
  `findings.rs`:
  - the key (`finding-173`, `finding-aae-orc-k3m9`);
  - the full stem;
  - the bare slug, when unique in the graph;
  - any of those after `graph::`.

  Callers: `kos ask` edge walking, validate's edge-target check (which today
  warns "may be a finding or probe" for every short citation, numbered ones
  included), charter render, and `aq finding`/`aq harvest` in the orc.
  - `aq`'s slug fallback globs `finding-*-*${key}*`. It happens to catch a
    bare suffix (`aq finding k3m9`) but also matches any slug containing
    those four characters, so it needs a prefix-aware match, not a glob.
  - A bare slug that matches two files is an error that lists both, git's
    ambiguous-abbreviation model.
  - A bare numbered citation resolves in the citing file's own graph first,
    which keeps orc `finding-173` and kos `finding-173` apart without edits.
    Opaque ids need no such rule.

**Citation key (RULED 2026-09-25, amends "slug kept ... as the citation
key"):** fan-out arms handed one topic are the case most likely to pick the
same slug. Two graphs picking the same slug is harmless, because the prefix
separates them. Two findings in one graph picking the same slug is the real
case, and there the slug cannot be the key. So:

- The canonical citation is the id (`finding-aae-orc-k3m9`).
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

**RULED 2026-09-25:**

- Once the validate key change (`aae-orc-ottn4`) ships and 175/177 are
  renumbered (`aae-orc-rr5h2`), only the duplicate-id section of
  `kos validate` becomes a required check: in kos CI then, and in orc CI when
  `aae-orc-bs28` wires it. That needs the section runnable on its own with its
  own exit status, so a required job can run it beside the advisory one.
- Everything else in validate stays advisory: warnings, drift, and the
  numbered-above-marker warning.
- Revisit at the first fan-out after adoption.

## 3. Plan

Flat tickets with `blocks` edges. wmna4 keeps the minting work. The new tickets
are pieces that land in their own PR.

| # | Work item | Repo, files | Proving test | Blocked by | Ticket |
|---|---|---|---|---|---|
| 1 | Finding key classification: numbered vs prefixed opaque (longest known prefix), integer compare, unrecognised-id WARN, prefix-differs-from-owner WARN, duplicate-slug WARN, `numbered_through` WARN, graph_id shape check; rename the two bd-named kos findings | kos: `src/findings.rs` (`finding_key`), `src/validate.rs`, `src/model.rs` (manifest field), `_kos/findings/` (two renames with forwarding notes) | table test: `finding-aae-orc-k3m9-x` and `finding-aae-orc-k3m9-y` collide; `finding-aae-orc-k3m9-x` and `finding-kos-k3m9-x` do not; `finding-019-a` and `finding-19-b` collide; `finding-kos-0173-x` is opaque, not 173; same slug under two ids WARNs, does not fail | none | `aae-orc-ottn4` |
| 2 | Mint: `src/id.rs` (prefix = owning graph_id lowercased; random 16 bytes per attempt, bd base36 widths, floor 4, adaptive over the graph's count, ten tries per length); `kos finding` mints and writes `.md` by default; `kos id finding` prints id and filename | kos: `src/id.rs`, `src/process.rs`, `src/main.rs`, `Cargo.toml` (`getrandom`) | unit: adaptive length at the kos count (51) and orc count (187) gives 4; injected RNG covers the existence retry and growth to the next length; minted ids carry the owning prefix. Integration (wmna4 acceptance): clone one base commit twice, run `kos finding same-slug --title same` in each, assert distinct ids and that `kos validate` passes on the union | 1 | `aae-orc-wmna4` (REMAINING updated) |
| 3 | One resolver for citations (key, stem, unique slug, `graph::` form, own graph first, ambiguity lists candidates) used by ask, validate's edge check, charter render | kos: `src/findings.rs`, `src/ask.rs`, `src/validate.rs`, `src/charter.rs` | resolver table test over a fixture graph with numbered, opaque, duplicate-slug and cross-graph entries; validate no longer warns on a short citation that resolves | 1 | `aae-orc-cnnim` |
| 4 | `aq finding` and `aq harvest` accept prefixed keys with a prefix-aware match instead of the slug glob | orc: `tools/aq` | fixture dir with `finding-aae-orc-k3m9-x.md` and a slug containing `k3m9`: `aq finding finding-aae-orc-k3m9`, `aq finding k3m9` and `aq finding x` each print only the finding; a numbered key still pads | 1 | `aae-orc-6di8d` |
| 5 | Authoring guidance and adoption: set `numbered_through` per graph, change "next free number" text to the mint verbs | orc: `CLAUDE.md`, `.claude/rules/` where findings are authored; kos: `CLAUDE.md`; each graph's `kos.yaml` | `grep -rn 'next free number\|max.plus.one'` in authoring docs returns only historical findings | 2 | `aae-orc-ul2h` (notes; its guidance item) |
| 6 | Renumber orc 175 and 177 duplicates (the later-merged of each pair), forwarding notes | orc: `_kos/findings/` | `kos validate` in the orc reports 0 duplicate-id failures | none | `aae-orc-rr5h2` |
| 7 | Gate: duplicate-id section required, run on its own with its own exit status | kos: `src/validate.rs`, `src/main.rs`, `.github/workflows/`; orc CI via `aae-orc-bs28` | a PR adding a duplicate id fails that one check; a warnings-only PR passes | 1, 6 | `aae-orc-dq328` |

Items 1 and 6 can start now and run in parallel. Item 2 cannot merge before 1
(ruled, section 2.4). Item 7 follows 1 and 6. Items 3 and 4 follow 1 and do not depend on 2, because a fixture
can hold an opaque file.

## 4. Rulings (operator, 2026-09-25, relayed by director)

All five items this note raised are ruled. They amend the 2026-09-16 ruling
and are to be harvested into `elem-kos-artifact-id-scheme` as its record.

1. **Prefix** (2.3): keep a project prefix naming the owning graph.
2. **Hash input and floor** (2.1, 2.2): 16 OS random bytes per attempt, length
   floor 4, bd's widths and attempt loop otherwise.
3. **Order** (2.4): the `finding_key` fix (`aae-orc-ottn4`) lands before any
   id is minted.
4. **Citation key** (2.4): the id is canonical; the slug is an alias.
5. **Gate** (2.7): only the duplicate-id check becomes required, after
   `aae-orc-ottn4` and `aae-orc-rr5h2` land.

## 5. Not decided here

- The relocation primitive itself (`aae-orc-qso2f`). Section 2.3 gives only
  the id rule: a moved finding keeps its id, prefix included.
- Whether charter render displays the slug beside an opaque id. This is a
  rendering choice for item 3's author.
