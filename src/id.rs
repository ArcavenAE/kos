//! Minted finding ids (opaque-finding-ids design, sections 2.1 to 2.3).
//!
//! A finding id is `finding-<prefix>-<suffix>`: the prefix is the owning
//! graph's graph_id, lowercased, and the suffix is base36 of OS random bytes,
//! sized by bd's adaptive rule with a floor of 4. The id is minted at
//! authoring, so parallel sessions never allocate the same number.

use std::collections::HashSet;
use std::path::Path;

use crate::error::{KosError, Result};
use crate::findings::{self, FindingLoad};

/// The shortest suffix minted in any graph (ruled 2026-09-25; bd's is 3).
pub const MIN_LENGTH: usize = 4;
/// The longest suffix; growth stops here, as in bd.
pub const MAX_LENGTH: usize = 8;
/// bd's birthday-bound threshold for choosing a length over the graph's count.
pub const MAX_COLLISION_PROBABILITY: f64 = 0.25;
/// Attempts at each length before growing, as in bd.
pub const TRIES_PER_LENGTH: usize = 10;

const BASE36: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";

/// The suffix length for a graph holding `count` findings: the shortest length
/// from [`MIN_LENGTH`] whose birthday-bound collision probability over the
/// count is at or under [`MAX_COLLISION_PROBABILITY`] (bd's
/// `ComputeAdaptiveLength`, with the floor raised to 4).
pub fn adaptive_length(count: usize) -> usize {
    #[allow(clippy::cast_precision_loss)]
    let n = count as f64;
    for length in MIN_LENGTH..=MAX_LENGTH {
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let space = 36f64.powi(length as i32);
        let probability = 1.0 - (-(n * n) / (2.0 * space)).exp();
        if probability <= MAX_COLLISION_PROBABILITY {
            return length;
        }
    }
    MAX_LENGTH
}

/// How many random bytes bd encodes for a suffix of `length` characters.
fn byte_width(length: usize) -> usize {
    match length {
        3 => 2,
        5 | 6 => 4,
        7 | 8 => 5,
        _ => 3,
    }
}

/// Encode `data` (at most 16 bytes) as base36 of exactly `length` characters,
/// bd's way: big-endian integer, zero-padded on the left, and truncated to the
/// least significant digits when longer.
pub fn encode_base36(data: &[u8], length: usize) -> String {
    let mut n: u128 = data
        .iter()
        .take(16)
        .fold(0u128, |acc, &b| (acc << 8) | u128::from(b));
    let mut digits: Vec<u8> = Vec::new();
    while n > 0 {
        let d = usize::try_from(n % 36).unwrap_or(0);
        digits.push(BASE36[d]);
        n /= 36;
    }
    while digits.len() < length {
        digits.push(b'0');
    }
    digits.truncate(length);
    digits.reverse();
    String::from_utf8(digits).unwrap_or_default()
}

/// Mint `finding-<prefix>-<suffix>` with an injected random source and
/// existence check: sixteen random bytes per attempt, bd's byte widths and
/// base36, ten attempts per length starting at [`adaptive_length`] of
/// `count`, growing through [`MAX_LENGTH`].
pub fn mint_with(
    prefix: &str,
    count: usize,
    exists: &dyn Fn(&str) -> bool,
    random: &mut dyn FnMut(&mut [u8; 16]) -> Result<()>,
) -> Result<String> {
    let start = adaptive_length(count);
    for length in start..=MAX_LENGTH {
        for _ in 0..TRIES_PER_LENGTH {
            let mut bytes = [0u8; 16];
            random(&mut bytes)?;
            let suffix = encode_base36(&bytes[..byte_width(length)], length);
            let id = format!("finding-{prefix}-{suffix}");
            if !exists(&id) {
                return Ok(id);
            }
        }
    }
    Err(KosError::Id {
        message: format!(
            "could not mint a finding id with prefix '{prefix}' after {TRIES_PER_LENGTH} attempts at each length {start} to {MAX_LENGTH}"
        ),
    })
}

/// Fill `buf` from the operating system's random source.
fn os_random(buf: &mut [u8; 16]) -> Result<()> {
    getrandom::getrandom(buf).map_err(|e| KosError::Id {
        message: format!("the OS random source failed: {e}"),
    })
}

