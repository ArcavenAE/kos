//! Node schema v0.4, PR 1 of the e689 chain (aae-orc-kv2rv). The new fields are
//! all optional, so every v0.3 graph parses as before. The design is
//! docs/design/schema-v0.4.md. kos has no Node writer (promote and compact
//! edit the text in place), so "write" here means the writers kos has: the
//! two commands must leave a v0.4 node's new fields byte-identical, and each
//! new type must serialize and re-parse to an equal value.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use kos::model::{Commitment, Examination, Node, NodeRef, ProvenanceKind, SourceGrade};

const V03: &str = "id: elem-old\ntype: element\nconfidence: bedrock\ntitle: old\n\
     content: a v0.3 body\n";

const V04: &str = "id: question-new\ntype: question\nconfidence: frontier\ntitle: new\n\
     content: a much longer v0.4 body of text that is larger than a short summary\n\
     examination: questioned\n\
     commitment: load-bearing\n\
     source_grade: compiler\n\
     conditional_on:\n\
     \x20 - target: elem-premise\n\
     \x20   note: holds while the index is current\n\
     \x20 - target: other-graph::elem-far\n\
     provenance:\n\
     \x20 created_by: human\n\
     \x20 kind: emergent-ratified\n\
     \x20 falsifier: a second index run that no longer lists the symbol\n\
     \x20 promoted_from:\n\
     \x20   - target: finding-045\n\
     \x20     note: the sketch\n\
     \x20   - target: adr-009\n";

fn parse(text: &str) -> Node {
    kos::yaml::from_str::<Node>(text).unwrap_or_else(|e| panic!("must parse: {e}\n{text}"))
}

fn r(target: &str, note: Option<&str>) -> NodeRef {
    NodeRef {
        target: target.to_string(),
        note: note.map(str::to_string),
    }
}

#[test]
fn a_v04_node_carries_every_new_field() {
    let n = parse(V04);
    assert_eq!(n.examination, Some(Examination::Questioned));
    assert_eq!(n.commitment, Some(Commitment::LoadBearing));
    assert_eq!(n.source_grade, Some(SourceGrade::Compiler));
    assert_eq!(
        n.conditional_on,
        vec![
            r("elem-premise", Some("holds while the index is current")),
            r("other-graph::elem-far", None)
        ]
    );
    let p = n.provenance.expect("provenance block");
    assert_eq!(p.created_by.as_deref(), Some("human"));
    assert_eq!(p.kind, Some(ProvenanceKind::EmergentRatified));
    assert_eq!(
        p.falsifier.as_deref(),
        Some("a second index run that no longer lists the symbol")
    );
    assert_eq!(
        p.promoted_from,
        vec![r("finding-045", Some("the sketch")), r("adr-009", None)]
    );
}

#[test]
fn a_v03_node_has_every_new_field_absent_where_a_v04_node_has_them() {
    let old = parse(V03);
    assert_eq!(old.examination, None);
    assert_eq!(old.commitment, None);
    assert_eq!(
        old.source_grade, None,
        "absent means authored, kept as None"
    );
    assert!(old.conditional_on.is_empty());
    assert!(old.provenance.is_none());
    // The same fields on a v0.4 node are present: the pair shows absence is
    // the file's, not the reader's.
    let new = parse(V04);
    assert!(new.examination.is_some() && !new.conditional_on.is_empty());
}

#[test]
fn a_v03_provenance_block_has_the_new_provenance_fields_absent() {
    let text = format!("{V03}provenance:\n  created_by: human\n  reviewed_by: null\n");
    let p = parse(&text).provenance.unwrap();
    assert_eq!(p.kind, None);
    assert!(p.promoted_from.is_empty());
    assert_eq!(p.falsifier, None);
    // And a v0.4 block parses with them present.
    assert!(parse(V04).provenance.unwrap().kind.is_some());
}

#[test]
fn a_field_the_binary_does_not_know_still_parses_beside_the_new_ones() {
    let text = format!("{V04}future_field: something later\nout_of_scope:\n  - x\n");
    let n = parse(&text);
    assert_eq!(n.examination, Some(Examination::Questioned));
    assert_eq!(n.id, "question-new");
}

