//! Maps every module of a symbol back to the codeword and error correction block it
//! belongs to. qrcodegen does not expose this, so the placement and interleaving rules
//! from ISO/IEC 18004 are re-implemented here. The tests verify the mapping against
//! real symbols by reading the codewords back and checking every Reed-Solomon block.

use crate::encode::ECL_NAMES;

// Indexed by [ecl][version]; index 0 is unused so versions can index directly.
const ECC_CODEWORDS_PER_BLOCK: [[i16; 41]; 4] = [
    [-1, 7, 10, 15, 20, 26, 18, 20, 24, 30, 18, 20, 24, 26, 30, 22, 24, 28, 30, 28, 28, 28, 28, 30, 30, 26, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
    [-1, 10, 16, 26, 18, 24, 16, 18, 22, 22, 26, 30, 22, 22, 24, 24, 28, 28, 26, 26, 26, 26, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28],
    [-1, 13, 22, 18, 26, 18, 24, 18, 22, 20, 24, 28, 26, 24, 20, 30, 24, 28, 28, 26, 30, 28, 30, 30, 30, 30, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
    [-1, 17, 28, 22, 16, 22, 28, 26, 26, 24, 28, 24, 28, 22, 24, 24, 30, 28, 28, 26, 28, 30, 24, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30],
];

const NUM_ERROR_CORRECTION_BLOCKS: [[i16; 41]; 4] = [
    [-1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4, 4, 4, 4, 6, 6, 6, 6, 7, 8, 8, 9, 9, 10, 12, 12, 12, 13, 14, 15, 16, 17, 18, 19, 19, 20, 21, 22, 24, 25],
    [-1, 1, 1, 1, 2, 2, 4, 4, 4, 5, 5, 5, 8, 9, 9, 10, 10, 11, 13, 14, 16, 17, 17, 18, 20, 21, 23, 25, 26, 28, 29, 31, 33, 35, 37, 38, 40, 43, 45, 47, 49],
    [-1, 1, 1, 2, 2, 4, 4, 6, 6, 8, 8, 8, 10, 12, 16, 12, 17, 16, 18, 21, 20, 23, 23, 25, 27, 29, 34, 34, 35, 38, 40, 43, 45, 48, 51, 53, 56, 59, 62, 65, 68],
    [-1, 1, 1, 2, 4, 4, 4, 5, 6, 8, 8, 11, 11, 16, 16, 18, 16, 19, 21, 25, 25, 25, 34, 30, 32, 35, 37, 40, 42, 45, 48, 51, 54, 57, 60, 63, 66, 70, 74, 77, 81],
];

/// Kind of a module that is not part of the data area.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Function {
    None,
    /// Finder patterns, separators, format info, timing, version info and the dark
    /// module. Covering any of these breaks detection, not just error correction.
    Critical,
    /// Alignment patterns. Decoders cope with a centre one being hidden by a logo
    /// because they can fall back to the finders for the geometry.
    Alignment,
}

pub fn symbol_size(version: u8) -> i32 {
    version as i32 * 4 + 17
}

pub fn ecc_per_block(version: u8, ecl: u8) -> usize {
    ECC_CODEWORDS_PER_BLOCK[ecl as usize][version as usize] as usize
}

pub fn num_blocks(version: u8, ecl: u8) -> usize {
    NUM_ERROR_CORRECTION_BLOCKS[ecl as usize][version as usize] as usize
}

/// Codewords reserved for misdecode protection in the smallest symbols (ISO/IEC 18004
/// table 9, the "p" column). They count against the correctable budget.
fn misdecode_protection(version: u8, ecl: u8) -> usize {
    match (version, ecl) {
        (1, 0) => 3,
        (1, 1) => 2,
        (1, _) => 1,
        (2, 0) => 2,
        (3, 0) => 1,
        _ => 0,
    }
}

pub fn alignment_positions(version: u8) -> Vec<i32> {
    if version == 1 {
        return Vec::new();
    }
    let ver = version as i32;
    let size = symbol_size(version);
    let count = ver / 7 + 2;
    let step = if ver == 32 { 26 } else { (ver * 4 + count * 2 + 1) / (count * 2 - 2) * 2 };
    let mut result: Vec<i32> = (0..count - 1).map(|i| size - 7 - i * step).collect();
    result.push(6);
    result.reverse();
    result
}

pub fn function_map(version: u8) -> Vec<Function> {
    let size = symbol_size(version);
    let mut map = vec![Function::None; (size * size) as usize];
    let mut set = |x: i32, y: i32, f: Function| map[(y * size + x) as usize] = f;

    for i in 0..size {
        set(6, i, Function::Critical);
        set(i, 6, Function::Critical);
    }
    // Finder + separator + format info. The bottom-left block includes the dark module.
    for y in 0..9 {
        for x in 0..9 {
            set(x, y, Function::Critical);
        }
        for x in size - 8..size {
            set(x, y, Function::Critical);
        }
    }
    for y in size - 8..size {
        for x in 0..9 {
            set(x, y, Function::Critical);
        }
    }

    let pos = alignment_positions(version);
    let n = pos.len();
    for (i, &cy) in pos.iter().enumerate() {
        for (j, &cx) in pos.iter().enumerate() {
            let at_finder = (i == 0 && j == 0) || (i == 0 && j == n - 1) || (i == n - 1 && j == 0);
            if at_finder {
                continue;
            }
            for dy in -2..=2 {
                for dx in -2..=2 {
                    set(cx + dx, cy + dy, Function::Alignment);
                }
            }
        }
    }

    if version >= 7 {
        for a in 0..6 {
            for b in size - 11..size - 8 {
                set(a, b, Function::Critical);
                set(b, a, Function::Critical);
            }
        }
    }
    map
}

/// Indices of the modules that carry data bits, in bit order (most significant bit of
/// codeword 0 first). Remainder bits at the end that carry no codeword are left out.
pub fn placement(version: u8, functions: &[Function]) -> Vec<usize> {
    let size = symbol_size(version);
    let raw_codewords = functions.iter().filter(|f| **f == Function::None).count() / 8;
    let mut order = Vec::with_capacity(raw_codewords * 8);
    let mut right = size - 1;
    while right >= 1 {
        if right == 6 {
            right = 5;
        }
        for vert in 0..size {
            for j in 0..2 {
                let x = right - j;
                let upward = ((right + 1) & 2) == 0;
                let y = if upward { size - 1 - vert } else { vert };
                let idx = (y * size + x) as usize;
                if functions[idx] == Function::None && order.len() < raw_codewords * 8 {
                    order.push(idx);
                }
            }
        }
        right -= 2;
    }
    order
}

/// Block index for every interleaved codeword, mirroring qrcodegen's
/// `add_ecc_and_interleave`: short blocks come first and lack one data codeword.
pub fn block_of_codeword(version: u8, ecl: u8, raw_codewords: usize) -> Vec<u16> {
    let blocks = num_blocks(version, ecl);
    let ecc = ecc_per_block(version, ecl);
    let short_blocks = blocks - raw_codewords % blocks;
    let short_len = raw_codewords / blocks;
    let mut result = Vec::with_capacity(raw_codewords);
    for i in 0..=short_len {
        for j in 0..blocks {
            if i != short_len - ecc || j >= short_blocks {
                result.push(j as u16);
            }
        }
    }
    debug_assert_eq!(result.len(), raw_codewords);
    result
}

/// Rectangle of modules hidden behind a logo, centred on the symbol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Knockout {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// Logo dimensions in modules.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LogoSize {
    pub w: f32,
    pub h: f32,
}

