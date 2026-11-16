//! Export, import and harvest for the fact store (aae-orc-xxe3n, PR 2).

use std::collections::BTreeMap;

use kos::error::KosError;
use kos::facts::{Fact, FactStore, Harvest, Version};

fn v(n: i64) -> Version {
    Version::new(n)
}

fn f(e: &str, a: &str, val: &str) -> Fact {
    Fact::new(e, a, val)
}

fn source(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, val)| ((*k).to_string(), (*val).to_string()))
        .collect()
}

const HEADER_EMPTY: &str = r#"{"kos_facts_export":1,"schema_version":1,"latest_version":null}"#;

/// An export file for the import tests: a header with `latest`, then `rows`.
fn file(latest: i64, rows: &[&str]) -> String {
    let mut text =
        format!(r#"{{"kos_facts_export":1,"schema_version":1,"latest_version":{latest}}}"#);
    text.push('\n');
    for row in rows {
        text.push_str(row);
        text.push('\n');
    }
    text
}

fn row(e: &str, from: i64, to: Option<i64>) -> String {
    let to = to.map_or_else(|| "null".to_string(), |n| n.to_string());
    format!(r#"{{"e":"{e}","a":"r","v":"1","valid_from":{from},"valid_to":{to}}}"#)
}

/// What the importer must say for a rule, as a word in the message.
fn refused_at(text: &str, line: usize, word: &str) {
    let mut store = FactStore::open_in_memory().unwrap();
    let result = store.import_jsonl(text.as_bytes());
    let shown = format!("{result:?}");
    let hit = match result {
        Err(KosError::FactImportLine { line, message }) => Some((line, message)),
        _ => None,
    };
    assert!(
        hit.is_some(),
        "expected FactImportLine at line {line} naming {word:?}, got {shown}"
    );
    let (got, message) = hit.unwrap();
    assert_eq!(got, line, "line of the refusal: {message}");
    assert!(
        message.contains(word),
        "message should say {word:?}, got {message:?}"
    );
    assert_eq!(store.latest_version().unwrap(), None, "store stays empty");
    assert_eq!(store.facts_as_of(v(99)).unwrap(), Vec::<Fact>::new());
}

fn repo_source() -> BTreeMap<String, String> {
    source(&[("repo", "demo/repo"), ("commit", "abc1234")])
}

fn export(store: &FactStore) -> String {
    let mut out = Vec::new();
    store.export_jsonl(&mut out).unwrap();
    String::from_utf8(out).unwrap()
}

/// A deterministic 30-version log, the same generator the core tests use.
fn long_log() -> FactStore {
    let mut store = FactStore::open_in_memory().unwrap();
    let mut live: std::collections::BTreeSet<Fact> = std::collections::BTreeSet::new();
    let mut seed: u64 = 20_261_116;
    let mut next = move || {
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        seed >> 33
    };
    for version in 1..=30_i64 {
        let mut add = Vec::new();
        let mut del = Vec::new();
        for _ in 0..12 {
            let fact = f(
                &format!("e{}", next() % 8),
                if next() % 2 == 0 {
                    "DEFINES"
                } else {
                    "REFERENCES"
                },
                &format!("s{}", next() % 8),
            );
            if live.contains(&fact) {
                if next() % 2 == 0 {
                    del.push(fact);
                }
            } else {
                add.push(fact);
            }
        }
        add.sort();
        add.dedup();
        del.sort();
        del.dedup();
        store.retract_facts(v(version), &del).unwrap();
        store.assert_facts(v(version), &add).unwrap();
        for d in &del {
            live.remove(d);
        }
        for a in &add {
            live.insert(a.clone());
        }
    }
    store
}

#[test]
fn export_then_import_reproduces_every_version_and_every_diff() {
    let source_store = long_log();
    let text = export(&source_store);
    let lines = text.lines().count();
    assert!(
        lines > 1,
        "the export holds a header and rows, got {lines} lines"
    );
    let mut copy = FactStore::open_in_memory().unwrap();
    let rows = copy.import_jsonl(text.as_bytes()).unwrap();
    assert_eq!(
        rows,
        lines - 1,
        "rows read equal lines written, less the header"
    );
    assert_eq!(copy.latest_version().unwrap(), Some(v(30)));
    for n in 0..=31 {
        assert_eq!(
            copy.facts_as_of(v(n)).unwrap(),
            source_store.facts_as_of(v(n)).unwrap(),
            "fact set at version {n}"
        );
    }
    for from in 0..=30 {
        for to in 0..=30 {
            assert_eq!(
                copy.diff(v(from), v(to)).unwrap(),
                source_store.diff(v(from), v(to)).unwrap(),
                "diff {from} to {to}"
            );
        }
    }
}

#[test]
fn export_is_sorted_and_the_same_bytes_every_time() {
    let mut store = FactStore::open_in_memory().unwrap();
    store
        .assert_facts(v(1), &[f("b", "r", "1"), f("a", "r", "2")])
        .unwrap();
    store.retract_facts(v(3), &[f("a", "r", "2")]).unwrap();
    store.assert_facts(v(5), &[f("a", "r", "2")]).unwrap();
    let first = export(&store);
    assert_eq!(first, export(&store));
    let want = concat!(
        r#"{"kos_facts_export":1,"schema_version":1,"latest_version":5}"#,
        "\n",
        r#"{"e":"a","a":"r","v":"2","valid_from":1,"valid_to":3}"#,
        "\n",
        r#"{"e":"a","a":"r","v":"2","valid_from":5,"valid_to":null}"#,
        "\n",
        r#"{"e":"b","a":"r","v":"1","valid_from":1,"valid_to":null}"#,
        "\n",
    );
    assert_eq!(first, want);
}

#[test]
fn import_refuses_a_store_that_already_holds_facts() {
    let mut full = FactStore::open_in_memory().unwrap();
    full.assert_facts(v(1), &[f("a", "r", "b")]).unwrap();
    let text = export(&long_log());
    let result = full.import_jsonl(text.as_bytes());
    assert!(
        matches!(result, Err(KosError::FactImportNotEmpty)),
        "expected FactImportNotEmpty, got {result:?}"
    );
    assert_eq!(full.facts_as_of(v(9)).unwrap(), vec![f("a", "r", "b")]);
    assert_eq!(full.latest_version().unwrap(), Some(v(1)));
}

#[test]
fn import_names_the_line_of_a_malformed_row_and_keeps_the_store_empty() {
    let text = format!("{}this is not json\n", file(1, &[&row("a", 1, None)]));
    let mut store = FactStore::open_in_memory().unwrap();
    let result = store.import_jsonl(text.as_bytes());
    assert!(
        matches!(result, Err(KosError::FactImportLine { line: 3, .. })),
        "expected FactImportLine at line 3, got {result:?}"
    );
    assert_eq!(store.facts_as_of(v(9)).unwrap(), Vec::<Fact>::new());
    assert_eq!(store.latest_version().unwrap(), None);
}

#[test]
fn import_refuses_a_file_without_the_header_or_with_an_unknown_export_number() {
    for (text, what) in [
        (format!("{}\n", row("a", 1, None)), "no header"),
        (
            "{\"kos_facts_export\":2,\"schema_version\":1,\"latest_version\":1}\n".to_string(),
            "export number 2",
        ),
        (String::new(), "an empty file"),
    ] {
        let mut store = FactStore::open_in_memory().unwrap();
        let result = store.import_jsonl(text.as_bytes());
        assert!(
            matches!(result, Err(KosError::FactImportHeader { .. })),
            "expected FactImportHeader for {what}, got {result:?}"
        );
        assert_eq!(store.latest_version().unwrap(), None);
    }
}

#[test]
fn a_write_that_changes_nothing_still_moves_latest_version_through_export_and_import() {
    let mut store = FactStore::open_in_memory().unwrap();
    store.assert_facts(v(2), &[f("a", "r", "b")]).unwrap();
    // Asserting a live fact changes no row but moves latest_version to 6.
    assert_eq!(store.assert_facts(v(6), &[f("a", "r", "b")]).unwrap(), 0);
    assert_eq!(store.latest_version().unwrap(), Some(v(6)));
    let text = export(&store);
    assert!(
        text.starts_with(r#"{"kos_facts_export":1,"schema_version":1,"latest_version":6}"#),
        "header carries latest_version 6, got {text:?}"
    );
    let mut copy = FactStore::open_in_memory().unwrap();
    copy.import_jsonl(text.as_bytes()).unwrap();
    assert_eq!(copy.latest_version().unwrap(), Some(v(6)));
    // The copy refuses what the source refuses.
    let late = copy.assert_facts(v(5), &[f("c", "r", "d")]);
    assert!(
        matches!(late, Err(KosError::FactVersionOrder { got: 5, latest: 6 })),
        "copy must refuse a write behind 6, got {late:?}"
    );
}

#[test]
fn an_empty_store_exports_only_the_header_and_imports_back_empty() {
    let store = FactStore::open_in_memory().unwrap();
    assert_eq!(export(&store), format!("{HEADER_EMPTY}\n"));
    let mut copy = FactStore::open_in_memory().unwrap();
    assert_eq!(copy.import_jsonl(export(&store).as_bytes()).unwrap(), 0);
    assert_eq!(copy.latest_version().unwrap(), None);
}

// The next five tests pin one import rule each. The message must contain the
// quoted word so a refusal names its rule.

#[test]
fn import_refuses_an_interval_that_does_not_end_after_it_starts() {
    // "greater": valid_to must be greater than valid_from.
    refused_at(&file(9, &[&row("a", 4, Some(4))]), 2, "greater");
    refused_at(&file(9, &[&row("a", 4, Some(3))]), 2, "greater");
}

#[test]
fn import_refuses_overlapping_intervals_of_one_fact() {
    // "overlap": the second interval starts before the first ends.
    refused_at(
        &file(9, &[&row("a", 1, Some(5)), &row("a", 3, Some(7))]),
        3,
        "overlap",
    );
}

#[test]
fn import_refuses_more_than_one_live_interval_of_one_fact() {
    // "live": the first interval is still live when the second begins.
    refused_at(
        &file(9, &[&row("a", 1, None), &row("a", 4, None)]),
        3,
        "live",
    );
}

#[test]
fn import_refuses_rows_that_are_not_in_the_stated_order() {
    // "order": b sorts after a, so a row for b before a row for a is out of order.
    refused_at(
        &file(9, &[&row("b", 1, None), &row("a", 1, None)]),
        3,
        "order",
    );
}

#[test]
fn import_refuses_an_interval_that_ends_after_latest_version() {
    // "latest_version": the header says 4, the row ends at 9.
    refused_at(&file(4, &[&row("a", 1, Some(9))]), 2, "latest_version");
}

#[test]
fn harvest_writes_the_kind_the_source_and_sorted_facts() {
    let mut store = FactStore::open_in_memory().unwrap();
    store
        .assert_facts(
            v(2),
            &[
                f("b.go", "DEFINES", "sym `b`/F()."),
                f("a.go", "REFERENCES", "sym: a"),
                f("c.go", "DEFINES", "unpicked"),
            ],
        )
        .unwrap();
    let src = source(&[
        ("repo", "demo/repo"),
        ("commit", "d0616f98"),
        ("version", "2"),
    ]);
    let yaml = store
        .harvest(
            v(2),
            &[
                f("b.go", "DEFINES", "sym `b`/F()."),
                f("a.go", "REFERENCES", "sym: a"),
            ],
            &src,
        )
        .unwrap();
    assert!(
        yaml.starts_with("kind: harvested-facts\n"),
        "file must open with its kind, got: {yaml:?}"
    );
    let parsed = Harvest::parse(&yaml).unwrap();
    assert_eq!(parsed.source, src);
    assert_eq!(
        parsed.facts,
        vec![
            f("a.go", "REFERENCES", "sym: a"),
            f("b.go", "DEFINES", "sym `b`/F().")
        ]
    );
    assert!(
        yaml.find("a.go").unwrap() < yaml.find("b.go").unwrap(),
        "facts are written sorted"
    );
    assert!(!yaml.contains("unpicked"), "only the picks are written");
}

#[test]
fn harvest_refuses_a_fact_that_does_not_hold_at_the_version() {
    let mut store = FactStore::open_in_memory().unwrap();
    store.assert_facts(v(1), &[f("a", "r", "b")]).unwrap();
    store.retract_facts(v(3), &[f("a", "r", "b")]).unwrap();
    let gone = store.harvest(v(3), &[f("a", "r", "b")], &repo_source());
    assert!(
        matches!(
            gone,
            Err(KosError::FactHarvestNotLive { ref e, version: 3, .. }) if e == "a"
        ),
        "a retracted fact must be refused, got {gone:?}"
    );
    let never = store.harvest(v(2), &[f("x", "r", "y")], &repo_source());
    assert!(
        matches!(never, Err(KosError::FactHarvestNotLive { .. })),
        "a fact never asserted must be refused, got {never:?}"
    );
}

#[test]
fn a_harvest_applies_into_another_store_and_a_second_apply_changes_nothing() {
    let mut store = FactStore::open_in_memory().unwrap();
    store
        .assert_facts(
            v(2),
            &[f("a", "r", "1"), f("b", "r", "2"), f("c", "r", "3")],
        )
        .unwrap();
    let yaml = store
        .harvest(
            v(2),
            &[f("a", "r", "1"), f("c", "r", "3")],
            &source(&[
                ("repo", "demo/repo"),
                ("commit", "abc1234"),
                ("version", "2"),
            ]),
        )
        .unwrap();
    let mut target = FactStore::open_in_memory().unwrap();
    assert_eq!(target.apply_harvest(v(7), &yaml).unwrap(), 2);
    assert_eq!(
        target.facts_as_of(v(7)).unwrap(),
        vec![f("a", "r", "1"), f("c", "r", "3")]
    );
    // The source says version 2; the apply call's version, 7, is the one written.
    assert_eq!(target.facts_as_of(v(6)).unwrap(), Vec::<Fact>::new());
    assert_eq!(target.facts_as_of(v(2)).unwrap(), Vec::<Fact>::new());
    assert_eq!(target.apply_harvest(v(8), &yaml).unwrap(), 0);
    assert_eq!(target.diff(v(7), v(8)).unwrap().added, Vec::<Fact>::new());
}

#[test]
fn a_file_with_another_kind_or_no_kind_is_refused() {
    let other = "kind: something-else\nsource: {}\nfacts: []\n";
    let none = "source: {}\nfacts: []\n";
    let mut store = FactStore::open_in_memory().unwrap();
    for text in [other, none] {
        let result = store.apply_harvest(v(1), text);
        assert!(
            matches!(result, Err(KosError::FactHarvestFormat { .. })),
            "expected FactHarvestFormat for {text:?}, got {result:?}"
        );
    }
    assert_eq!(store.latest_version().unwrap(), None);
}

#[test]
fn a_harvest_of_a_few_facts_is_small_beside_the_whole_export() {
    let mut store = FactStore::open_in_memory().unwrap();
    let all: Vec<Fact> = (0..2000)
        .map(|n| f(&format!("doc{n}.go"), "REFERENCES", &format!("symbol-{n}")))
        .collect();
    store.assert_facts(v(1), &all).unwrap();
    let picks = &all[..20];
    let yaml = store.harvest(v(1), picks, &repo_source()).unwrap();
    let whole = export(&store);
    assert!(
        !yaml.is_empty() && yaml.len() * 20 < whole.len(),
        "harvest {} bytes against export {} bytes",
        yaml.len(),
        whole.len()
    );
}

#[test]
fn harvest_needs_repo_and_commit_in_the_source() {
    let mut store = FactStore::open_in_memory().unwrap();
    store.assert_facts(v(1), &[f("a", "r", "b")]).unwrap();
    for src in [
        BTreeMap::new(),
        source(&[("repo", "demo/repo")]),
        source(&[("commit", "abc1234")]),
    ] {
        let result = store.harvest(v(1), &[f("a", "r", "b")], &src);
        assert!(
            matches!(result, Err(KosError::FactHarvestFormat { .. })),
            "expected FactHarvestFormat for source {src:?}, got {result:?}"
        );
    }
}

#[test]
fn unknown_extra_keys_in_a_harvested_facts_file_are_ignored() {
    let text = "kind: harvested-facts\n\
                note: a later optional key\n\
                source:\n  repo: demo/repo\n  commit: abc1234\n  extra: also fine\n\
                facts:\n- e: a\n  a: r\n  v: b\n  hint: ignored too\n";
    let mut store = FactStore::open_in_memory().unwrap();
    assert_eq!(store.apply_harvest(v(3), text).unwrap(), 1);
    assert_eq!(store.facts_as_of(v(3)).unwrap(), vec![f("a", "r", "b")]);
}

/// The reader binds as the writer does: a harvest's source is its provenance,
/// and a file without repo and commit is refused by `parse` and by `apply`.
fn assert_source_refused(text: &str, what: &str) {
    let parsed = Harvest::parse(text);
    assert!(
        matches!(parsed, Err(KosError::FactHarvestFormat { .. })),
        "parse must refuse {what}, got {parsed:?}"
    );
    let mut store = FactStore::open_in_memory().unwrap();
    let applied = store.apply_harvest(v(1), text);
    assert!(
        matches!(applied, Err(KosError::FactHarvestFormat { .. })),
        "apply must refuse {what}, got {applied:?}"
    );
    assert_eq!(
        store.latest_version().unwrap(),
        None,
        "{what}: store stays empty"
    );
}

#[test]
fn a_harvest_file_with_no_source_is_refused() {
    assert_source_refused(
        "kind: harvested-facts\nfacts:\n- e: a\n  a: r\n  v: b\n",
        "a file with no source",
    );
}

#[test]
fn a_harvest_file_whose_source_has_no_repo_is_refused() {
    assert_source_refused(
        "kind: harvested-facts\nsource:\n  commit: abc1234\nfacts:\n- e: a\n  a: r\n  v: b\n",
        "a source with no repo",
    );
}

#[test]
fn a_harvest_file_whose_source_has_no_commit_is_refused() {
    assert_source_refused(
        "kind: harvested-facts\nsource:\n  repo: demo/repo\nfacts:\n- e: a\n  a: r\n  v: b\n",
        "a source with no commit",
    );
}
