//! R1 probe, MDE tests (aae-orc-6fy1). Probe code, local, never committed.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

use kos::facts::{Fact, FactStore, Harvest, Version};

fn die(msg: &str) -> ! {
    eprintln!("r1_mde: {msg}");
    std::process::exit(2)
}
fn arg(a: &[String], n: &str) -> Option<String> {
    a.iter().position(|x| x == n).and_then(|i| a.get(i + 1)).cloned()
}
fn need(a: &[String], n: &str) -> String {
    arg(a, n).unwrap_or_else(|| die(&format!("missing {n}")))
}
fn mine(repo: &str, f: &Fact) -> bool {
    if repo == "kos" {
        f.e == "kos" || f.e.starts_with("kos:") || f.e.starts_with("rust-analyzer ")
    } else {
        f.e == "marvel" || f.e.starts_with("marvel:") || f.e.starts_with("scip-go ")
    }
}
fn set(v: Vec<Fact>) -> BTreeSet<Fact> {
    v.into_iter().collect()
}

fn lockin(a: &[String]) {
    let src = FactStore::open(Path::new(&need(a, "--store"))).unwrap_or_else(|e| die(&format!("open: {e}")));
    let dir = need(a, "--dir");
    let latest = src.latest_version().unwrap().map(|v| v.get()).unwrap_or(0);
    let path = format!("{dir}/export.jsonl");
    let t = Instant::now();
    let mut f = fs::File::create(&path).unwrap();
    src.export_jsonl(&mut f).unwrap_or_else(|e| die(&format!("export: {e}")));
    drop(f);
    let t_exp = t.elapsed().as_secs_f64();
    let lines = fs::read_to_string(&path).unwrap().lines().count();
    let fresh_path = format!("{dir}/fresh.db");
    let _ = fs::remove_file(&fresh_path);
    let mut fresh = FactStore::open(Path::new(&fresh_path)).unwrap_or_else(|e| die(&format!("open fresh: {e}")));
    let t = Instant::now();
    let n = fresh.import_jsonl(BufReader::new(fs::File::open(&path).unwrap())).unwrap_or_else(|e| die(&format!("import: {e}")));
    let t_imp = t.elapsed().as_secs_f64();
    println!("lockin export_seconds={t_exp:.2} import_seconds={t_imp:.2} export_lines={lines} imported_rows={n} latest_source={latest} latest_fresh={:?}", fresh.latest_version().unwrap().map(|v| v.get()));
    let (mut bad_state, mut bad_diff, mut bad_count) = (0, 0, 0);
    let mut prev_count = 0usize;
    for v in 1..=latest {
        let a1 = set(src.facts_as_of(Version::new(v)).unwrap());
        let b1 = set(fresh.facts_as_of(Version::new(v)).unwrap());
        let da = src.diff(Version::new(v - 1), Version::new(v)).unwrap();
        let db = fresh.diff(Version::new(v - 1), Version::new(v)).unwrap();
        let state_eq = a1 == b1;
        let diff_eq = da.added == db.added && da.removed == db.removed;
        if !state_eq {
            bad_state += 1;
        }
        if !diff_eq {
            bad_diff += 1;
        }
        if a1.len() != b1.len() {
            bad_count += 1;
        }
        println!("version={v} facts_src={} facts_fresh={} added={} removed={} state_equal={state_eq} diff_equal={diff_eq}", a1.len(), b1.len(), da.added.len(), da.removed.len());
        prev_count = a1.len();
    }
    let _ = prev_count;
    println!("lockin versions={latest} state_mismatches={bad_state} diff_mismatches={bad_diff} count_mismatches={bad_count}");
    if bad_state + bad_diff + bad_count > 0 {
        std::process::exit(1);
    }
}

fn dump(a: &[String]) {
    let st = FactStore::open(Path::new(&need(a, "--store"))).unwrap_or_else(|e| die(&format!("open: {e}")));
    let v: i64 = need(a, "--version").parse().unwrap();
    let repo = need(a, "--repo");
    let mut facts: Vec<Fact> = st.facts_as_of(Version::new(v)).unwrap().into_iter().filter(|f| f.a.starts_with("code/") && mine(&repo, f)).collect();
    facts.sort();
    for f in facts {
        println!("{}\t{}\t{}", f.e, f.a, f.v);
    }
}

fn live(st: &FactStore, f: &Fact, n: i64) -> bool {
    st.values_of(&f.e, &f.a, Version::new(n)).map(|vs| vs.contains(&f.v)).unwrap_or(false)
}

