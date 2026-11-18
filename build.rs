use std::env;
use std::process::Command;

fn main() {
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());

    let commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    let build_time = utc_now_rfc3339();

    let channel = env::var("KOS_CHANNEL").unwrap_or_else(|_| "alpha".to_string());
    let tag = env::var("KOS_TAG").unwrap_or_else(|_| format!("{version}-{commit}"));

    let long_version = format!("{tag} ({commit}, {channel})");

    println!("cargo:rustc-env=KOS_VERSION={version}");
    println!("cargo:rustc-env=KOS_COMMIT={commit}");
    println!("cargo:rustc-env=KOS_BUILD_TIME={build_time}");
    println!("cargo:rustc-env=KOS_CHANNEL={channel}");
    println!("cargo:rustc-env=KOS_TAG={tag}");
    println!("cargo:rustc-env=KOS_LONG_VERSION={long_version}");

    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-env-changed=KOS_CHANNEL");
    println!("cargo:rerun-if-env-changed=KOS_TAG");
}

/// The current time as RFC 3339 in UTC with a numeric `+00:00` offset and 3, 6 or
/// 9 fractional digits only when the clock has them: the shape
/// `chrono::Utc::now().to_rfc3339()` wrote (tests/timestamp_format.rs pins it).
/// Taken from a `jiff::Timestamp`, which is UTC, never from a zoned local time.
fn utc_now_rfc3339() -> String {
    let now = jiff::Timestamp::now();
    let nanos = now.subsec_nanosecond();
    let frac = if nanos == 0 {
        String::new()
    } else if nanos % 1_000_000 == 0 {
        format!(".{:03}", nanos / 1_000_000)
    } else if nanos % 1_000 == 0 {
        format!(".{:06}", nanos / 1_000)
    } else {
        format!(".{nanos:09}")
    };
    format!("{}{frac}+00:00", now.strftime("%Y-%m-%dT%H:%M:%S"))
}
