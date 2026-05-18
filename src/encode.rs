use crate::mode::{EC, mode_details};
use encoding_rs::SHIFT_JIS;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum Encoding {
    Num,
    Alphanum,
    Byte,
    Kanji, // only kanji
}
fn smallest_encode(s: &String) -> Encoding {
    let mut current = Encoding::Num;

    for c in s.chars() {
        match current {
            Encoding::Num => {
                if !c.is_ascii_digit() {
                    if c.is_ascii_alphanumeric() || is_alpha_symbol(c) {
                        current = Encoding::Alphanum
                    } else {
                        current = Encoding::Kanji;
                    }
                }
            }
            Encoding::Alphanum => {
                if !c.is_ascii_alphanumeric() && !is_alpha_symbol(c) {
                    current = Encoding::Kanji;
                }
            }
            _ => break,
        }
    }
    if current == Encoding::Kanji {
        // Encode the native Rust UTF-8 string into actual Shift_JIS bytes
        let (bytes, _, has_errors) = SHIFT_JIS.encode(&s);

        if !has_errors && is_jis_encoding(&bytes) {
            Encoding::Kanji
        } else {
            Encoding::Byte
        }
    } else {
        current
    }
}
fn is_alpha_symbol(c: char) -> bool {
    let symbols = ['$', '%', '*', '+', '-', '.', '/', ':', ' '];
    for x in symbols {
        if c == x {
            return true;
        }
    }
    false
}
fn is_jis_encoding(bytes: &[u8]) -> bool {
    if bytes.len() % 2 != 0 || bytes.is_empty() {
        return false;
    }
    let mut i = 0;
    while i < bytes.len() {
        let b1 = bytes[i];
        let b2 = bytes[i + 1];
        // Valid Shift-JIS Kanji ranges
        let is_valid_lead = (0x81..=0x9F).contains(&b1) || (0xE0..=0xEB).contains(&b1);
        let is_valid_trail = (0x40..=0xFC).contains(&b2) && b2 != 0x7F;

        if !is_valid_lead || !is_valid_trail {
            return false;
        }
        //upper byte block for QR Kanji
        if b1 == 0xEA && b2 > 0xA4 {
            return false;
        }
        i += 2; // next double byte
    }
    true
}
pub fn encode(s: String, size: &'static str) -> String {
    let enc: Encoding = smallest_encode(&s);
    let mut ec: EC = EC::L;

    match size {
        "l" => ec = EC::L,
        "m" => ec = EC::M,
        "h" => ec = EC::H,
        "q" => ec = EC::Q,
        _ => ec = EC::L,
    }

    let (mode, cnt, data) = mode_details(enc, ec);
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode() {
        let queries = [
            ("https://abc.com", Encoding::Alphanum),
            ("0123", Encoding::Num),
            ("https://abcÁÇ.de", Encoding::Byte),
            ("こんにちは", Encoding::Kanji),
        ];
        for (url, encoding) in &queries {
            assert_eq!(smallest_encode(&url.to_string()), *encoding);
        }
    }
}
