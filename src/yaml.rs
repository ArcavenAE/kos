//! The one place kos configures its YAML reader.
//!
//! The reader was serde_yaml; serde-saphyr replaced it (RUSTSEC-2024-0449).
//! serde-saphyr defaults to YAML 1.1 words in untyped values, so `yes`, `on`
//! and the like would read as booleans where kos has always read them as text.
//! `strict_booleans` keeps only `true` and `false` as booleans, and
//! `AsString` keeps a non-finite float such as `.inf` loadable.

use serde::de::DeserializeOwned;
use serde_saphyr::{Options, options::NonFiniteFloatPolicy};

fn options() -> Options {
    let mut o = Options::default();
    o.strict_booleans = true;
    o.non_finite_float_policy = NonFiniteFloatPolicy::AsString;
    o
}

/// Parse one YAML document with kos's reader settings.
pub fn from_str<T: DeserializeOwned>(text: &str) -> Result<T, serde_saphyr::Error> {
    serde_saphyr::from_str_with_options(text, options())
}
