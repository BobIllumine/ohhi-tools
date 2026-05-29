//! Legal complete-line enumeration used by the combo-pool fill algorithm.

/// Returns all valid red-bit patterns for a complete line of length `n`.
///
/// A pattern is kept iff:
/// - Exactly `n/2` bits are set (balance: equal red and blue).
/// - No three consecutive reds: `x & (x >> 1) & (x >> 2) == 0` within the window.
/// - No three consecutive blues: same test on the complement within the window.
///
/// The caller interprets each `u16` as a red mask; blue = `mask & !pattern`.
pub(crate) fn legal_lines(n: usize) -> Vec<u16> {
    assert!(n <= 16, "line length must fit in u16");
    let mask: u16 = if n == 16 { u16::MAX } else { (1u16 << n) - 1 };
    let half = (n / 2) as u32;

    (0u16..=(mask))
        .filter(|&x| {
            // Balance: exactly n/2 ones.
            x.count_ones() == half
                // No triple of reds.
                && (x & (x >> 1) & (x >> 2)) == 0
                // No triple of blues.
                && {
                    let blue = (!x) & mask;
                    (blue & (blue >> 1) & (blue >> 2)) == 0
                }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legal_lines_n2() {
        let lines = legal_lines(2);
        let mut lines = lines.clone();
        lines.sort();
        // Only 0b01 and 0b10 are balanced 2-bit patterns.
        assert_eq!(lines, vec![0b01, 0b10]);
    }

    #[test]
    fn legal_lines_n4_count() {
        let lines = legal_lines(4);
        // All 6 balanced 4-bit patterns survive (none has 3 adjacent equal in a 4-wide line).
        assert_eq!(lines.len(), 6);
    }

    #[test]
    fn legal_lines_n4_no_triples() {
        for &pat in &legal_lines(4) {
            let blue = (!pat) & 0b1111;
            assert_eq!(pat & (pat >> 1) & (pat >> 2), 0, "red triple in {pat:04b}");
            assert_eq!(blue & (blue >> 1) & (blue >> 2), 0, "blue triple in {pat:04b}");
        }
    }

    #[test]
    fn legal_lines_n6_all_balanced() {
        for &pat in &legal_lines(6) {
            assert_eq!(pat.count_ones(), 3, "unbalanced pattern {pat:06b}");
        }
    }
}
