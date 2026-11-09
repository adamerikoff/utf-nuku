//! Decode input: text typed by the user → bytes.
//! Bytes are separated by spaces. Each one is either
//! - 2 hex digits:  c3
//! - 8 bits:        11000011   (underscores allowed: 1100_0011)

/// Read one written byte: 8 bits or 2 hex digits.
fn parse_byte(byte_text: &str) -> Result<u8, String> {
    let digits = byte_text.replace('_', "");

    let parsed_byte = if digits.len() == 8 {
        u8::from_str_radix(&digits, 2)
    } else if digits.len() == 2 {
        u8::from_str_radix(&digits, 16)
    } else {
        return Err(format!(
            "'{byte_text}' is not a byte: use 2 hex digits (c3) or 8 bits (11000011)"
        ));
    };

    match parsed_byte {
        Ok(byte) => Ok(byte),
        Err(_) => Err(format!(
            "'{byte_text}' is not a byte: use 2 hex digits (c3) or 8 bits (11000011)"
        )),
    }
}

/// Read all bytes, separated by spaces.
///
/// ```text
/// "48 c3 a9"                        → [0b0100_1000, 0b1100_0011, 0b1010_1001]
/// "01001000 11000011 10101001"      → the same bytes
/// ```
pub fn parse_byte_input(input_text: &str) -> Result<Vec<u8>, String> {
    let mut input_bytes = Vec::new();
    // split_whitespace = cut at every run of spaces/newlines
    for byte_text in input_text.split_whitespace() {
        let byte = parse_byte(byte_text)?;
        input_bytes.push(byte);
    }
    Ok(input_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_bytes() {
        assert_eq!(
            parse_byte_input("48 c3 A9"),
            Ok(vec![0b0100_1000, 0b1100_0011, 0b1010_1001])
        );
    }

    #[test]
    fn bit_bytes() {
        assert_eq!(
            parse_byte_input("01001000 1100_0011"),
            Ok(vec![0b0100_1000, 0b1100_0011])
        );
    }

    #[test]
    fn hex_and_bits_mixed() {
        assert_eq!(
            parse_byte_input("48 11000011"),
            Ok(vec![0b0100_1000, 0b1100_0011])
        );
    }

    #[test]
    fn empty_input() {
        assert_eq!(parse_byte_input("  "), Ok(vec![]));
    }

    #[test]
    fn bad_bytes() {
        assert!(parse_byte_input("zz").is_err(), "not hex");
        assert!(parse_byte_input("c").is_err(), "1 digit");
        assert!(
            parse_byte_input("11000012").is_err(),
            "8 digits but not bits"
        );
    }
}
