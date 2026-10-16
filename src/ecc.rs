//! Reed-Solomon error correction over GF(2^8) and codeword interleaving.
use crate::mode::EC;

/// Error correction codewords per block, indexed by [level][version - 1].
#[rustfmt::skip]
const ECC_PER_BLOCK: [[u8; 40]; 4] = [
    [7, 10, 15, 20, 26, 18, 20, 24, 30, 18, 20, 24, 26, 30, 22, 24, 28, 30, 28, 28, 28, 28, 30, 30, 26, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
    [10, 16, 26, 18, 24, 16, 18, 22, 22, 26, 30, 22, 22, 24, 24, 28, 28, 26, 26, 26, 26, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28],
    [13, 22, 18, 26, 18, 24, 18, 22, 20, 24, 28, 26, 24, 20, 30, 24, 28, 28, 26, 30, 28, 30, 30, 30, 30, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
    [17, 28, 22, 16, 22, 28, 26, 26, 24, 28, 24, 28, 22, 24, 24, 30, 28, 28, 26, 28, 30, 24, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
];
/// Number of error correction blocks, indexed by [level][version - 1].
#[rustfmt::skip]
const NUM_BLOCKS: [[u8; 40]; 4] = [
    [1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4, 4, 4, 4, 6, 6, 6, 6, 7, 8, 8, 9, 9, 10, 12, 12, 12, 13, 14, 15, 16, 17, 18, 19, 19, 20, 21, 22, 24, 25],
    [1, 1, 1, 2, 2, 4, 4, 4, 5, 5, 5, 8, 9, 9, 10, 10, 11, 13, 14, 16, 17, 17, 18, 20, 21, 23, 25, 26, 28, 29, 31, 33, 35, 37, 38, 40, 43, 45, 47, 49],
    [1, 1, 2, 2, 4, 4, 6, 6, 8, 8, 8, 10, 12, 16, 12, 17, 16, 18, 21, 20, 23, 23, 25, 27, 29, 34, 34, 35, 38, 40, 43, 45, 48, 51, 53, 56, 59, 62, 65, 68],
    [1, 1, 2, 4, 4, 4, 5, 6, 8, 8, 11, 11, 16, 16, 18, 16, 19, 21, 25, 25, 25, 34, 30, 32, 35, 37, 40, 42, 45, 48, 51, 54, 57, 60, 63, 66, 70, 74, 77, 81],
];
/// Primitive polynomial x^8 + x^4 + x^3 + x^2 + 1 defining the QR Galois field.
const GF_PRIME: u32 = 0x11D;

/// Side length of the symbol in modules.
pub fn size(version: u8) -> usize {
    17 + 4 * usize::from(version)
}

/// Total codewords (data plus error correction) that fit in a symbol of this version.
pub fn num_total_codewords(version: u8) -> usize {
    let v = usize::from(version);
    let mut modules = (16 * v + 128) * v + 64;
    if v >= 2 {
        let align = v / 7 + 2;
        modules -= (25 * align - 10) * align - 55;
        if v >= 7 {
            modules -= 36;
        }
    }
    modules / 8
}

pub fn num_data_codewords(version: u8, ec: EC) -> usize {
    let (ecc_len, blocks) = block_layout(version, ec);
    num_total_codewords(version) - ecc_len * blocks
}

/// Returns (error correction codewords per block, number of blocks).
fn block_layout(version: u8, ec: EC) -> (usize, usize) {
    let v = usize::from(version) - 1;
    (ECC_PER_BLOCK[ec.index()][v].into(), NUM_BLOCKS[ec.index()][v].into())
}

fn gf_mul(a: u8, b: u8) -> u8 {
    let (mut a, mut b, mut product) = (u32::from(a), u32::from(b), 0);
    while b > 0 {
        if b & 1 == 1 {
            product ^= a;
        }
        a <<= 1;
        if a & 0x100 != 0 {
            a ^= GF_PRIME;
        }
        b >>= 1;
    }
    product as u8
}

/// Generator polynomial of the given degree, highest coefficient first.
fn rs_generator(degree: usize) -> Vec<u8> {
    let mut generator = vec![1u8];
    let mut root = 1u8;
    for _ in 0..degree {
        let mut next = vec![0u8; generator.len() + 1];
        for (i, &coef) in generator.iter().enumerate() {
            next[i] ^= coef;
            next[i + 1] ^= gf_mul(coef, root);
        }
        generator = next;
        root = gf_mul(root, 2);
    }
    generator
}

fn rs_remainder(data: &[u8], generator: &[u8]) -> Vec<u8> {
    let mut rem = vec![0u8; generator.len() - 1];
    for &b in data {
        let factor = b ^ rem[0];
        rem.remove(0);
        rem.push(0);
        for (r, &g) in rem.iter_mut().zip(&generator[1..]) {
            *r ^= gf_mul(g, factor);
        }
    }
    rem
}

/// Splits data into blocks, computes each block's error correction and interleaves them.
pub fn add_error_correction(data: &[u8], version: u8, ec: EC) -> Vec<u8> {
    let (ecc_len, blocks) = block_layout(version, ec);
    let total = num_total_codewords(version);
    let short_len = total / blocks - ecc_len;
    let num_short = blocks - total % blocks;
    let generator = rs_generator(ecc_len);

    let mut data_blocks: Vec<&[u8]> = Vec::with_capacity(blocks);
    let mut ecc_blocks = Vec::with_capacity(blocks);
    let mut offset = 0;
    for i in 0..blocks {
        let len = short_len + usize::from(i >= num_short);
        let block = &data[offset..offset + len];
        offset += len;
        data_blocks.push(block);
        ecc_blocks.push(rs_remainder(block, &generator));
    }

    let mut out = Vec::with_capacity(total);
    for i in 0..=short_len {
        out.extend(data_blocks.iter().filter_map(|block| block.get(i)));
    }
    for i in 0..ecc_len {
        out.extend(ecc_blocks.iter().map(|block| block[i]));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_world_matches_reference_ecc() {
        let data = [
            0x20, 0x5B, 0x0B, 0x78, 0xD1, 0x72, 0xDC, 0x4D, 0x43, 0x40, 0xEC, 0x11, 0xEC, 0x11,
            0xEC, 0x11,
        ];
        let out = add_error_correction(&data, 1, EC::M);
        assert_eq!(out[..16], data);
        assert_eq!(out[16..], [196, 35, 39, 119, 235, 215, 231, 226, 93, 23]);
    }

    #[test]
    fn capacities_match_spec() {
        assert_eq!(num_data_codewords(1, EC::L), 19);
        assert_eq!(num_data_codewords(5, EC::Q), 62);
        assert_eq!(num_data_codewords(40, EC::H), 1276);
    }
}
