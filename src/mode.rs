//! Encoding modes and error correction levels.
use encoding_rs::SHIFT_JIS;
use std::str::FromStr;

/// Characters allowed in alphanumeric mode, in their encoded order.
pub const ALPHANUM_CHARS: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Encoding {
    Num,
    Alphanum,
    Byte,
    Kanji,
}

impl Encoding {
    /// Picks the most compact mode able to represent every character of `s`.
    pub fn detect(s: &str) -> Encoding {
        if s.chars().all(|c| c.is_ascii_digit()) {
            return Encoding::Num;
        }
        if s.chars().all(|c| ALPHANUM_CHARS.contains(c)) {
            return Encoding::Alphanum;
        }
        let (bytes, _, had_errors) = SHIFT_JIS.encode(s);
        if !had_errors && bytes.chunks(2).all(is_qr_kanji) {
            Encoding::Kanji
        } else {
            Encoding::Byte
        }
    }

    pub fn indicator(self) -> u32 {
        match self {
            Encoding::Num => 0b0001,
            Encoding::Alphanum => 0b0010,
            Encoding::Byte => 0b0100,
            Encoding::Kanji => 0b1000,
        }
    }

    /// Width of the character count field for the given symbol version.
    pub fn count_bits(self, version: u8) -> u8 {
        let tier = match version {
            1..=9 => 0,
            10..=26 => 1,
            _ => 2,
        };
        match self {
            Encoding::Num => [10, 12, 14][tier],
            Encoding::Alphanum => [9, 11, 13][tier],
            Encoding::Byte => [8, 16, 16][tier],
            Encoding::Kanji => [8, 10, 12][tier],
        }
    }
}

/// True when the Shift JIS byte pair lies in one of the two ranges Kanji mode can encode.
fn is_qr_kanji(pair: &[u8]) -> bool {
    let &[hi, lo] = pair else { return false };
    let word = u16::from_be_bytes([hi, lo]);
    (0x8140..=0x9FFC).contains(&word) || (0xE040..=0xEBBF).contains(&word)
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum EC {
    L,
    M,
    Q,
    H,
}

impl EC {
    /// Row index into the per-level lookup tables.
    pub fn index(self) -> usize {
        self as usize
    }

    /// Two-bit level code stored in the format information.
    pub fn format_bits(self) -> u32 {
        match self {
            EC::L => 0b01,
            EC::M => 0b00,
            EC::Q => 0b11,
            EC::H => 0b10,
        }
    }
}

impl FromStr for EC {
    type Err = String;

    fn from_str(s: &str) -> Result<EC, String> {
        match s.to_ascii_lowercase().as_str() {
            "l" => Ok(EC::L),
            "m" => Ok(EC::M),
            "q" => Ok(EC::Q),
            "h" => Ok(EC::H),
            _ => Err(format!("unknown error correction level '{s}' (expected l, m, q or h)")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_smallest_mode() {
        assert_eq!(Encoding::detect("0123"), Encoding::Num);
        assert_eq!(Encoding::detect("HTTPS://ABC.COM"), Encoding::Alphanum);
        assert_eq!(Encoding::detect("https://abc.com"), Encoding::Byte);
        assert_eq!(Encoding::detect("https://abcÁÇ.de"), Encoding::Byte);
        assert_eq!(Encoding::detect("こんにちは"), Encoding::Kanji);
        assert_eq!(Encoding::detect("こんにちはA"), Encoding::Byte);
    }
}
