//! Criterion 4 traversal, shared by Probes A and B.
//!
//! From each seed symbol S:
//!   hop 1: documents that REFERENCE S
//!   hop 2: symbols DEFINED in those documents
//!   hop 3: documents that REFERENCE those symbols
//! The set logic runs here in Rust. The store answers only indexed point
//! lookups, one per frontier element, so no hop pulls the whole log.
//!
//! Usage: trav <db-url> <seeds.txt> <results.jsonl> [tx | vt:<n> | asof:<ref>]
//!   db-url: sqlite://<file>?mode=ro or mysql://root@127.0.0.1:<port>/<db>
//!   tx: when given, every lookup is filtered to that tx (Probe B snapshots).
//!   vt:<n>: valid-time, facts live at commit n (valid_from <= n < valid_to).
//!   asof:<ref>: Dolt, every lookup reads `facts AS OF '<ref>'`.

use std::collections::BTreeSet;
use std::io::Write;
use std::time::Instant;

use sqlx::any::{install_default_drivers, AnyPoolOptions};
use sqlx::{AnyPool, Row};

async fn lookup(pool: &AnyPool, sql: &'static str, key: &str, tx: Option<i64>, nb: usize) -> Vec<String> {
    let mut q = sqlx::query(sql).bind(key.to_string());
    if let Some(t) = tx {
        for _ in 0..nb {
            q = q.bind(t);
        }
    }
    q.fetch_all(pool)
        .await
        .expect("lookup")
        .iter()
        .map(|r| r.get::<String, _>(0))
        .collect()
}

fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (url, seeds, out) = (&args[1], &args[2], &args[3]);
    let mode = args.get(4).cloned().unwrap_or_default();
    let (tx, nb, table, txf): (Option<i64>, usize, String, &str) = if mode.is_empty() {
        (None, 0, "facts".into(), "")
    } else if let Some(n) = mode.strip_prefix("vt:") {
        (Some(n.parse().expect("vt")), 2, "facts".into(), " AND valid_from <= ? AND (valid_to IS NULL OR valid_to > ?)")
    } else if let Some(r) = mode.strip_prefix("asof:") {
        assert!(r.chars().all(|c| c.is_ascii_alphanumeric()), "asof ref");
        (None, 0, format!("facts AS OF '{r}'"), "")
    } else {
        (Some(mode.parse().expect("tx")), 1, "facts".into(), " AND tx = ?")
    };
    install_default_drivers();
    let pool = AnyPoolOptions::new().max_connections(1).connect(url).await.expect("connect");
    let q_refs: &'static str = Box::leak(format!("SELECT e FROM {table} WHERE a = 'REFERENCES' AND v = ?{txf}").into_boxed_str());
    let q_defs: &'static str = Box::leak(format!("SELECT v FROM {table} WHERE e = ? AND a = 'DEFINES'{txf}").into_boxed_str());

    let seeds: Vec<String> = std::fs::read_to_string(seeds)
        .expect("seeds")
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();
    let mut fh = std::fs::File::create(out).expect("out");
    let mut worst = 0f64;
    let mut lookups_total = 0usize;
    let t_all = Instant::now();
    for s in &seeds {
        let t0 = Instant::now();
        let mut n_lookups = 1usize;
        let hop1: BTreeSet<String> = lookup(&pool, q_refs, s, tx, nb).await.into_iter().collect();
        let mut hop2: BTreeSet<String> = BTreeSet::new();
        for d in &hop1 {
            n_lookups += 1;
            hop2.extend(lookup(&pool, q_defs, d, tx, nb).await);
        }
        let mut hop3: BTreeSet<String> = BTreeSet::new();
        for y in &hop2 {
            n_lookups += 1;
            hop3.extend(lookup(&pool, q_refs, y, tx, nb).await);
        }
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        worst = worst.max(ms);
        lookups_total += n_lookups;
        let j = |set: &BTreeSet<String>| set.iter().map(|x| esc(x)).collect::<Vec<_>>().join(",");
        writeln!(
            fh,
            "{{\"s\":{},\"hop1\":{},\"hop2\":{},\"hop3\":{},\"lookups\":{},\"ms\":{:.3},\"hop1_set\":[{}],\"hop2_set\":[{}],\"result\":[{}]}}",
            esc(s),
            hop1.len(),
            hop2.len(),
            hop3.len(),
            n_lookups,
            ms,
            j(&hop1),
            j(&hop2),
            j(&hop3)
        )
        .expect("write");
    }
    println!(
        "seeds={} slowest_ms={:.3} total_ms={:.1} lookups={}",
        seeds.len(),
        worst,
        t_all.elapsed().as_secs_f64() * 1000.0,
        lookups_total
    );
}
