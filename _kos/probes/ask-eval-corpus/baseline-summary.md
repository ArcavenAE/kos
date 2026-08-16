# What kos ask has to beat: an honest read of grep and orient

Written 2026-08-16, before `kos ask` was scored, so it cannot be tuned later to
flatter the verb. This is the "what does the verb have to beat" statement the
pre-registration node (`question-kos-ask-retrieval-value`) asks for. It is paired
with `eval-corpus.md`, which holds the 15 questions and per-question baseline
runs. Measurements are from ripgrep over the two graph roots and from Homebrew
`kos alpha-20260809-030901-3495ed1 orient`.

## The two baselines, stated fairly

**grep reaches everything and ranks nothing.** `rg` searches every file in both
graphs, including the 124 `.md` findings orient cannot see. When the query term
is distinctive, grep is genuinely good: Q5 ("who pays / survival predictor")
returns 3 files with the bedrock node first, and I would not expect the verb to
do meaningfully better than that. grep's failures are all the same failure:
no ranking, no tier awareness, no polarity. It cannot tell a graveyard node from
the bedrock node that overturned it, and it cannot tell a common word from a rare
one.

**orient ranks by tier and is blind to most findings.** orient groups the graph
into bedrock / frontier / graveyard / findings sections, which is a real
advantage grep lacks: on ruled-out questions the reader jumps to a 7-to-10-item
graveyard section instead of scanning a flat file list. But orient has two hard
limits. It takes no query, so it dumps 222 to 269 lines and makes the reader
triage all of it. And it reads only the 19 `.yaml` findings in the orc graph; it
never surfaces any of the 124 `.md` findings. I confirmed finding-044 and
finding-136 are absent from orient output. For a graph whose recent findings are
almost all `.md`, that is most of the record.

## Where the baselines already suffice

The verb does not need to win everywhere, and I want the cases it does not need
to win recorded up front.

- **Distinctive-phrase point lookups.** Q5 (survival predictor) is solved by
  grep in 3 files. The verb should match this and add the tier label; it will not
  win by margin.
- **Small-graveyard ruled-out questions.** Q8 (speckit / bmad format) lands in a
  5-file grep and a 7-item orient graveyard section. Both baselines are already
  fine. The bar here is "do not regress."
- **Find-the-node questions where the node is `.yaml` and the wording matches.**
  When the ground truth is a frontier `.yaml` node and the query uses its
  vocabulary, orient lists it by name in the right section. The verb's job is to
  save the reader the whole-dump scan, not to reach something orient cannot.

## Where the baselines bury the answer

This is the middle of the distribution, and it is where the verb earns or fails
its keep.

- **Common-word frontier questions.** Q13 ("director") is 124 grep files. The one
  question node is one hit in 124. orient beats grep here by naming the node in
  its open-questions section, but even orient cannot rank it or attach its `.md`
  finding (finding-064). A verb that returns the director question and its
  finding near the top, in one shot, is a large improvement over both.
- **Popular-substring point lookups.** Q2 ("dolt server") is 27 grep files with
  the two right answers scattered among two dozen bd-friction findings, and one
  of those right answers (finding-039) is `.md` and invisible to orient. Neither
  baseline serves this well.
- **Idea-file-heavy frontier questions.** Q11 (charter management, 28 files) and
  Q12 (active knowledge surfacing, 23 files) drown the question node under idea
  files and cross-references. orient suppresses the idea noise but splits the
  answer across two graphs and two sections.

## Where neither baseline can answer at all

- **`.md` finding lookups.** Q3 (finding-044) is unanswerable from orient by
  construction, and grep only reaches it by brute force among decoys. Any
  question whose answer is a `.md` finding is a place the verb can beat both
  baselines at once, simply by reaching the file and ranking it. Given that 124
  of 143 orc findings are `.md`, this is not an edge case; it is the common case
  for anything decided since about finding-018.
- **Cross-graph questions.** Q1, Q7, Q11, and Q12 have ground truth in both
  graphs. orient is cwd-scoped and must run twice with a manual merge; grep can
  search both roots at once but then ranks nothing. A verb that reads both graphs
  and returns one ranked list is doing work neither baseline does. (The graphs
  currently share ids across the boundary, for example `grv-kos-as-task-tracker`
  in both, which the verb has to disambiguate rather than dedupe.)

## The freshness and polarity gap

Sub-question 4 of the node asks whether the verb can rank a bedrock node above a
superseded one without a separate freshness model. The baselines set a low bar
and a real one. grep gives no tier signal, so on Q9 (git as substrate) it lists
the ruled-out graveyard node and the adopted bedrock storage model side by side,
and a hurried reader can take the wrong one as the answer. orient does better:
its tier sections keep ruled-out and adopted apart. So the verb's freshness
target is orient, not grep. Matching grep's flat list would be a regression
against the baseline that already exists.

## The false-confidence bar

Q14 and Q15 are the questions the graph cannot answer (a work-assignment lookup
and a live-session absence-detection). Here the baselines have an accidental
virtue: grep returns near-nothing and orient returns irrelevance, and because
neither claims to have answered, neither misleads. The verb has a way to be worse
than both, by returning a confident, well-ranked, wrong node. The node says this
outright: a ranked answer that presents an untested or off-topic finding as
authoritative is worse than grep, which at least hands over the raw file. So on
these two the verb has to clear a bar the baselines clear for free: say nothing
confident when there is nothing to say.

## Summary judgment

grep and orient already suffice on distinctive-phrase lookups and small-graveyard
ruled-out questions, and the verb should aim only not to regress there. They both
fail, in different ways, on the large middle: common-word and popular-substring
questions where the answer is buried (grep) or unranked and dump-scale (orient),
and on cross-graph questions that force two runs or a flat multi-root search.
They both fail outright on `.md` finding lookups, which orient cannot see and
grep reaches only by brute force, and that is the majority of the recent record.
The verb's clearest wins are therefore reaching `.md` findings, ranking within a
question, merging both graphs, and carrying tier for freshness. Its clearest risk
is the false-confidence pair, where the honest answer is the empty one the
baselines already give. If the verb ranks the buried middle, reaches the `.md`
record, and still declines to answer Q14 and Q15, it beats what exists. If it
wins the benchmark but sessions keep typing `rg`, the node has already ruled that
a loss.
