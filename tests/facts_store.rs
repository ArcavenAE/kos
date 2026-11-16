//! Behavior of the SQLite valid-time fact store (aae-orc-xxe3n, PR 1).

use std::collections::BTreeSet;

use kos::error::KosError;
use kos::facts::{Fact, FactDiff, FactStore, Version};

fn v(n: i64) -> Version {
    Version::new(n)
}

fn f(e: &str, a: &str, val: &str) -> Fact {
    Fact::new(e, a, val)
}

fn sorted(mut facts: Vec<Fact>) -> Vec<Fact> {
    facts.sort();
    facts
}

#[test]
fn empty_store_holds_nothing_at_any_version() {
    let mut store = FactStore::open_in_memory().unwrap();
    assert_eq!(store.facts_as_of(v(0)).unwrap(), Vec::<Fact>::new());
    assert_eq!(store.facts_as_of(v(99)).unwrap(), Vec::<Fact>::new());
    assert_eq!(store.latest_version().unwrap(), None);
    // A write moves the latest version; an empty batch does not.
    store.assert_facts(v(7), &[f("a", "r", "b")]).unwrap();
    store.assert_facts(v(8), &[]).unwrap();
    assert_eq!(store.latest_version().unwrap(), Some(v(7)));
}

#[test]
fn assert_is_visible_from_its_version_and_not_before() {
    let mut store = FactStore::open_in_memory().unwrap();
    let added = store
        .assert_facts(v(3), &[f("doc", "DEFINES", "sym")])
        .unwrap();
    assert_eq!(added, 1);
    assert_eq!(store.facts_as_of(v(2)).unwrap(), Vec::<Fact>::new());
    assert_eq!(
        store.facts_as_of(v(3)).unwrap(),
        vec![f("doc", "DEFINES", "sym")]
    );
    assert_eq!(
        store.facts_as_of(v(40)).unwrap(),
        vec![f("doc", "DEFINES", "sym")]
    );
    assert_eq!(store.latest_version().unwrap(), Some(v(3)));
}

#[test]
fn asserting_a_live_fact_again_changes_nothing() {
    let mut store = FactStore::open_in_memory().unwrap();
    assert_eq!(store.assert_facts(v(1), &[f("a", "r", "b")]).unwrap(), 1);
    assert_eq!(store.assert_facts(v(2), &[f("a", "r", "b")]).unwrap(), 0);
    // Still one fact, and its interval did not restart at version 2.
    assert_eq!(store.facts_as_of(v(1)).unwrap(), vec![f("a", "r", "b")]);
    assert_eq!(store.diff(v(1), v(2)).unwrap(), FactDiff::default());
}

#[test]
fn retract_closes_the_interval_at_its_version() {
    let mut store = FactStore::open_in_memory().unwrap();
    store.assert_facts(v(1), &[f("a", "r", "b")]).unwrap();
    assert_eq!(store.retract_facts(v(4), &[f("a", "r", "b")]).unwrap(), 1);
    assert_eq!(store.facts_as_of(v(3)).unwrap(), vec![f("a", "r", "b")]);
    assert_eq!(store.facts_as_of(v(4)).unwrap(), Vec::<Fact>::new());
}

#[test]
fn retracting_a_fact_that_is_not_live_returns_zero() {
    let mut store = FactStore::open_in_memory().unwrap();
    assert_eq!(store.retract_facts(v(1), &[f("a", "r", "b")]).unwrap(), 0);
    store.assert_facts(v(2), &[f("a", "r", "b")]).unwrap();
    assert_eq!(store.retract_facts(v(3), &[f("a", "r", "b")]).unwrap(), 1);
    assert_eq!(store.retract_facts(v(4), &[f("a", "r", "b")]).unwrap(), 0);
}

#[test]
fn reassert_after_retract_gives_two_intervals() {
    let mut store = FactStore::open_in_memory().unwrap();
    let fact = f("a", "r", "b");
    store
        .assert_facts(v(1), std::slice::from_ref(&fact))
        .unwrap();
    store
        .retract_facts(v(3), std::slice::from_ref(&fact))
        .unwrap();
    assert_eq!(
        store
            .assert_facts(v(5), std::slice::from_ref(&fact))
            .unwrap(),
        1
    );
    let held: Vec<bool> = (0..7)
        .map(|n| !store.facts_as_of(v(n)).unwrap().is_empty())
        .collect();
    assert_eq!(held, vec![false, true, true, false, false, true, true]);
}

#[test]
fn assert_and_retract_at_one_version_leave_no_trace() {
    let mut store = FactStore::open_in_memory().unwrap();
    store.assert_facts(v(2), &[f("a", "r", "b")]).unwrap();
    assert_eq!(store.retract_facts(v(2), &[f("a", "r", "b")]).unwrap(), 1);
    assert_eq!(store.facts_as_of(v(2)).unwrap(), Vec::<Fact>::new());
    assert_eq!(store.facts_as_of(v(9)).unwrap(), Vec::<Fact>::new());
    assert_eq!(store.diff(v(1), v(9)).unwrap(), FactDiff::default());
}

