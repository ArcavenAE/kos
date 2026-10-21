//! Criterion 4 traversal, batched per hop, for corporate research's blind
//! Dolt design (one query per hop, IN lists for hops 2 and 3).
//!
//! hop 1: SELECT e FROM facts WHERE a='REFERENCES' AND v=?
//! hop 2: SELECT v FROM facts WHERE a='DEFINES' AND e IN (...)
//! hop 3: SELECT e FROM facts WHERE a='REFERENCES' AND v IN (...)
//! Set logic in Rust. Usage: trav2 <db-url> <seeds.txt> <results.jsonl>

use std::collections::BTreeSet;
use std::io::Write;
use std::time::Instant;

use sqlx::any::{install_default_drivers, AnyPoolOptions};
use sqlx::{AnyPool, Row};

async fn q_in(pool: &AnyPool, head: &str, keys: &BTreeSet<String>) -> Vec<String> {
    if keys.is_empty() {
        return Vec::new();
    }
    let marks = vec!["?"; keys.len()].join(",");
    let sql: &'static str = Box::leak(format!("{head} ({marks})").into_boxed_str());
    let mut q = sqlx::query(sql);
    for k in keys {
        q = q.bind(k.clone());
    }
    q.fetch_all(pool).await.expect("query").iter().map(|r| r.get::<String, _>(0)).collect()
}

fn esc(s: &str) -> String {
    let mut o = String::from("\"");
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
    install_default_drivers();
    let pool = AnyPoolOptions::new().max_connections(1).connect(&args[1]).await.expect("connect");
    let seeds: Vec<String> = std::fs::read_to_string(&args[2]).expect("seeds").lines().filter(|l| !l.is_empty()).map(String::from).collect();
    let mut fh = std::fs::File::create(&args[3]).expect("out");
    let (mut worst, t_all) = (0f64, Instant::now());
    for s in &seeds {
        let t0 = Instant::now();
        let one: BTreeSet<String> = [s.clone()].into_iter().collect();
        let hop1: BTreeSet<String> = q_in(&pool, "SELECT e FROM facts WHERE a = 'REFERENCES' AND v IN", &one).await.into_iter().collect();
        let hop2: BTreeSet<String> = q_in(&pool, "SELECT v FROM facts WHERE a = 'DEFINES' AND e IN", &hop1).await.into_iter().collect();
        let hop3: BTreeSet<String> = q_in(&pool, "SELECT e FROM facts WHERE a = 'REFERENCES' AND v IN", &hop2).await.into_iter().collect();
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        worst = worst.max(ms);
        let res: Vec<String> = hop3.iter().map(|x| esc(x)).collect();
        writeln!(fh, "{{\"s\":{},\"hop1\":{},\"hop2\":{},\"hop3\":{},\"ms\":{:.3},\"result\":[{}]}}", esc(s), hop1.len(), hop2.len(), hop3.len(), ms, res.join(",")).expect("write");
    }
    println!("seeds={} slowest_ms={:.3} total_ms={:.1}", seeds.len(), worst, t_all.elapsed().as_secs_f64() * 1000.0);
}
