//! Multi-hop traversal over the fact store (aae-orc-xxe3n, PR 3).

use std::collections::BTreeSet;

use kos::facts::{Fact, FactStore, Step, Version};

fn v(n: i64) -> Version {
    Version::new(n)
}

fn f(e: &str, a: &str, val: &str) -> Fact {
    Fact::new(e, a, val)
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

fn seeds(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

/// The probe's recipe: documents that reference a symbol, the symbols those
/// documents define, the documents that reference those.
fn probe_steps() -> Vec<Step> {
    vec![
        Step::backward("REFERENCES"),
        Step::forward("DEFINES"),
        Step::backward("REFERENCES"),
    ]
}

fn code_graph() -> FactStore {
    let mut store = FactStore::open_in_memory().unwrap();
    store
        .assert_facts(
            v(1),
            &[
                f("d1", "REFERENCES", "s0"),
                f("d2", "REFERENCES", "s0"),
                f("d1", "DEFINES", "s1"),
                f("d1", "DEFINES", "s2"),
                f("d2", "DEFINES", "s3"),
                f("d3", "REFERENCES", "s1"),
                f("d4", "REFERENCES", "s3"),
                f("d5", "REFERENCES", "unrelated"),
            ],
        )
        .unwrap();
    store
}

#[test]
fn the_probe_recipe_returns_one_set_per_hop() {
    let store = code_graph();
    let hops = store
        .traverse(v(1), &seeds(&["s0"]), &probe_steps())
        .unwrap();
    assert_eq!(
        hops,
        vec![
            set(&["d1", "d2"]),
            set(&["s1", "s2", "s3"]),
            set(&["d3", "d4"]),
        ]
    );
}

#[test]
fn a_traversal_reads_the_facts_as_of_its_version() {
    let mut store = code_graph();
    store
        .retract_facts(v(5), &[f("d1", "REFERENCES", "s0")])
        .unwrap();
    let before = store
        .traverse(v(4), &seeds(&["s0"]), &probe_steps())
        .unwrap();
    let after = store
        .traverse(v(5), &seeds(&["s0"]), &probe_steps())
        .unwrap();
    assert_eq!(before.len(), 3, "three steps give three hops at version 4");
    assert_eq!(after.len(), 3, "three steps give three hops at version 5");
    assert_eq!(before[0], set(&["d1", "d2"]), "hop 1 at version 4");
    assert_eq!(after[0], set(&["d2"]), "hop 1 at version 5");
    assert_eq!(after[1], set(&["s3"]), "hop 2 at version 5");
    assert_eq!(after[2], set(&["d4"]), "hop 3 at version 5");
    let early = store
        .traverse(v(0), &seeds(&["s0"]), &probe_steps())
        .unwrap();
    assert_eq!(
        early,
        vec![BTreeSet::new(), BTreeSet::new(), BTreeSet::new()],
        "nothing holds at version 0"
    );
}

#[test]
fn no_steps_gives_no_hops_and_no_seeds_gives_empty_hops() {
    let store = code_graph();
    assert_eq!(
        store.traverse(v(1), &seeds(&["s0"]), &[]).unwrap(),
        Vec::<BTreeSet<String>>::new()
    );
    // Two steps with no seeds: two empty sets, so the caller can index by hop.
    let hops = store
        .traverse(
            v(1),
            &[],
            &[Step::backward("REFERENCES"), Step::forward("DEFINES")],
        )
        .unwrap();
    assert_eq!(hops, vec![BTreeSet::new(), BTreeSet::new()]);
}

#[test]
fn an_attribute_nobody_uses_gives_an_empty_hop_and_not_an_error() {
    let store = code_graph();
    let hops = store
        .traverse(
            v(1),
            &seeds(&["s0"]),
            &[
                Step::backward("NO_SUCH_ATTRIBUTE"),
                Step::forward("DEFINES"),
            ],
        )
        .unwrap();
    assert_eq!(hops, vec![BTreeSet::new(), BTreeSet::new()]);
}

#[test]
fn repeated_seeds_count_once_and_forward_then_backward_returns_to_the_entity() {
    let store = code_graph();
    let hops = store
        .traverse(
            v(1),
            &seeds(&["d1", "d1", "d2"]),
            &[Step::forward("DEFINES"), Step::backward("DEFINES")],
        )
        .unwrap();
    assert_eq!(hops, vec![set(&["s1", "s2", "s3"]), set(&["d1", "d2"])]);
}

/// A 30-version log checked against a plain computation over `facts_as_of`:
/// for every sampled version and every seed, the three hop sets equal the
/// join done in memory.
#[test]
fn traversal_equals_a_plain_join_over_a_long_log() {
    let mut store = FactStore::open_in_memory().unwrap();
    let mut live: BTreeSet<Fact> = BTreeSet::new();
    let mut state: u64 = 20_261_117;
    let mut next = move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        state >> 33
    };
    for version in 1..=30_i64 {
        let mut add = Vec::new();
        let mut del = Vec::new();
        for _ in 0..14 {
            let fact = f(
                &format!("e{}", next() % 8),
                if next() % 2 == 0 {
                    "DEFINES"
                } else {
                    "REFERENCES"
                },
                &format!("e{}", next() % 8),
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
    let mut checked = 0;
    for version in [1_i64, 7, 15, 22, 30] {
        let facts = store.facts_as_of(v(version)).unwrap();
        for seed in 0..8 {
            let s = format!("e{seed}");
            let hop1: BTreeSet<String> = facts
                .iter()
                .filter(|x| x.a == "REFERENCES" && x.v == s)
                .map(|x| x.e.clone())
                .collect();
            let hop2: BTreeSet<String> = facts
                .iter()
                .filter(|x| x.a == "DEFINES" && hop1.contains(&x.e))
                .map(|x| x.v.clone())
                .collect();
            let hop3: BTreeSet<String> = facts
                .iter()
                .filter(|x| x.a == "REFERENCES" && hop2.contains(&x.v))
                .map(|x| x.e.clone())
                .collect();
            let got = store
                .traverse(v(version), std::slice::from_ref(&s), &probe_steps())
                .unwrap();
            assert_eq!(
                got,
                vec![hop1, hop2, hop3],
                "hop sets from seed {s} at version {version}"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 40);
}