#[test]
fn a_write_behind_the_latest_version_is_refused() {
    let mut store = FactStore::open_in_memory().unwrap();
    store.assert_facts(v(5), &[f("a", "r", "b")]).unwrap();
    let late_assert = store.assert_facts(v(4), &[f("c", "r", "d")]);
    assert!(
        matches!(
            late_assert,
            Err(KosError::FactVersionOrder { got: 4, latest: 5 })
        ),
        "version 4 is behind 5, got {late_assert:?}"
    );
    let late_retract = store.retract_facts(v(3), &[f("a", "r", "b")]);
    assert!(
        matches!(
            late_retract,
            Err(KosError::FactVersionOrder { got: 3, latest: 5 })
        ),
        "version 3 is behind 5, got {late_retract:?}"
    );
    // The refused writes changed nothing.
    assert_eq!(store.facts_as_of(v(9)).unwrap(), vec![f("a", "r", "b")]);
}

#[test]
fn diff_lists_added_and_removed_sorted() {
    let mut store = FactStore::open_in_memory().unwrap();
    store
        .assert_facts(
            v(1),
            &[f("a", "r", "1"), f("a", "r", "2"), f("b", "r", "1")],
        )
        .unwrap();
    store.retract_facts(v(2), &[f("a", "r", "2")]).unwrap();
    store
        .assert_facts(v(2), &[f("c", "r", "1"), f("a", "r", "3")])
        .unwrap();
    let d = store.diff(v(1), v(2)).unwrap();
    assert_eq!(d.added, vec![f("a", "r", "3"), f("c", "r", "1")]);
    assert_eq!(d.removed, vec![f("a", "r", "2")]);
    // Reversed, the two sides swap.
    let back = store.diff(v(2), v(1)).unwrap();
    assert_eq!(back.added, d.removed);
    assert_eq!(back.removed, d.added);
}

#[test]
fn lookups_follow_the_as_of_set() {
    let mut store = FactStore::open_in_memory().unwrap();
    store
        .assert_facts(
            v(1),
            &[
                f("d1", "DEFINES", "s1"),
                f("d1", "REFERENCES", "s2"),
                f("d2", "REFERENCES", "s2"),
            ],
        )
        .unwrap();
    store
        .retract_facts(v(3), &[f("d2", "REFERENCES", "s2")])
        .unwrap();
    assert_eq!(
        store.values_of("d1", "REFERENCES", v(2)).unwrap(),
        vec!["s2".to_string()]
    );
    assert_eq!(
        store.entities_with("REFERENCES", "s2", v(2)).unwrap(),
        vec!["d1".to_string(), "d2".to_string()]
    );
    assert_eq!(
        store.entities_with("REFERENCES", "s2", v(3)).unwrap(),
        vec!["d1".to_string()]
    );
    assert_eq!(
        store.values_of("d2", "REFERENCES", v(3)).unwrap(),
        Vec::<String>::new()
    );
}

/// A deterministic log over 30 versions, checked against a plain set model:
/// every version's fact set and every pair's diff must equal the model's.
#[test]
fn store_agrees_with_a_plain_set_model_over_a_long_log() {
    let mut store = FactStore::open_in_memory().unwrap();
    let mut model: Vec<BTreeSet<Fact>> = vec![BTreeSet::new()];
    let mut live: BTreeSet<Fact> = BTreeSet::new();
    let mut seed: u64 = 20_261_115;
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
        model.push(live.clone());
    }
    for (n, set) in model.iter().enumerate() {
        let want: Vec<Fact> = set.iter().cloned().collect();
        let got = store.facts_as_of(v(i64::try_from(n).unwrap())).unwrap();
        assert_eq!(got, want, "fact set at version {n}");
    }
    for (from, before) in model.iter().enumerate() {
        for (to, after) in model.iter().enumerate() {
            let want_added: Vec<Fact> = after.difference(before).cloned().collect();
            let want_removed: Vec<Fact> = before.difference(after).cloned().collect();
            let got = store
                .diff(
                    v(i64::try_from(from).unwrap()),
                    v(i64::try_from(to).unwrap()),
                )
                .unwrap();
            assert_eq!(sorted(got.added), want_added, "added {from} to {to}");
            assert_eq!(sorted(got.removed), want_removed, "removed {from} to {to}");
        }
    }
}

#[test]
fn a_file_store_keeps_its_facts_across_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("facts.db");
    {
        let mut store = FactStore::open(&path).unwrap();
        store
            .assert_facts(v(1), &[f("a", "r", "b"), f("c", "r", "d")])
            .unwrap();
        store.retract_facts(v(2), &[f("c", "r", "d")]).unwrap();
    }
    let store = FactStore::open(&path).unwrap();
    assert_eq!(store.facts_as_of(v(1)).unwrap().len(), 2);
    assert_eq!(store.facts_as_of(v(2)).unwrap(), vec![f("a", "r", "b")]);
    assert_eq!(store.latest_version().unwrap(), Some(v(2)));
}

#[test]
fn the_four_covering_indexes_exist() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("facts.db");
    {
        let _store = FactStore::open(&path).unwrap();
    }
    let conn = rusqlite::Connection::open(&path).unwrap();
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'index' AND tbl_name = 'facts' ORDER BY name")
        .unwrap();
    let names: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    for want in ["facts_aevt", "facts_avet", "facts_eavt", "facts_vaet"] {
        assert!(
            names.iter().any(|n| n == want),
            "index {want} missing; found {names:?}"
        );
    }
}

#[test]
fn a_store_from_a_newer_schema_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("facts.db");
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.pragma_update(None, "user_version", 999).unwrap();
    }
    let result = FactStore::open(&path);
    assert!(
        matches!(result, Err(KosError::FactSchemaTooNew { found: 999, .. })),
        "expected FactSchemaTooNew for user_version 999"
    );
}