impl LogoSize {
    /// The largest logo with this `aspect` that fits into `w` x `h` modules.
    pub fn fit(w: f32, h: f32, aspect: f32) -> Option<LogoSize> {
        if w <= 0.0 || h <= 0.0 {
            return None;
        }
        let fw = w.min(h * aspect);
        Some(LogoSize { w: fw, h: fw / aspect })
    }

    /// A logo whose longer side is `long` modules, with `aspect` = width / height.
    pub fn from_long_side(long: f32, aspect: f32) -> LogoSize {
        if aspect >= 1.0 {
            LogoSize { w: long, h: long / aspect }
        } else {
            LogoSize { w: long * aspect, h: long }
        }
    }
}

/// Smallest whole number of modules that covers `len`, with the same parity as the
/// symbol size so the knockout sits exactly on the module grid around the centre.
fn grid_len(size: i32, len: f32) -> i32 {
    let mut n = ((len - 1e-4).ceil() as i32).max(1);
    if (size - n) % 2 != 0 {
        n += 1;
    }
    n.min(size)
}

impl Knockout {
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.w && y < self.y + self.h
    }

    fn with_size(size: i32, w: i32, h: i32) -> Knockout {
        Knockout { x: (size - w) / 2, y: (size - h) / 2, w, h }
    }

    /// Centred knockout that fits `logo` plus `padding` modules on every side.
    pub fn around(size: i32, logo: LogoSize, padding: f32) -> Knockout {
        let p = 2.0 * padding.max(0.0);
        Knockout::with_size(size, grid_len(size, logo.w + p), grid_len(size, logo.h + p))
    }

    /// Shorthand for `Coverage::best_fit` when only one search is needed.
    pub fn largest(version: u8, ecl: u8, aspect: f32, padding: f32) -> Option<(Knockout, LogoSize)> {
        Coverage::new(version, ecl).best_fit(aspect, padding)
    }
}

