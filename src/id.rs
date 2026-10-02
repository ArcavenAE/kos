//! Minted finding ids (opaque-finding-ids design, sections 2.1 to 2.3).

use crate::error::Result;

/// The shortest suffix minted in any graph (ruled 2026-09-25; bd's is 3).
pub const MIN_LENGTH: usize = 4;
/// The longest suffix; growth stops here, as in bd.
pub const MAX_LENGTH: usize = 8;
/// bd's birthday-bound threshold for choosing a length over the graph's count.
pub const MAX_COLLISION_PROBABILITY: f64 = 0.25;
/// Attempts at each length before growing, as in bd.
pub const TRIES_PER_LENGTH: usize = 10;

/// The suffix length for a graph holding `count` findings.
pub fn adaptive_length(_count: usize) -> usize {
    0
}

/// Encode `data` as base36 of exactly `length` characters, bd's way.
pub fn encode_base36(_data: &[u8], _length: usize) -> String {
    String::new()
}

/// Mint `finding-<prefix>-<suffix>`.
pub fn mint_with(
    _prefix: &str,
    _count: usize,
    _exists: &dyn Fn(&str) -> bool,
    _random: &mut dyn FnMut(&mut [u8; 16]) -> Result<()>,
) -> Result<String> {
    Ok(String::new())
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
