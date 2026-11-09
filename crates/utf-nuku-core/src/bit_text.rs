//! Numbers → bit text, for showing traces (CLI -v and the GUI).

/// `value` as `bit_count` bits, with `_` every 4 bits counted from the right.
///
/// ```text
/// bits_text(0b1110_1001, 11) → "000_1110_1001"
/// bits_text(0b110, 3)        → "110"
/// ```
pub fn bits_text(value: u32, bit_count: u32) -> String {
    let mut grouped_bits = String::new();

    // walk from the leftmost bit down to bit 0
    for bit_position in (0..bit_count).rev() {
        let shifted_value = value >> bit_position;
        let bit = shifted_value & 0b1;
        if bit == 1 {
            grouped_bits.push('1');
        } else {
            grouped_bits.push('0');
        }

        // a group of 4 ends here (but not after the last bit)
        let is_group_end = bit_position % 4 == 0 && bit_position != 0;
        if is_group_end {
            grouped_bits.push('_');
        }
    }

    grouped_bits
}

/// UTF-8: how many data bits the whole char has (lead byte data + 6 per continuation byte).
/// Used to print the code point with exactly the bits UTF-8 stores: 7, 11, 16 or 21.
pub fn utf8_code_point_bit_count(
    lead_byte_header_bit_count: u32,
    byte_sequence_count: usize,
) -> u32 {
    let lead_byte_data_bit_count = 8 - lead_byte_header_bit_count;
    let continuation_byte_count = (byte_sequence_count - 1) as u32;
    lead_byte_data_bit_count + 6 * continuation_byte_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_text_groups_from_the_right() {
        assert_eq!(bits_text(0b1110_1001, 11), "000_1110_1001");
        assert_eq!(bits_text(0b110, 3), "110");
        assert_eq!(bits_text(0b0, 1), "0");
        assert_eq!(bits_text(0b1100_0011, 8), "1100_0011");
    }

    #[test]
    fn utf8_code_point_bit_counts() {
        assert_eq!(utf8_code_point_bit_count(1, 1), 7);
        assert_eq!(utf8_code_point_bit_count(3, 2), 11);
        assert_eq!(utf8_code_point_bit_count(4, 3), 16);
        assert_eq!(utf8_code_point_bit_count(5, 4), 21);
    }
}
