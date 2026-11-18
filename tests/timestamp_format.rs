//! The two timestamps kos writes, pinned (replacing the chrono dependency, which
//! RUSTSEC-2026-0543 flags as unmaintained). Each is RFC 3339 in UTC with a
//! numeric `+00:00` offset and, only when the clock has them, 3, 6 or 9 fractional
//! digits: the shape `chrono::Utc::now().to_rfc3339()` produced. A swap of the
//! time crate must not change it, since `orient.jsonl` is read by other tools and
//! the `built:` line of `kos version` is read by people and scripts.
//!
//! Sites: `build.rs` (the `built:` line of `kos version`) and `src/orient.rs`
//! (the `timestamp` field of the opt-in `orient --log` line). Neither passes a
//! time type across a serde boundary; the JSON field is a plain string, and the
//! second test pins the whole key set of that line.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// True when `s` is `YYYY-MM-DDTHH:MM:SS`, then nothing or `.` and exactly 3, 6
/// or 9 digits, then `+00:00`, and nothing after.
fn is_rfc3339_utc_numeric_offset(s: &str) -> bool {
    let b = s.as_bytes();
    let digits = |r: std::ops::Range<usize>| {
        r.into_iter()
            .all(|i| b.get(i).is_some_and(u8::is_ascii_digit))
    };
    if b.len() < 25
        || !digits(0..4)
        || b[4] != b'-'
        || !digits(5..7)
        || b[7] != b'-'
        || !digits(8..10)
    {
        return false;
    }
    if b[10] != b'T'
        || !digits(11..13)
        || b[13] != b':'
        || !digits(14..16)
        || b[16] != b':'
        || !digits(17..19)
    {
        return false;
    }
    let rest = &s[19..];
    let rest = match rest.strip_prefix('.') {
        Some(after) => {
            let n = after.bytes().take_while(u8::is_ascii_digit).count();
            if ![3, 6, 9].contains(&n) {
                return false;
            }
            &after[n..]
        }
        None => rest,
    };
    rest == "+00:00"
}

/// Seconds since the Unix epoch for the date and time fields of `s` (UTC), by the
/// days-from-civil rule, so the test needs no time crate.
fn epoch_seconds(s: &str) -> i64 {
    let n = |r: std::ops::Range<usize>| s[r].parse::<i64>().unwrap();
    let (y, m, d) = (n(0..4), n(5..7), n(8..10));
    let (hh, mm, ss) = (n(11..13), n(14..16), n(17..19));
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    days * 86400 + hh * 3600 + mm * 60 + ss
}

fn now_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn kos(dir: &Path, home: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_kos"))
        .args(args)
        .current_dir(dir)
        .env_remove("KOS_WORKSPACE")
        .env("HOME", home)
        .env("XDG_DATA_HOME", home.join("xdg"))
        .output()
        .unwrap()
}

fn find_named(root: &Path, name: &str, out: &mut Vec<PathBuf>) {
    for e in fs::read_dir(root).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            find_named(&p, name, out);
        } else if p.file_name().is_some_and(|n| n == name) {
            out.push(p);
        }
    }
}

#[test]
fn the_shape_check_accepts_what_chrono_wrote_and_refuses_the_zulu_form() {
    for ok in [
        "2026-11-18T15:52:42+00:00",
        "2026-11-18T15:52:42.282+00:00",
        "2026-11-18T15:52:42.282010+00:00",
        "2026-11-18T15:52:42.282010123+00:00",
    ] {
        assert!(is_rfc3339_utc_numeric_offset(ok), "should accept {ok}");
    }
    for bad in [
        "2026-11-18T15:52:42.282010Z",
        "2026-11-18T15:52:42.28+00:00",
        "2026-11-18T15:52:42+0000",
        "2026-11-18 15:52:42+00:00",
        "2026-11-18T15:52:42.282010+00:00 ",
        "2026-11-18T15:52:42[UTC]",
    ] {
        assert!(!is_rfc3339_utc_numeric_offset(bad), "should refuse {bad}");
    }
    assert_eq!(epoch_seconds("1970-01-01T00:00:00+00:00"), 0);
    assert_eq!(epoch_seconds("2026-11-18T00:00:00+00:00"), 1_794_960_000);
}

#[test]
fn kos_version_prints_the_build_time_in_the_chrono_shape() {
    let tmp = tempfile::tempdir().unwrap();
    let out = kos(tmp.path(), tmp.path(), &["version"]);
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    let line = text
        .lines()
        .find(|l| l.trim_start().starts_with("built:"))
        .expect("a built: line");
    let value = line
        .trim_start()
        .trim_start_matches("built:")
        .split_whitespace()
        .next()
        .unwrap();
    assert!(
        is_rfc3339_utc_numeric_offset(value),
        "built time `{value}` is not RFC 3339 UTC with a +00:00 offset"
    );
    let t = epoch_seconds(value);
    assert!(
        t > epoch_seconds("2026-01-01T00:00:00+00:00"),
        "built time {value} is implausibly early"
    );
    assert!(
        t <= now_seconds() + 60,
        "built time {value} is in the future"
    );
}

#[test]
fn the_orient_usage_log_timestamp_is_chrono_shaped_current_and_utc() {
    let tmp = tempfile::tempdir().unwrap();
    let ws = tmp.path().join("ws");
    let home = tmp.path().join("home");
    fs::create_dir_all(&ws).unwrap();
    fs::create_dir_all(&home).unwrap();
    assert!(
        kos(&ws, &home, &["init"]).status.success(),
        "kos init failed"
    );
    let before = now_seconds();
    let out = kos(&ws, &home, &["orient", "--log", "--json"]);
    assert!(
        out.status.success(),
        "orient failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let after = now_seconds();

    let mut logs = Vec::new();
    find_named(&home, "orient.jsonl", &mut logs);
    assert_eq!(
        logs.len(),
        1,
        "expected one orient.jsonl under the scratch home, found {logs:?}"
    );
    let text = fs::read_to_string(&logs[0]).unwrap();
    let last = text.lines().last().expect("a log line");
    let v: serde_json::Value = serde_json::from_str(last).unwrap();
    let ts = v["timestamp"].as_str().expect("timestamp is a JSON string");
    assert!(
        is_rfc3339_utc_numeric_offset(ts),
        "timestamp `{ts}` is not RFC 3339 UTC with a +00:00 offset"
    );
    let t = epoch_seconds(ts);
    assert!(
        t >= before - 2 && t <= after + 2,
        "timestamp {ts} is not the current UTC time ({before}..{after})"
    );

    let mut keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "bedrock_nodes",
            "charter_items",
            "duration_ms",
            "findings",
            "graveyard_nodes",
            "ideas",
            "json_output",
            "probes",
            "questions",
            "rd_briefs",
            "rd_findings",
            "standalone",
            "target",
            "timestamp",
        ]
    );
}
