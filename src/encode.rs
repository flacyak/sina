//! Builds the data codewords: mode header, payload, terminator and padding.
use crate::ecc::num_data_codewords;
use crate::mode::{ALPHANUM_CHARS, EC, Encoding};
use encoding_rs::SHIFT_JIS;

const MIN_VERSION: u8 = 1;
const MAX_VERSION: u8 = 40;
/// Alternating filler bytes defined by the spec for unused data capacity.
const PAD_BYTES: [u8; 2] = [0xEC, 0x11];

/// Append-only bit sequence, most significant bit first.
#[derive(Default)]
struct BitBuf {
    bits: Vec<bool>,
}

impl BitBuf {
    fn push(&mut self, value: u32, len: u8) {
        for i in (0..len).rev() {
            self.bits.push((value >> i) & 1 == 1);
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.bits
            .chunks(8)
            .map(|chunk| {
                chunk.iter().enumerate().fold(0, |acc, (i, &bit)| acc | (u8::from(bit) << (7 - i)))
            })
            .collect()
    }
}

pub struct Encoded {
    pub version: u8,
    pub ec: EC,
    pub codewords: Vec<u8>,
}

/// Encodes `text` into data codewords using the smallest version that fits.
pub fn encode(text: &str, ec: EC) -> Result<Encoded, String> {
    let mode = Encoding::detect(text);
    let payload = encode_payload(text, mode);
    let char_count = match mode {
        Encoding::Byte => text.len(),
        _ => text.chars().count(),
    };
    let fits = |version: u8| {
        let needed = 4 + usize::from(mode.count_bits(version)) + payload.bits.len();
        needed <= num_data_codewords(version, ec) * 8
    };
    let version = (MIN_VERSION..=MAX_VERSION)
        .find(|&v| fits(v))
        .ok_or_else(|| format!("input is too long for a QR code at level {ec:?}"))?;
    let capacity = num_data_codewords(version, ec) * 8;

    let mut bits = BitBuf::default();
    bits.push(mode.indicator(), 4);
    bits.push(char_count as u32, mode.count_bits(version));
    bits.bits.extend(&payload.bits);
    let terminator = 4.min(capacity - bits.bits.len());
    bits.push(0, terminator as u8);
    bits.push(0, ((8 - bits.bits.len() % 8) % 8) as u8);
    let padding = (capacity - bits.bits.len()) / 8;
    for i in 0..padding {
        bits.push(u32::from(PAD_BYTES[i % 2]), 8);
    }
    Ok(Encoded { version, ec, codewords: bits.to_bytes() })
}

fn encode_payload(text: &str, mode: Encoding) -> BitBuf {
    let mut bits = BitBuf::default();
    match mode {
        Encoding::Num => {
            for chunk in text.as_bytes().chunks(3) {
                let value = chunk.iter().fold(0, |acc, &b| acc * 10 + u32::from(b - b'0'));
                bits.push(value, 1 + 3 * chunk.len() as u8);
            }
        }
        Encoding::Alphanum => {
            let position = |b: u8| ALPHANUM_CHARS.find(b as char).unwrap_or(0) as u32;
            for chunk in text.as_bytes().chunks(2) {
                let value = chunk.iter().fold(0, |acc, &b| acc * 45 + position(b));
                bits.push(value, 1 + 5 * chunk.len() as u8);
            }
        }
        Encoding::Byte => {
            for &b in text.as_bytes() {
                bits.push(u32::from(b), 8);
            }
        }
        Encoding::Kanji => {
            let (bytes, _, _) = SHIFT_JIS.encode(text);
            for pair in bytes.chunks_exact(2) {
                let word = u16::from_be_bytes([pair[0], pair[1]]);
                let offset = if word <= 0x9FFC { word - 0x8140 } else { word - 0xC140 };
                bits.push(u32::from((offset >> 8) * 0xC0 + (offset & 0xFF)), 13);
            }
        }
    }
    bits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_world_matches_reference_codewords() {
        let encoded = encode("HELLO WORLD", EC::M).unwrap();
        assert_eq!(encoded.version, 1);
        let expected = [
            0x20, 0x5B, 0x0B, 0x78, 0xD1, 0x72, 0xDC, 0x4D, 0x43, 0x40, 0xEC, 0x11, 0xEC, 0x11,
            0xEC, 0x11,
        ];
        assert_eq!(encoded.codewords, expected);
    }

    #[test]
    fn rejects_oversized_input() {
        assert!(encode(&"A".repeat(5000), EC::L).is_err());
    }
}
