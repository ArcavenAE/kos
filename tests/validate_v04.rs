//! `kos validate` for the schema v0.4 fields (aae-orc-blj1w, PR 2 of the e689
//! chain; design docs/design/schema-v0.4.md sections 4 and 5, with the
//! architect's rulings of 2026-11-16).
//!
//! - `promoted_from` and `conditional_on` targets resolve exactly as edge
//!   targets do: a plain target found nowhere warns, a `graph::target` that
//!   does not exist fails, one naming a graph outside the workspace warns "not
//!   checked".
//! - A value outside the vocabulary of `examination`, `commitment`,
//!   `source_grade` or `provenance.kind` fails the node, naming the field, the
//!   raw value and the legal values (structural).
//! - A node whose `conditional_on` parent is undermined now gets a warning:
//!   parent confidence graveyard or placeholder, examination ruled-out, or
//!   commitment abandoned. The warning never changes the exit status. A
//!   stateless validate cannot see a change, so it warns on the state.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn kos(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kos"))
        .current_dir(dir)
        .args(args)
        .env_remove("KOS_WORKSPACE")
        .output()
        .expect("run kos")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn all(out: &Output) -> String {
    format!("{}{}", stdout(out), String::from_utf8_lossy(&out.stderr))
}

/// A standalone graph named `fixture`.
fn graph(root: &Path) {
    write(
        &root.join("_kos/kos.yaml"),
        "graph_id: fixture\nscope: repo\nschema_version: '0.4'\n",
    );
}

/// One node file in its confidence directory. `extra` is appended verbatim.
fn node(root: &Path, id: &str, confidence: &str, extra: &str) {
    write(
        &root.join(format!("_kos/nodes/{confidence}/{id}.yaml")),
        &format!(
            "id: {id}\ntype: element\nconfidence: {confidence}\ntitle: t\ncontent: c\n{extra}"
        ),
    );
}

fn depends_on(parent: &str) -> String {
    format!("conditional_on:\n  - target: {parent}\n")
}

fn promoted_from(target: &str) -> String {
    format!("provenance:\n  created_by: human\n  promoted_from:\n    - target: {target}\n")
}

fn validate(root: &Path) -> Output {
    kos(root, &["validate"])
}

#[test]
fn a_plain_promoted_from_or_conditional_on_target_found_nowhere_warns() {
    let tmp = tempfile::tempdir().unwrap();
    graph(tmp.path());
    node(
        tmp.path(),
        "elem-a",
        "frontier",
        &promoted_from("finding-nowhere"),
    );
    node(
        tmp.path(),
        "elem-b",
        "frontier",
        &depends_on("elem-nowhere"),
    );
    let out = validate(tmp.path());
    let text = stdout(&out);
    assert_eq!(out.status.code(), Some(0), "a warning never fails: {text}");
    assert!(text.contains("WARN  elem-a"), "{text}");
    assert!(
        text.contains("promoted_from target 'finding-nowhere' not found"),
        "{text}"
    );
    assert!(text.contains("WARN  elem-b"), "{text}");
    assert!(
        text.contains("conditional_on target 'elem-nowhere' not found"),
        "{text}"
    );
}

#[test]
fn a_graph_target_that_does_not_exist_fails_and_one_that_does_passes() {
    let tmp = tempfile::tempdir().unwrap();
    graph(tmp.path());
    node(tmp.path(), "elem-parent", "bedrock", "");
    node(
        tmp.path(),
        "elem-a",
        "frontier",
        &promoted_from("fixture::elem-nowhere"),
    );
    node(
        tmp.path(),
        "elem-b",
        "frontier",
        &depends_on("fixture::elem-nowhere"),
    );
    node(
        tmp.path(),
        "elem-c",
        "frontier",
        &promoted_from("fixture::elem-parent"),
    );
    node(
        tmp.path(),
        "elem-d",
        "frontier",
        &depends_on("fixture::elem-parent"),
    );
    let out = validate(tmp.path());
    let text = stdout(&out);
    assert_eq!(
        out.status.code(),
        Some(1),
        "a missing graph target gates: {text}"
    );
    assert!(text.contains("FAIL  elem-a"), "{text}");
    assert!(
        text.contains("promoted_from target 'fixture::elem-nowhere' does not resolve"),
        "{text}"
    );
    assert!(text.contains("FAIL  elem-b"), "{text}");
    assert!(
        text.contains("conditional_on target 'fixture::elem-nowhere' does not resolve"),
        "{text}"
    );
    assert!(
        text.contains("PASS  elem-c"),
        "a resolving target passes: {text}"
    );
    assert!(text.contains("PASS  elem-d"), "{text}");
}

