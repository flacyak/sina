//! Module placement: function patterns, data bits, masking and format information.
use crate::ecc::size;
use crate::mode::EC;

/// Dark-run-light-run sequence of a finder pattern plus its light margin.
const FINDER_LIKE: [bool; 11] =
    [true, false, true, true, true, false, true, false, false, false, false];
const PENALTY_RUN: u32 = 3;
const PENALTY_BOX: u32 = 3;
const PENALTY_FINDER_LIKE: u32 = 40;
const PENALTY_BALANCE: u32 = 10;

pub struct Matrix {
    pub size: usize,
    dark: Vec<bool>,
    /// Function pattern cells that data placement and masking leave untouched.
    reserved: Vec<bool>,
}

impl Matrix {
    pub fn build(version: u8, ec: EC, codewords: &[u8]) -> Matrix {
        let size = size(version);
        let mut m =
            Matrix { size, dark: vec![false; size * size], reserved: vec![false; size * size] };
        m.draw_function_patterns(version, ec);
        m.draw_codewords(codewords);
        let mut best = (u32::MAX, 0);
        for mask in 0..8 {
            m.apply_mask(mask);
            m.draw_format(ec, mask);
            let penalty = m.penalty();
            if penalty < best.0 {
                best = (penalty, mask);
            }
            m.apply_mask(mask);
        }
        m.apply_mask(best.1);
        m.draw_format(ec, best.1);
        m
    }

    pub fn get(&self, row: usize, col: usize) -> bool {
        self.dark[row * self.size + col]
    }

    fn set_function(&mut self, row: usize, col: usize, dark: bool) {
        let i = row * self.size + col;
        self.dark[i] = dark;
        self.reserved[i] = true;
    }

    fn draw_function_patterns(&mut self, version: u8, ec: EC) {
        for i in 0..self.size {
            self.set_function(6, i, i % 2 == 0);
            self.set_function(i, 6, i % 2 == 0);
        }
        let far = self.size as i32 - 4;
        for (row, col) in [(3, 3), (3, far), (far, 3)] {
            self.draw_centered(row, col, 4, |d| d != 2 && d != 4);
        }
        let positions = alignment_positions(version);
        let last = positions.len().saturating_sub(1);
        for (i, &row) in positions.iter().enumerate() {
            for (j, &col) in positions.iter().enumerate() {
                let on_finder = (i == 0 && (j == 0 || j == last)) || (i == last && j == 0);
                if !on_finder {
                    self.draw_centered(row as i32, col as i32, 2, |d| d != 1);
                }
            }
        }
        self.draw_format(ec, 0);
        self.draw_version(version);
    }

    /// Draws concentric rings around a center; `dark` decides the color by Chebyshev distance.
    fn draw_centered(&mut self, row: i32, col: i32, radius: i32, dark: impl Fn(i32) -> bool) {
        for dr in -radius..=radius {
            for dc in -radius..=radius {
                let (r, c) = (row + dr, col + dc);
                if (0..self.size as i32).contains(&r) && (0..self.size as i32).contains(&c) {
                    self.set_function(r as usize, c as usize, dark(dr.abs().max(dc.abs())));
                }
            }
        }
    }

    /// Writes the 15-bit format information (level, mask, BCH remainder) in both locations.
    fn draw_format(&mut self, ec: EC, mask: u8) {
        let data = (ec.format_bits() << 3) | u32::from(mask);
        let mut rem = data;
        for _ in 0..10 {
            rem = (rem << 1) ^ ((rem >> 9) * 0x537);
        }
        let bits = ((data << 10) | rem) ^ 0x5412;
        let bit = |i: usize| (bits >> i) & 1 == 1;
        let n = self.size;
        for i in 0..6 {
            self.set_function(i, 8, bit(i));
            self.set_function(8, n - 1 - i, bit(i));
        }
        self.set_function(7, 8, bit(6));
        self.set_function(8, n - 7, bit(6));
        self.set_function(8, n - 8, bit(7));
        self.set_function(8, 8, bit(7));
        self.set_function(8, 7, bit(8));
        for i in 9..15 {
            self.set_function(8, 14 - i, bit(i));
        }
        for i in 8..15 {
            self.set_function(n - 15 + i, 8, bit(i));
        }
        self.set_function(n - 8, 8, true);
    }

    /// Writes the 18-bit version information for versions 7 and up.
    fn draw_version(&mut self, version: u8) {
        if version < 7 {
            return;
        }
        let mut rem = u32::from(version);
        for _ in 0..12 {
            rem = (rem << 1) ^ ((rem >> 11) * 0x1F25);
        }
        let bits = (u32::from(version) << 12) | rem;
        for i in 0..18 {
            let (a, b) = (self.size - 11 + i % 3, i / 3);
            self.set_function(b, a, (bits >> i) & 1 == 1);
            self.set_function(a, b, (bits >> i) & 1 == 1);
        }
    }