#[test]
fn a_value_outside_the_vocabulary_parses_as_unknown_and_keeps_the_raw_text() {
    let text = format!(
        "{V03}examination: maybe\ncommitment: sort-of\nsource_grade: vibes\n\
         provenance:\n  kind: whim\n"
    );
    let n = parse(&text);
    assert_eq!(n.examination, Some(Examination::Unknown("maybe".into())));
    assert_eq!(n.commitment, Some(Commitment::Unknown("sort-of".into())));
    assert_eq!(n.source_grade, Some(SourceGrade::Unknown("vibes".into())));
    assert_eq!(
        n.provenance.unwrap().kind,
        Some(ProvenanceKind::Unknown("whim".into()))
    );
}

/// Serialize one value, check the scalar is the vocabulary word, and read it
/// back to an equal value.
macro_rules! round_trip {
    ($ty:ty, $value:expr, $word:expr) => {{
        let value: $ty = $value;
        let text = serde_saphyr::to_string(&value).expect("serializes");
        assert_eq!(text.trim(), $word, "{value:?} writes its vocabulary word");
        let back: $ty = kos::yaml::from_str(&text).expect("re-parses");
        assert_eq!(back, value, "{value:?} survives a write and a read");
    }};
}

#[test]
fn examination_serializes_and_re_parses_to_an_equal_value() {
    round_trip!(Examination, Examination::UnExamined, "un-examined");
    round_trip!(Examination, Examination::Questioned, "questioned");
    round_trip!(Examination, Examination::Answered, "answered");
    round_trip!(Examination, Examination::RuledOut, "ruled-out");
    round_trip!(Examination, Examination::Unknown("maybe".into()), "maybe");
}

#[test]
fn commitment_serializes_and_re_parses_to_an_equal_value() {
    round_trip!(Commitment, Commitment::Floating, "floating");
    round_trip!(Commitment, Commitment::LoadBearing, "load-bearing");
    round_trip!(Commitment, Commitment::Essential, "essential");
    round_trip!(Commitment, Commitment::Abandoned, "abandoned");
    round_trip!(Commitment, Commitment::Unknown("sort-of".into()), "sort-of");
}

#[test]
fn provenance_kind_serializes_and_re_parses_to_an_equal_value() {
    round_trip!(ProvenanceKind, ProvenanceKind::Deliberated, "deliberated");
    round_trip!(
        ProvenanceKind,
        ProvenanceKind::EmergentRatified,
        "emergent-ratified"
    );
    round_trip!(ProvenanceKind, ProvenanceKind::Inherited, "inherited");
    round_trip!(ProvenanceKind, ProvenanceKind::PreQuestion, "pre-question");
    round_trip!(
        ProvenanceKind,
        ProvenanceKind::Unknown("whim".into()),
        "whim"
    );
}

#[test]
fn source_grade_serializes_and_re_parses_to_an_equal_value() {
    round_trip!(SourceGrade, SourceGrade::Authored, "authored");
    round_trip!(SourceGrade, SourceGrade::Compiler, "compiler");
    round_trip!(SourceGrade, SourceGrade::Heuristic, "heuristic");
    round_trip!(SourceGrade, SourceGrade::Inferred, "inferred");
    round_trip!(SourceGrade, SourceGrade::Unknown("vibes".into()), "vibes");
}

#[test]
fn a_node_ref_round_trips_and_writes_no_note_key_when_there_is_none() {
    let with = r("elem-premise", Some("a reason"));
    let text = serde_saphyr::to_string(&with).unwrap();
    assert!(text.contains("target: elem-premise"), "{text}");
    assert!(text.contains("note: a reason"), "{text}");
    assert_eq!(kos::yaml::from_str::<NodeRef>(&text).unwrap(), with);
    let bare = r("elem-premise", None);
    let text = serde_saphyr::to_string(&bare).unwrap();
    assert!(text.contains("target: elem-premise"), "{text}");
    assert!(!text.contains("note"), "None stays absent: {text}");
    assert_eq!(kos::yaml::from_str::<NodeRef>(&text).unwrap(), bare);
}

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn kos_in(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kos"))
        .args(args)
        .current_dir(cwd)
        .env_remove("KOS_WORKSPACE")
        .output()
        .expect("run kos")
}

fn graph(root: &Path) {
    write(
        &root.join("_kos/kos.yaml"),
        "graph_id: kos\nscope: repo\nschema_version: '0.3'\n",
    );
    write(&root.join("_kos/nodes/frontier/question-new.yaml"), V04);
    write(
        &root.join("_kos/nodes/bedrock/elem-premise.yaml"),
        "id: elem-premise\ntype: element\nconfidence: bedrock\ntitle: p\ncontent: c\n",
    );
    write(&root.join("summary.txt"), "short summary\n");
}

