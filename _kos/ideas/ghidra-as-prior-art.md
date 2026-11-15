# Ghidra as prior art for kos

*Idea. Generative, uncommitted. Unexamined.*

## Read this first

- This note is unexamined. Nobody has tested any point in it.
- The operator says the question behind it is not yet understood. The notes below record what was raised and nothing more.
- None of it applies to any build or to any schema now. No node, finding or code change follows from it.
- The points come from director's general knowledge of Ghidra. They were not checked against a current Ghidra release, so each may be out of date or wrong.
- Current focus stays on the work already set out in the graph. This file only keeps the points from getting lost.

## The points as raised

### Source grade of a fact

Ghidra records where each symbol name came from, as a SourceType: default, analysis, imported or user-defined. A name from a lower grade cannot overwrite one from a higher grade. That is a possible prior art for keeping source grade apart from confidence tier in kos: how a fact was obtained, and how far it is trusted, as two separate axes.

### Facts about what ships

Binary-derived facts describe what was actually built: dead code, what the compiler inlined, what was linked, and whether a release matches its tag. Source-level facts cannot say any of these.

### Code with no source

For code that exists only as a binary, Ghidra's analysis of Go binaries is one route to facts at all. That gives kos a way to hold facts about code nobody can read in source form.

### Comparing versions

BSim and Version Tracking are Ghidra's tools for finding similar functions and for matching two builds of a program. They look like binary-level analogues of the commit-to-commit fact diff that kos does over source.

### Concurrent writers

Ghidra Server hosts a versioned project that several users edit. It is possible prior art for the open question question-kos-multi-writer-concurrency.

## Costs, as raised

- Ghidra is heavy: it runs on a JVM.
- Decompiler output is analysis. It is never compiler truth, and a fact that rests on it should say so.
- The license is Apache-2.0.

## What would have to be true before this goes anywhere

The operator would first need to say what the original question was. Until then there is nothing to probe, and this file stays an idea.
