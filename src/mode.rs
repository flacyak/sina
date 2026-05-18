pub use crate::encode::Encoding;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum EC {
    L,
    M,
    Q,
    H,
}

pub fn mode_details(enc: Encoding, ec: EC) -> (u8, &'static str, u8) {
    (num_mode(ec, enc), mode_indicator(enc), num_bits(enc))
}

fn num_mode(ec: EC, enc: Encoding) -> u8 {
    match (ec, enc) {
        (EC::L, Encoding::Num) => 187,
        (EC::L, Encoding::Alphanum) => 114,
        (EC::L, Encoding::Byte) => 78,
        (EC::L, Encoding::Kanji) => 48,
        (EC::M, Encoding::Num) => 149,
        (EC::M, Encoding::Alphanum) => 90,
        (EC::M, Encoding::Byte) => 62,
        (EC::M, Encoding::Kanji) => 38,
        (EC::Q, Encoding::Num) => 111,
        (EC::Q, Encoding::Alphanum) => 67,
        (EC::Q, Encoding::Byte) => 46,
        (EC::Q, Encoding::Kanji) => 28,
        (EC::H, Encoding::Num) => 82,
        (EC::H, Encoding::Alphanum) => 50,
        (EC::H, Encoding::Byte) => 34,
        (EC::H, Encoding::Kanji) => 21,
    }
}

fn mode_indicator(enc: Encoding) -> &'static str {
    match enc {
        Encoding::Num => "0001",
        Encoding::Alphanum => "0010",
        Encoding::Byte => "0100",
        Encoding::Kanji => "1000",
    }
}

fn num_bits(enc: Encoding) -> u8 {
    match enc {
        Encoding::Num => 10,
        Encoding::Alphanum => 9,
        Encoding::Byte => 8,
        Encoding::Kanji => 8,
    }
}
