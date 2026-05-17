mod encode;
pub use encode::Encoding;

enum EC {
    L,
    M,
    Q,
    H,
}

pub fn num_mode(ec: EC, enc: Encoding) -> u32 {
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
        (_, _) => 0,
    }
}