/// What a knockout costs in a symbol of one version and level. Building this walks
/// the whole placement once, so searches that test many rectangles reuse it.
pub struct Coverage {
    ecl: u8,
    size: i32,
    critical: Vec<bool>,
    /// Codeword carried by each module, `u32::MAX` for none.
    codeword: Vec<u32>,
    block_of: Vec<u16>,
    blocks: usize,
    /// Codewords each block can lose and still decode, minus a one-codeword reserve.
    budget: usize,
}

impl Coverage {
    pub fn new(version: u8, ecl: u8) -> Coverage {
        let functions = function_map(version);
        let order = placement(version, &functions);
        let raw = order.len() / 8;
        let mut codeword = vec![u32::MAX; functions.len()];
        for (bit, &idx) in order.iter().enumerate() {
            codeword[idx] = (bit / 8) as u32;
        }
        let ecc = ecc_per_block(version, ecl);
        Coverage {
            ecl,
            size: symbol_size(version),
            critical: functions.iter().map(|f| *f == Function::Critical).collect(),
            codeword,
            block_of: block_of_codeword(version, ecl, raw),
            blocks: num_blocks(version, ecl),
            budget: ((ecc - misdecode_protection(version, ecl)) / 2).saturating_sub(1),
        }
    }

    /// Verifies that the symbol still decodes reliably with the knockout unreadable.
    /// Hidden modules act as errors at unknown positions, so each block can absorb at
    /// most half of its error correction codewords. One codeword per block is kept in
    /// reserve for print defects, glare and the logo bleeding into neighbouring modules.
    pub fn check(&self, ko: Knockout) -> Result<(), String> {
        let size = self.size;
        let mut touched = vec![false; self.block_of.len()];
        let mut per_block = vec![0usize; self.blocks];
        for y in ko.y..ko.y + ko.h {
            for x in ko.x..ko.x + ko.w {
                let idx = (y * size + x) as usize;
                if self.critical[idx] {
                    return Err(format!(
                        "the logo covers the finder, timing or format patterns of this {size}x{size} QR code; reduce logo-size or logo-padding"
                    ));
                }
                let cw = self.codeword[idx];
                if cw != u32::MAX && !touched[cw as usize] {
                    touched[cw as usize] = true;
                    per_block[self.block_of[cw as usize] as usize] += 1;
                }
            }
        }

        let budget = self.budget;
        if let Some((block, &hit)) = per_block.iter().enumerate().max_by_key(|(_, h)| **h) {
            if hit > budget {
                let level = ECL_NAMES[self.ecl as usize];
                let hint = if self.ecl < 3 { "; use ecl: \"H\", " } else { "; " };
                return Err(format!(
                    "the logo hides too much of the QR code to scan reliably (block {} loses {hit} codewords, error correction level {level} recovers at most {budget}){hint}reduce logo-size or logo-padding, or pass logo-check: false to skip this check",
                    block + 1
                ));
            }
        }
        Ok(())
    }

    /// The knockout that leaves room for the largest logo with the given `aspect`
    /// (width / height) and `padding` on every side. Returns it with the logo size.
    ///
    /// Every centred rectangle is a candidate, not just ones with the logo's aspect:
    /// whole-module rounding and the uneven spread of codewords often make a slightly
    /// different rectangle hold a bigger logo. Growing a rectangle only ever hides more
    /// codewords, so for each width the tallest rectangle that passes shrinks as the
    /// width grows. Walking that staircase visits every maximal rectangle with about
    /// one check per column.
    pub fn best_fit(&self, aspect: f32, padding: f32) -> Option<(Knockout, LogoSize)> {
        let size = self.size;
        let padding = padding.max(0.0);
        let mut best: Option<LogoSize> = None;
        let mut h = size;
        for w in (1..=size).step_by(2) {
            while h >= 1 && self.check(Knockout::with_size(size, w, h)).is_err() {
                h -= 2;
            }
            if h < 1 {
                break;
            }
            let Some(logo) = LogoSize::fit(w as f32 - 2.0 * padding, h as f32 - 2.0 * padding, aspect) else {
                continue;
            };
            if best.is_none_or(|b| logo.w > b.w + 1e-4) {
                best = Some(logo);
            }
        }
        // The tightest knockout around the winner sits inside the rectangle that
        // passed, so it passes too and clears nothing the logo doesn't need.
        best.map(|logo| (Knockout::around(size, logo, padding), logo))
    }
}

/// One-off check of a knockout; see `Coverage::check`. Only the version and level
/// matter, not the data, so callers can probe versions without encoding them.
pub fn check_knockout(version: u8, ecl: u8, ko: Knockout) -> Result<(), String> {
    Coverage::new(version, ecl).check(ko)
}