#[test]
fn a_target_in_a_graph_outside_the_workspace_warns_not_checked() {
    let tmp = tempfile::tempdir().unwrap();
    graph(tmp.path());
    node(
        tmp.path(),
        "elem-a",
        "frontier",
        &promoted_from("elsewhere::elem-x"),
    );
    node(
        tmp.path(),
        "elem-b",
        "frontier",
        &depends_on("elsewhere::elem-x"),
    );
    let out = validate(tmp.path());
    let text = stdout(&out);
    assert_eq!(out.status.code(), Some(0), "{text}");
    assert!(
        text.contains("promoted_from target 'elsewhere::elem-x' was not checked"),
        "{text}"
    );
    assert!(
        text.contains("conditional_on target 'elsewhere::elem-x' was not checked"),
        "{text}"
    );
}

#[test]
fn a_value_outside_a_vocabulary_fails_the_node_naming_field_value_and_legal_values() {
    let cases: [(&str, &str, &str); 4] = [
        (
            "examination: maybe\n",
            "examination 'maybe' is not legal",
            "expected one of: un-examined, questioned, answered, ruled-out",
        ),
        (
            "commitment: sort-of\n",
            "commitment 'sort-of' is not legal",
            "expected one of: floating, load-bearing, essential, abandoned",
        ),
        (
            "source_grade: vibes\n",
            "source_grade 'vibes' is not legal",
            "expected one of: authored, compiler, heuristic, inferred",
        ),
        (
            "provenance:\n  kind: whim\n",
            "provenance.kind 'whim' is not legal",
            "expected one of: deliberated, emergent-ratified, inherited, pre-question",
        ),
    ];
    for (extra, named, legal) in cases {
        let tmp = tempfile::tempdir().unwrap();
        graph(tmp.path());
        node(tmp.path(), "elem-bad", "frontier", extra);
        let out = validate(tmp.path());
        let text = stdout(&out);
        assert_eq!(out.status.code(), Some(1), "{extra:?} gates: {text}");
        assert!(text.contains("FAIL  elem-bad"), "{extra:?}: {text}");
        assert!(
            text.contains(named),
            "{extra:?} names field and value: {text}"
        );
        assert!(
            text.contains(legal),
            "{extra:?} lists the legal values: {text}"
        );
    }
}

#[test]
fn a_dependent_of_an_undermined_parent_warns_naming_the_parent_and_the_reason() {
    // (parent confidence, parent extra fields, the reason the warning gives)
    let cases: [(&str, &str, &str); 4] = [
        ("graveyard", "", "confidence is graveyard"),
        ("placeholder", "", "confidence is placeholder"),
        (
            "frontier",
            "examination: ruled-out\n",
            "examination is ruled-out",
        ),
        (
            "frontier",
            "commitment: abandoned\n",
            "commitment is abandoned",
        ),
    ];
    for (confidence, extra, reason) in cases {
        let tmp = tempfile::tempdir().unwrap();
        graph(tmp.path());
        node(tmp.path(), "elem-parent", confidence, extra);
        node(
            tmp.path(),
            "elem-dep",
            "frontier",
            &depends_on("elem-parent"),
        );
        let out = validate(tmp.path());
        let text = stdout(&out);
        assert_eq!(
            out.status.code(),
            Some(0),
            "the dependent warning never changes the exit status ({reason}): {text}"
        );
        assert!(text.contains("WARN  elem-dep"), "{reason}: {text}");
        assert!(
            text.contains("conditional_on parent 'elem-parent' is undermined"),
            "{reason}: {text}"
        );
        assert!(text.contains(reason), "{reason}: {text}");
    }
}

#[test]
fn only_the_dependents_of_an_undermined_parent_warn() {
    let tmp = tempfile::tempdir().unwrap();
    graph(tmp.path());
    // Parents that are the normal case: frontier and questioned, bedrock, and
    // one with no v0.4 fields at all.
    node(
        tmp.path(),
        "elem-frontier",
        "frontier",
        "examination: questioned\ncommitment: load-bearing\n",
    );
    node(
        tmp.path(),
        "elem-bedrock",
        "bedrock",
        "examination: answered\ncommitment: essential\n",
    );
    node(tmp.path(), "elem-bare", "frontier", "");
    node(tmp.path(), "elem-dead", "graveyard", "");
    for (dep, parent) in [
        ("dep-frontier", "elem-frontier"),
        ("dep-bedrock", "elem-bedrock"),
        ("dep-bare", "elem-bare"),
        ("dep-dead", "elem-dead"),
    ] {
        node(tmp.path(), dep, "frontier", &depends_on(parent));
    }
    let out = validate(tmp.path());
    let text = stdout(&out);
    assert_eq!(out.status.code(), Some(0), "{text}");
    assert!(
        text.contains("WARN  dep-dead"),
        "the undermined one warns: {text}"
    );
    for quiet in ["dep-frontier", "dep-bedrock", "dep-bare"] {
        assert!(
            text.contains(&format!("PASS  {quiet}")),
            "{quiet} has a sound parent and must pass: {text}"
        );
    }
}