/// Mint a finding id for the graph at `graph_root`. The prefix is the graph's
/// graph_id, lowercased; the existence check and the adaptive length run over
/// the findings already in the graph's `findings/`.
pub fn mint_finding_id(graph_root: &Path) -> Result<String> {
    let manifest = findings::load_manifest(graph_root).ok_or_else(|| KosError::Id {
        message: format!(
            "no readable kos.yaml at {}; a minted id takes its prefix from graph_id",
            graph_root.display()
        ),
    })?;
    let prefix = manifest.graph_id.to_lowercase();
    if !findings::graph_id_is_prefix_shaped(&prefix) {
        return Err(KosError::Id {
            message: format!(
                "graph_id '{}' cannot be a finding-id prefix: it must start with a letter and use only [a-z0-9-] once lowercased",
                manifest.graph_id
            ),
        });
    }
    let known = findings::known_prefixes(graph_root);
    let loads = findings::load_findings(&graph_root.join("findings"))?;
    let mut taken: HashSet<String> = HashSet::new();
    for load in &loads {
        match load {
            FindingLoad::Loaded(l) => {
                taken.insert(findings::finding_key(&l.node.id, &known));
                taken.insert(findings::finding_key(&l.stem, &known));
            }
            FindingLoad::Unloadable { path, .. } => {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    taken.insert(findings::finding_key(stem, &known));
                }
            }
        }
    }
    mint_with(
        &prefix,
        loads.len(),
        &|id| taken.contains(id),
        &mut os_random,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptive_length_is_four_at_the_kos_and_orc_counts() {
        assert_eq!(adaptive_length(0), 4);
        assert_eq!(adaptive_length(51), 4);
        assert_eq!(adaptive_length(187), 4);
    }

    #[test]
    fn adaptive_length_grows_with_the_count() {
        assert_eq!(adaptive_length(2000), 5);
        assert_eq!(adaptive_length(40_000), 7);
    }

    #[test]
    fn encode_base36_matches_bd() {
        // Vectors from bd's EncodeBase36 (internal/idgen/hash.go).
        assert_eq!(encode_base36(&[0xff, 0xff, 0xff], 4), "zldr");
        assert_eq!(encode_base36(&[0, 0, 1], 4), "0001");
        assert_eq!(encode_base36(&[0x12, 0x34, 0x56, 0x78], 5), "1u7i0");
        assert_eq!(encode_base36(&[0xff; 5], 8), "e13wu1of");
    }

    /// A random source that hands out the given fills in order.
    fn scripted(fills: Vec<u8>) -> impl FnMut(&mut [u8; 16]) -> Result<()> {
        let mut next = fills.into_iter();
        move |buf: &mut [u8; 16]| {
            let b = next.next().expect("scripted random source ran out");
            *buf = [b; 16];
            Ok(())
        }
    }

    #[test]
    fn minted_id_carries_the_owning_prefix() {
        let id = mint_with("kos", 51, &|_| false, &mut scripted(vec![7])).unwrap();
        assert!(id.starts_with("finding-kos-"), "{id}");
        assert_eq!(id.len(), "finding-kos-".len() + 4);
    }

    #[test]
    fn an_existing_id_is_retried_at_the_same_length() {
        let first = mint_with("kos", 51, &|_| false, &mut scripted(vec![1])).unwrap();
        let taken = first.clone();
        let id = mint_with("kos", 51, &move |c| c == taken, &mut scripted(vec![1, 2])).unwrap();
        assert_ne!(id, first);
        assert_eq!(id.len(), first.len(), "a retry keeps the length");
    }

    #[test]
    fn ten_misses_grow_the_length() {
        let id = mint_with(
            "kos",
            51,
            &|c| c.len() == "finding-kos-".len() + 4,
            &mut scripted((0..11).collect()),
        )
        .unwrap();
        assert_eq!(id.len(), "finding-kos-".len() + 5);
    }

    #[test]
    fn exhaustion_through_the_longest_length_is_an_error() {
        let fills = (0..=255u8).cycle().take(TRIES_PER_LENGTH * 5).collect();
        assert!(mint_with("kos", 51, &|_| true, &mut scripted(fills)).is_err());
    }
}