    /// Places codeword bits in the zigzag order defined by the spec, skipping function cells.
    fn draw_codewords(&mut self, data: &[u8]) {
        let n = self.size;
        let mut bit_index = 0;
        let mut right = n - 1;
        loop {
            if right == 6 {
                right = 5;
            }
            for step in 0..n {
                for col in [right, right - 1] {
                    let upward = ((right + 1) & 2) == 0;
                    let row = if upward { n - 1 - step } else { step };
                    let i = row * n + col;
                    if !self.reserved[i] && bit_index < data.len() * 8 {
                        self.dark[i] = (data[bit_index / 8] >> (7 - bit_index % 8)) & 1 == 1;
                        bit_index += 1;
                    }
                }
            }
            if right < 3 {
                break;
            }
            right -= 2;
        }
    }

    fn apply_mask(&mut self, mask: u8) {
        for row in 0..self.size {
            for col in 0..self.size {
                let i = row * self.size + col;
                if !self.reserved[i] && mask_bit(mask, row, col) {
                    self.dark[i] = !self.dark[i];
                }
            }
        }
    }

    fn penalty(&self) -> u32 {
        let n = self.size;
        let mut score = 0;
        for i in 0..n {
            let row: Vec<bool> = (0..n).map(|c| self.get(i, c)).collect();
            let col: Vec<bool> = (0..n).map(|r| self.get(r, i)).collect();
            score += line_penalty(&row) + line_penalty(&col);
        }
        for r in 0..n - 1 {
            for c in 0..n - 1 {
                let a = self.get(r, c);
                if a == self.get(r, c + 1) && a == self.get(r + 1, c) && a == self.get(r + 1, c + 1)
                {
                    score += PENALTY_BOX;
                }
            }
        }
        // Deviation of the dark share from 50%, in 5% steps. Size is odd so the difference is never 0.
        let dark = self.dark.iter().filter(|&&d| d).count();
        let total = n * n;
        let steps = (dark * 20).abs_diff(total * 10).div_ceil(total) - 1;
        score + steps as u32 * PENALTY_BALANCE
    }
}

fn mask_bit(mask: u8, row: usize, col: usize) -> bool {
    match mask {
        0 => (row + col).is_multiple_of(2),
        1 => row.is_multiple_of(2),
        2 => col.is_multiple_of(3),
        3 => (row + col).is_multiple_of(3),
        4 => (row / 2 + col / 3).is_multiple_of(2),
        5 => (row * col) % 2 + (row * col) % 3 == 0,
        6 => ((row * col) % 2 + (row * col) % 3).is_multiple_of(2),
        _ => ((row + col) % 2 + (row * col) % 3).is_multiple_of(2),
    }
}

/// Penalises long same-color runs and finder-like sequences along one row or column.
fn line_penalty(line: &[bool]) -> u32 {
    let run_penalty = |run: u32| if run >= 5 { PENALTY_RUN + run - 5 } else { 0 };
    let mut score = 0;
    let mut run = 1;
    for i in 1..line.len() {
        if line[i] == line[i - 1] {
            run += 1;
        } else {
            score += run_penalty(run);
            run = 1;
        }
    }
    score += run_penalty(run);
    let mut reversed = FINDER_LIKE;
    reversed.reverse();
    for window in line.windows(FINDER_LIKE.len()) {
        if window == FINDER_LIKE || window == reversed {
            score += PENALTY_FINDER_LIKE;
        }
    }
    score
}

/// Center coordinates of alignment patterns along one axis.
fn alignment_positions(version: u8) -> Vec<usize> {
    if version == 1 {
        return Vec::new();
    }
    let v = usize::from(version);
    let count = v / 7 + 2;
    let step = if version == 32 { 26 } else { (v * 4 + count * 2 + 1) / (count * 2 - 2) * 2 };
    let mut positions: Vec<usize> = (0..count - 1).map(|i| size(version) - 7 - i * step).collect();
    positions.push(6);
    positions.reverse();
    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignment_positions_match_spec() {
        assert_eq!(alignment_positions(1), []);
        assert_eq!(alignment_positions(2), [6, 18]);
        assert_eq!(alignment_positions(7), [6, 22, 38]);
        assert_eq!(alignment_positions(32), [6, 34, 60, 86, 112, 138]);
        assert_eq!(alignment_positions(40), [6, 30, 58, 86, 114, 142, 170]);
    }
}