fn roundtrip(a: &[String]) {
    let st = FactStore::open(Path::new(&need(a, "--store"))).unwrap_or_else(|e| die(&format!("open: {e}")));
    let repo = need(a, "--repo");
    let picks_path = need(a, "--picks");
    let dir = need(a, "--dir");
    let latest = st.latest_version().unwrap().map(|v| v.get()).unwrap_or(0);
    let picks: Vec<Fact> = fs::read_to_string(&picks_path)
        .unwrap()
        .lines()
        .filter_map(|l| {
            let mut p = l.splitn(3, '\t');
            Some(Fact::new(p.next()?, p.next()?, p.next()?))
        })
        .collect();
    // start and end of each pick's interval in the source
    let mut info: BTreeMap<Fact, (i64, Option<i64>)> = BTreeMap::new();
    let mut prior_intervals: BTreeMap<Fact, usize> = BTreeMap::new();
    for f in &picks {
        // the interval that holds at the head: walk back from the latest version while the fact is live
        let mut start = 0i64;
        let mut n = latest;
        while n >= 1 && live(&st, f, n) {
            start = n;
            n -= 1;
        }
        let end: Option<i64> = None;
        // earlier closed intervals of the same fact (live before, absent between): reported, not carried by a head harvest
        let mut earlier = 0usize;
        let mut was_live = false;
        for m in 1..start.max(1) {
            let l = live(&st, f, m);
            if l && !was_live {
                earlier += 1;
            }
            was_live = l;
        }
        if earlier > 0 {
            prior_intervals.insert(f.clone(), earlier);
        }
        info.insert(f.clone(), (start, end));
    }
    let prov = |n: i64| -> BTreeMap<String, String> {
        let ent = format!("load/{repo}@{n}");
        ["repo", "commit", "indexer", "indexer-version", "index-sha256", "grade"]
            .iter()
            .map(|k| (k.to_string(), st.values_of(&ent, &format!("prov/{k}"), Version::new(n)).unwrap_or_default().first().cloned().unwrap_or_default()))
            .collect()
    };
    let mut by_start: BTreeMap<i64, Vec<Fact>> = BTreeMap::new();
    for (f, (s, _)) in &info {
        by_start.entry(*s).or_default().push(f.clone());
    }
    let mut fresh = FactStore::open_in_memory().unwrap();
    let (mut files, mut yaml_facts_equal, mut src_missing) = (0, 0, 0);
    let mut yaml_keys: BTreeSet<String> = BTreeSet::new();
    let mut yaml_lost_prov: BTreeSet<String> = BTreeSet::new();
    for (s, group) in &by_start {
        if *s == 0 {
            src_missing += group.len();
            continue;
        }
        let p = prov(*s);
        let mut source = BTreeMap::new();
        source.insert("repo".to_string(), p["repo"].clone());
        source.insert("commit".to_string(), p["commit"].clone());
        source.insert("version".to_string(), s.to_string());
        let yaml = st.harvest(Version::new(*s), group, &source).unwrap_or_else(|e| die(&format!("harvest: {e}")));
        fs::write(format!("{dir}/harvest-{repo}-v{s}.yaml"), &yaml).unwrap();
        files += 1;
        let back = Harvest::parse(&yaml).unwrap_or_else(|e| die(&format!("parse: {e}")));
        let mut want = group.clone();
        want.sort();
        let mut got = back.facts.clone();
        got.sort();
        if want == got {
            yaml_facts_equal += want.len();
        }
        for k in back.source.keys() {
            yaml_keys.insert(k.clone());
        }
        for k in p.keys() {
            if !back.source.contains_key(k) {
                yaml_lost_prov.insert(k.clone());
            }
        }
        // apply in ascending start version, as one harvest per start version
        fresh.apply_harvest(Version::new(*s), &yaml).unwrap_or_else(|e| die(&format!("apply: {e}")));
    }
    // intervals as rebuilt (ends: the rebuilt store never retracts, so an end the source has cannot come back)
    let (mut start_eq, mut end_eq, mut n_ended_src) = (0, 0, 0);
    for (f, (s, e)) in &info {
        if *s == 0 {
            continue;
        }
        let rs = (1..=latest).find(|&n| live(&fresh, f, n)).unwrap_or(0);
        let re = ((rs + 1)..=latest).find(|&n| !live(&fresh, f, n));
        if rs == *s {
            start_eq += 1;
        }
        if re == *e {
            end_eq += 1;
        }
        if e.is_some() {
            n_ended_src += 1;
        }
    }
    println!("roundtrip repo={repo} picks={} source_missing={src_missing} yaml_files={files} facts_equal_after_yaml={yaml_facts_equal} start_equal_after_apply={start_eq} end_equal_after_apply={end_eq} picks_with_end_in_source={n_ended_src} picks_with_an_earlier_closed_interval={} earlier_closed_intervals_total={}", picks.len(), prior_intervals.len(), prior_intervals.values().sum::<usize>());
    println!("roundtrip yaml_source_keys={:?} provenance_values_not_in_yaml={:?}", yaml_keys, yaml_lost_prov);
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    match a.first().map(String::as_str) {
        Some("lockin") => lockin(&a),
        Some("dump") => dump(&a),
        Some("roundtrip") => roundtrip(&a),
        _ => die("usage: lockin|dump|roundtrip"),
    }
}