#[test]
fn a_dependent_lists_each_undermined_parent_and_not_the_sound_one() {
    let tmp = tempfile::tempdir().unwrap();
    graph(tmp.path());
    node(tmp.path(), "elem-dead", "graveyard", "");
    node(
        tmp.path(),
        "elem-ruled",
        "frontier",
        "examination: ruled-out\n",
    );
    node(tmp.path(), "elem-sound", "bedrock", "");
    node(
        tmp.path(),
        "elem-dep",
        "frontier",
        "conditional_on:\n  - target: elem-dead\n  - target: elem-sound\n  - target: elem-ruled\n",
    );
    let text = stdout(&validate(tmp.path()));
    assert!(
        text.contains("conditional_on parent 'elem-dead' is undermined"),
        "{text}"
    );
    assert!(
        text.contains("conditional_on parent 'elem-ruled' is undermined"),
        "{text}"
    );
    assert!(!text.contains("parent 'elem-sound'"), "{text}");
}

#[test]
fn a_parent_with_a_value_outside_the_vocabulary_fails_itself_and_does_not_warn_its_dependent() {
    let tmp = tempfile::tempdir().unwrap();
    graph(tmp.path());
    node(
        tmp.path(),
        "elem-parent",
        "frontier",
        "examination: maybe\n",
    );
    node(
        tmp.path(),
        "elem-dep",
        "frontier",
        &depends_on("elem-parent"),
    );
    let out = validate(tmp.path());
    let text = stdout(&out);
    assert_eq!(out.status.code(), Some(1), "{text}");
    assert!(text.contains("FAIL  elem-parent"), "{text}");
    assert!(
        text.contains("PASS  elem-dep"),
        "an unknown value is not an undermined state: {text}"
    );
}

#[test]
fn a_parent_the_graph_does_not_hold_gives_no_dependent_warning() {
    // A finding, or a node in another graph, resolves for existence only: its
    // state is not read, so there is nothing to warn about beyond check 3.
    let tmp = tempfile::tempdir().unwrap();
    graph(tmp.path());
    write(
        &tmp.path().join("_kos/findings/finding-001-first.md"),
        "# first\n\nbody\n",
    );
    node(
        tmp.path(),
        "elem-dep",
        "frontier",
        &depends_on("finding-001"),
    );
    node(
        tmp.path(),
        "elem-far",
        "frontier",
        &depends_on("elsewhere::elem-x"),
    );
    let text = stdout(&validate(tmp.path()));
    assert!(text.contains("PASS  elem-dep"), "{text}");
    assert!(!text.contains("is undermined"), "{text}");
    // The far target still reports its own check-3 result, which is what makes
    // this a statement about the dependent warning and not about silence.
    assert!(
        text.contains("conditional_on target 'elsewhere::elem-x' was not checked"),
        "{text}"
    );
}

#[test]
fn promote_refuses_a_node_with_a_value_outside_a_vocabulary_and_moves_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    graph(tmp.path());
    node(tmp.path(), "question-x", "frontier", "examination: maybe\n");
    fs::create_dir_all(tmp.path().join("_kos/nodes/bedrock")).unwrap();
    let out = kos(tmp.path(), &["promote", "question-x", "bedrock"]);
    let text = all(&out);
    assert!(!out.status.success(), "must be refused:\n{text}");
    assert!(text.contains("examination 'maybe' is not legal"), "{text}");
    assert!(
        tmp.path()
            .join("_kos/nodes/frontier/question-x.yaml")
            .exists()
    );
    assert!(
        !tmp.path()
            .join("_kos/nodes/bedrock/question-x.yaml")
            .exists()
    );
}

#[test]
fn the_design_says_validate_warns_on_the_parents_state_and_names_its_limit() {
    let text = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/design/schema-v0.4.md"),
    )
    .unwrap();
    assert!(
        !text.contains("When a target's confidence or coordinate changes"),
        "a stateless validate cannot see a change"
    );
    assert!(text.contains("undermined"), "names the state it warns on");
    assert!(
        text.contains("change detection"),
        "says change detection is left to a later ticket"
    );
    assert!(
        text.contains("sibling graph"),
        "names the limit: a parent outside the checked graph is resolved for existence only"
    );
}