/// The v0.4 field lines of `V04`, from `examination:` to the end, in order.
fn field_block() -> Vec<&'static str> {
    V04.lines()
        .skip_while(|l| !l.starts_with("examination:"))
        .collect()
}

/// The field block appears in `text` whole: contiguous and in order, with no
/// line between or reordered.
fn assert_block_intact(text: &str) {
    let lines: Vec<&str> = text.lines().collect();
    let block = field_block();
    let start = lines
        .iter()
        .position(|l| *l == block[0])
        .unwrap_or_else(|| panic!("no examination line in:\n{text}"));
    let found = lines.get(start..start + block.len());
    assert_eq!(
        found,
        Some(block.as_slice()),
        "the v0.4 field lines stay contiguous and in order:\n{text}"
    );
}

#[test]
fn promote_leaves_the_v04_fields_byte_identical_and_the_node_still_reads_them() {
    let tmp = tempfile::tempdir().unwrap();
    graph(tmp.path());
    let out = kos_in(tmp.path(), &["promote", "question-new", "bedrock"]);
    assert!(
        out.status.success(),
        "promote: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let moved =
        fs::read_to_string(tmp.path().join("_kos/nodes/bedrock/question-new.yaml")).unwrap();
    assert_eq!(
        moved,
        V04.replacen("confidence: frontier\n", "confidence: bedrock\n", 1),
        "only the confidence line changes"
    );
    assert_block_intact(&moved);
    let n = parse(&moved);
    assert_eq!(n.commitment, Some(Commitment::LoadBearing));
    assert_eq!(n.conditional_on.len(), 2);
}

#[test]
fn compact_leaves_the_v04_fields_byte_identical_and_the_node_still_reads_them() {
    let tmp = tempfile::tempdir().unwrap();
    graph(tmp.path());
    let summary = tmp.path().join("summary.txt");
    let out = kos_in(
        tmp.path(),
        &[
            "compact",
            "--node",
            "question-new",
            "--summary",
            summary.to_str().unwrap(),
        ],
    );
    assert!(
        out.status.success(),
        "compact: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let after =
        fs::read_to_string(tmp.path().join("_kos/nodes/frontier/question-new.yaml")).unwrap();
    assert!(after.contains("[COMPACTED]"), "compaction ran:\n{after}");
    assert_block_intact(&after);
    let n = parse(&after);
    assert_eq!(n.examination, Some(Examination::Questioned));
    assert_eq!(n.provenance.unwrap().promoted_from.len(), 2);
}

#[test]
fn the_schema_file_declares_v04_and_documents_each_new_field() {
    let text =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("schema/node.schema.yaml"))
            .unwrap();
    assert!(text.contains("# Version: 0.4"), "version line");
    assert!(text.contains("0.4 - aae-orc-kv2rv:"), "history entry");
    for field in [
        "examination:",
        "commitment:",
        "source_grade:",
        "conditional_on:",
        "kind:",
        "promoted_from:",
        "falsifier:",
    ] {
        assert!(
            text.lines().any(|l| l.trim_start().starts_with(field)),
            "schema documents {field}"
        );
    }
    for word in [
        "un-examined",
        "questioned",
        "answered",
        "ruled-out",
        "floating",
        "load-bearing",
        "essential",
        "abandoned",
        "deliberated",
        "emergent-ratified",
        "inherited",
        "pre-question",
        "authored",
        "compiler",
        "heuristic",
        "inferred",
    ] {
        assert!(text.contains(word), "schema lists the value {word}");
    }
}

#[test]
fn the_v04_history_entry_has_no_em_dash() {
    let text =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("schema/node.schema.yaml"))
            .unwrap();
    let lines: Vec<&str> = text.lines().collect();
    let start = lines
        .iter()
        .position(|l| l.contains("aae-orc-kv2rv:"))
        .expect("the 0.4 history entry names its ticket");
    // The entry is the line that names the ticket and its comment continuation.
    for line in lines[start..]
        .iter()
        .take_while(|l| l.starts_with('#'))
        .take(4)
    {
        assert!(
            !line.contains('\u{2014}'),
            "no em dash in the 0.4 entry: {line:?}"
        );
    }
}
