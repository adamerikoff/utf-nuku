//! Bit helpers: AND, OR, shift right, shift left. No UTF knowledge.
//!
//! How UTF-8 uses them:
//! - `and`         → cut out a sequence   (e.g. the last 6 bits)
//! - `shift_right` → bring the next sequence down
//! - `shift_left`  → put a header in front
//! - `or`          → glue header + data together

/// AND: a bit is 1 only if it is 1 in BOTH `value` and `mask`.
/// Used with a mask to keep some bits and erase the rest.
///
/// ```text
///   1110_1001   (value)
/// & 0011_1111   (mask "keep last 6")
/// = 0010_1001
/// ```
pub(crate) fn and(value: u32, mask: u32) -> u32 {
    let result = value & mask;
    return result;
}

/// OR: a bit is 1 if it is 1 in EITHER `left` or `right`.
/// Used to glue two sequences that don't overlap (header + data).
///
/// ```text
///   1000_0000   (left  = header 10 in front)
/// | 0010_1001   (right = data)
/// = 1010_1001
/// ```
pub(crate) fn or(left: u32, right: u32) -> u32 {
    let result = left | right;
    return result;
}

/// Shift right: move all bits `step` places right; the rightmost bits fall off.
/// Used to bring the next sequence down to the bottom.
///
/// ```text
/// 1110_1001 >> 6 = 0000_0011
/// ```
pub(crate) fn shift_right(value: u32, step: u32) -> u32 {
    let result = value >> step;
    return result;
}

/// Shift left: move all bits `step` places left; zeros come in on the right.
/// Used to move a header to the front of a byte.
///
/// ```text
/// 0000_0110 << 5 = 1100_0000
/// ```
pub(crate) fn shift_left(value: u32, step: u32) -> u32 {
    let result = value << step;
    return result;
}

/// Mask of `bit_count` ones at the right: 6 → 0b0011_1111.
/// (1 << 6 = 0b0100_0000, minus 1 = 0b0011_1111)
pub(crate) fn low_mask(bit_count: u32) -> u32 {
    let single_one = shift_left(1, bit_count);
    single_one - 1
}

/// Split one sequence of bits into a lead sequence + continuation sequences.
/// Continuation sequences are `continuation_size` bits each, cut from the RIGHT.
/// The lead sequence is whatever is left on the left.
/// Returns `(lead_sequence, continuation_sequences)`, continuation sequences in
/// left-to-right order.
///
/// ```text
/// split(000_1110_1001, 1, 6)
///   pass 1: and(000_1110_1001, 0011_1111) = 10_1001   → continuation sequence
///           shift_right(000_1110_1001, 6) = 0_0011    → what's left
///   → (0_0011, [10_1001])
/// ```
pub(crate) fn split(
    bits: u32,
    continuation_count: usize,
    continuation_size: u32,
) -> (u32, Vec<u32>) {
    let continuation_mask = low_mask(continuation_size);
    let mut lead_sequence = bits;
    let mut continuation_sequences = Vec::new();

    for _ in 0..continuation_count {
        // cut out the last `continuation_size` bits
        let continuation_sequence = and(lead_sequence, continuation_mask);
        continuation_sequences.push(continuation_sequence);

        // drop them; what's left shrinks toward the lead sequence
        lead_sequence = shift_right(lead_sequence, continuation_size);
    }

    // We cut from the right, so the sequences are backwards. Put them left-to-right.
    continuation_sequences.reverse();
    (lead_sequence, continuation_sequences)
}

/// Join a lead sequence + continuation sequences back into one sequence of bits.
/// For each continuation sequence: make room with `shift_left`, put it in with `or`.
///
/// ```text
/// join(0_0011, [10_1001], 6)
///   shift_left(0_0011, 6)       = 000_1100_0000
///   or(000_1100_0000, 10_1001)  = 000_1110_1001
/// ```
pub(crate) fn join(
    lead_sequence: u32,
    continuation_sequences: Vec<u32>,
    continuation_size: u32,
) -> u32 {
    let mut joined_bits = lead_sequence;

    for continuation_sequence in continuation_sequences {
        // make room on the right
        let joined_bits_with_room = shift_left(joined_bits, continuation_size);
        // put the sequence into the room
        joined_bits = or(joined_bits_with_room, continuation_sequence);
    }

    joined_bits
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- and ----------

    #[test]
    fn and_keeps_last_6_bits() {
        let input = 0b1110_1001; //    é
        let mask = 0b0011_1111; //     keep the last 6 bits
        let expected = 0b0010_1001; // last 6 bits of é

        let got = and(input, mask);

        assert_eq!(
            got, expected,
            "\n  and({input:#010b}, {mask:#010b})\n  expected {expected:#010b}\n  got      {got:#010b}"
        );
    }

    // ---------- or ----------

    #[test]
    fn or_adds_continuation_header() {
        let header = 0b1000_0000; //   continuation header 10, already in front
        let data = 0b0010_1001; //     6 data bits
        let expected = 0b1010_1001; // second byte of é

        let got = or(header, data);

        assert_eq!(
            got, expected,
            "\n  or({header:#010b}, {data:#010b})\n  expected {expected:#010b}\n  got      {got:#010b}"
        );
    }

    // ---------- shift_right ----------

    #[test]
    fn shift_right_drops_last_6_bits() {
        let input = 0b1110_1001; //    é
        let step = 6; //               drop the 6 bits we already used
        let expected = 0b0000_0011; // what's left for the first byte

        let got = shift_right(input, step);

        assert_eq!(
            got, expected,
            "\n  shift_right({input:#010b}, {step})\n  expected {expected:#010b}\n  got      {got:#010b}"
        );
    }

    // ---------- shift_left ----------

    #[test]
    fn shift_left_moves_header_to_front() {
        let header = 0b0000_0110; //   2-byte start header 110
        let step = 5; //               free bits in the first byte
        let expected = 0b1100_0000; // header now at the front of the byte

        let got = shift_left(header, step);

        assert_eq!(
            got, expected,
            "\n  shift_left({header:#010b}, {step})\n  expected {expected:#010b}\n  got      {got:#010b}"
        );
    }

    // ---------- low_mask ----------

    #[test]
    fn low_mask_sizes() {
        assert_eq!(low_mask(3), 0b0000_0111);
        assert_eq!(low_mask(5), 0b0001_1111);
        assert_eq!(low_mask(6), 0b0011_1111);
        assert_eq!(low_mask(7), 0b0111_1111);
    }

    // ---------- split / join ----------

    #[test]
    fn split_into_6_bit_sequences() {
        // nothing to cut
        assert_eq!(split(0b0100_1000, 0, 6), (0b0100_1000, vec![]));
        // é: 000_1110_1001 → 00011 | 101001
        assert_eq!(split(0b000_1110_1001, 1, 6), (0b0_0011, vec![0b10_1001]));
        // 😀: 0001_1111_0110_0000_0000 → 000 | 011111 | 011000 | 000000
        assert_eq!(
            split(0b0001_1111_0110_0000_0000, 3, 6),
            (0b000, vec![0b01_1111, 0b01_1000, 0b00_0000])
        );
    }

    #[test]
    fn join_is_the_opposite_of_split() {
        assert_eq!(join(0b0100_1000, vec![], 6), 0b0100_1000);
        assert_eq!(join(0b0_0011, vec![0b10_1001], 6), 0b000_1110_1001);
        assert_eq!(
            join(0b000, vec![0b01_1111, 0b01_1000, 0b00_0000], 6),
            0b0001_1111_0110_0000_0000
        );
    }
}
