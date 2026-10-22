//! Probe C: run labelled queries that return one row of two integers, timing each in process.
//! Usage: qtime <db-url> <queries.tsv> <out.jsonl>   (each line: label TAB sql)
use std::io::Write;
use std::time::Instant;
use sqlx::any::{install_default_drivers, AnyPoolOptions};
use sqlx::Row;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let a: Vec<String> = std::env::args().collect();
    install_default_drivers();
    let pool = AnyPoolOptions::new().max_connections(1).connect(&a[1]).await.expect("connect");
    let mut fh = std::fs::File::create(&a[3]).expect("out");
    let text: &'static str = Box::leak(std::fs::read_to_string(&a[2]).expect("queries").into_boxed_str());
    for line in text.lines().filter(|l| !l.is_empty()) {
        let (label, sql) = line.split_once('\t').expect("label TAB sql");
        let t0 = Instant::now();
        let row = sqlx::query(sql).fetch_one(&pool).await.expect("query");
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        let c1: i64 = row.get(0);
        let c2: i64 = row.get(1);
        writeln!(fh, "{{\"label\":\"{label}\",\"ms\":{ms:.3},\"c1\":{c1},\"c2\":{c2}}}").expect("write");
    }
}
