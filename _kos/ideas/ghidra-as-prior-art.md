# Ghidra as prior art for representing and navigating code at scale

*Idea. Generative, uncommitted. Unexamined.*

## The operator's question

> my question for ghidra relates to the problem domain, and data structures, algorithms, organization, navigation of code. it's space is navigating compiled code, reversing into source. consider the relationship between that domain and a code graph, there is potential for benefits and borrowing advanced techniques for navigating and representing code at scale, efficiently, even though one goes one direction and the other comes the other direction, they are related problems in an area that is grounded in years of engineering experience

## Read this first

- This note is unexamined. Nobody has tested any point in it.
- None of it applies to any build or to any schema now. No node, finding or code change follows from it.
- The points below are from director's general knowledge of Ghidra. They are unverified against Ghidra's current source and documentation, so each may be out of date or wrong.
- Ghidra works from compiled code back toward source. The code graph arc works from source forward. The note records where the two problems look alike, and nothing in it claims they are the same problem.

## Candidate techniques

### Representation

- One location type that every view speaks. Ghidra's views share an address and a ProgramLocation, so a click in one view means the same thing in every other.
- Range maps for properties, instead of one row per item. A property that holds over a span of addresses is stored once for the span.
- Typed references: call, read, write, flow and data, each indexed in both directions, and each carrying a source grade. The source grade is Ghidra's SourceType (default, analysis, imported, user-defined), where a lower grade cannot overwrite a higher one. It is possible prior art for keeping how a fact was obtained apart from how far it is trusted.
- Several program trees over one address set, so one body of code can be organized more than one way without copying it.
- Shared type and library archives, so a type or a library signature is defined once and used by many programs.

### Algorithms

- Graphs built on demand from stored facts, using block and call models, instead of stored as graphs. This looks close to the hypothesis in the fact store question (question-kos-fact-store-substrate): a versioned store of facts below, and traversal built above it.
- One intermediate representation across languages (SLEIGH into p-code), so analysis is written once.
- Prioritized analyzers that re-run only on changed ranges, instead of re-analyzing a whole program after an edit.
- Version Tracking correlators, which decide whether an entity in one version is the same entity in another.
- BSim feature vectors with locality-sensitive hashing, and Function ID hashes, for finding similar functions at scale.

### Navigation

- Cross-linked views on one cursor, with history, so moving in one view moves the others and a path can be retraced.
- Incoming and outgoing call trees that expand lazily, so a large graph is explored without being built first.

## The gap this names

The code graph arc does not yet address the identity of a symbol across commits, such as a rename or a move. Version Tracking correlators are one place to look at how that has been handled.

## Costs, as raised

- Ghidra is heavy: it runs on a JVM.
- Decompiler output is analysis. It is never compiler truth, and a fact that rests on it should say so.
- The license is Apache-2.0.

## Earlier reading, superseded

The first draft of this note read the question as one about using Ghidra's outputs as facts. It listed binary-derived facts about what ships (dead code, inlining, what was linked, whether a release matches its tag), facts for code that exists only as a binary, and Ghidra Server's versioned multi-user project as prior art for question-kos-multi-writer-concurrency. The operator says that reading missed the question. Those points are named here only for the record, and carry the same caveats.
