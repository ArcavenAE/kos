//! `FactStore::apply`: retractions and assertions at one version, in one
//! SQLite transaction (aae-orc-yto7v; the substrate is aae-orc-xxe3n's valid-time
//! store). A load that ends facts and asserts new ones must not leave half of
//! itself behind when a write fails.

use kos::facts::{Applied, Fact, FactStore, Version};
use tempfile::TempDir;

fn v(n: i64) -> Version {
    Version::new(n)
}

fn f(e: &str, a: &str, val: &str) -> Fact {
    Fact::new(e, a, val)
}

fn export(store: &FactStore) -> String {
    let mut out = Vec::new();
    store.export_jsonl(&mut out).unwrap();
    String::from_utf8(out).unwrap()
}

/// A store file with three facts at version 1.
fn seeded(dir: &TempDir) -> std::path::PathBuf {
    let db = dir.path().join("facts.db");
    let mut store = FactStore::open(&db).unwrap();
    store
        .assert_facts(
            v(1),
            &[f("a", "r", "1"), f("b", "r", "1"), f("c", "r", "1")],
        )
        .unwrap();
    db
}

#[test]
fn apply_retracts_and_asserts_at_one_version() {
    let dir = TempDir::new().unwrap();
    let mut store = FactStore::open(&seeded(&dir)).unwrap();
    let done = store
        .apply(
            v(2),
            &[f("a", "r", "1")],
            &[f("d", "r", "1"), f("b", "r", "1")],
        )
        .unwrap();
    assert_eq!(
        done,
        Applied {
            retracted: 1,
            asserted: 1
        },
        "b is already live, so only d is newly asserted"
    );
    assert_eq!(
        store.facts_as_of(v(2)).unwrap(),
        vec![f("b", "r", "1"), f("c", "r", "1"), f("d", "r", "1")]
    );
    assert_eq!(
        store.facts_as_of(v(1)).unwrap().len(),
        3,
        "version 1 is intact"
    );
}

#[test]
fn apply_behind_the_latest_version_is_refused_and_changes_nothing() {
    let dir = TempDir::new().unwrap();
    let mut store = FactStore::open(&seeded(&dir)).unwrap();
    store.assert_facts(v(3), &[f("z", "r", "1")]).unwrap();
    let before = export(&store);
    let result = store.apply(v(2), &[f("a", "r", "1")], &[f("d", "r", "1")]);
    assert!(result.is_err(), "a version behind the latest is refused");
    assert_eq!(export(&store), before);
}

#[test]
fn a_failed_apply_leaves_the_store_unchanged() {
    let dir = TempDir::new().unwrap();
    let db = seeded(&dir);
    // A trigger makes the insert of one value fail after the retraction and an
    // earlier assertion have already been written inside the same apply.
    let conn = rusqlite::Connection::open(&db).unwrap();
    conn.execute_batch(
        "CREATE TRIGGER fail_on_boom BEFORE INSERT ON facts WHEN NEW.v = 'boom'
         BEGIN SELECT RAISE(ABORT, 'forced failure'); END;",
    )
    .unwrap();
    drop(conn);

    let mut store = FactStore::open(&db).unwrap();
    let before = export(&store);
    let result = store.apply(
        v(2),
        &[f("a", "r", "1")],
        &[f("d", "r", "1"), f("e", "r", "boom")],
    );
    assert!(result.is_err(), "the forced failure surfaces");
    assert_eq!(
        export(&store),
        before,
        "the retraction and the first assertion were rolled back with the failure"
    );
    assert_eq!(
        store.latest_version().unwrap(),
        Some(v(1)),
        "the latest version did not move"
    );
}
