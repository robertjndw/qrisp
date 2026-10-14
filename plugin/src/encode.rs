use qrcodegen::{Mask, QrCode, QrCodeEcc, QrSegment, Version};

/// Error correction levels in the order used by the byte protocol (0 = L ... 3 = H).
pub const ECL_NAMES: [&str; 4] = ["L", "M", "Q", "H"];

#[derive(Clone, Debug, Default)]
pub struct EncodeOptions {
    /// 0..=3, `None` picks automatically.
    pub ecl: Option<u8>,
    /// 1..=40, `None` picks the smallest version that fits.
    pub version: Option<u8>,
    /// 0..=7, `None` lets the encoder pick the mask with the lowest penalty.
    pub mask: Option<u8>,
    /// Data is UTF-8 text, so numeric/alphanumeric modes may be used.
    pub text: bool,
}

/// A finished QR symbol as a plain module grid.
#[derive(Clone, Debug)]
pub struct Symbol {
    pub version: u8,
    pub ecl: u8,
    pub mask: u8,
    pub size: i32,
    modules: Vec<bool>,
}

impl Symbol {
    /// Row-major module grid, `true` for dark.
    pub fn modules(&self) -> &[bool] {
        &self.modules
    }

    /// Dark module at (x, y). Anything outside the grid is light, which is what both
    /// the outline tracer and the quiet zone want.
    pub fn get(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.size && y < self.size && self.modules[(y * self.size + x) as usize]
    }

    /// Turns every module where `hide` is true light.
    pub fn clear_where(&mut self, hide: impl Fn(i32, i32) -> bool) {
        for y in 0..self.size {
            for x in 0..self.size {
                if hide(x, y) {
                    self.modules[(y * self.size + x) as usize] = false;
                }
            }
        }
    }
}

fn ecc_from_index(i: u8) -> QrCodeEcc {
    match i {
        0 => QrCodeEcc::Low,
        1 => QrCodeEcc::Medium,
        2 => QrCodeEcc::Quartile,
        _ => QrCodeEcc::High,
    }
}

fn ecc_to_index(e: QrCodeEcc) -> u8 {
    match e {
        QrCodeEcc::Low => 0,
        QrCodeEcc::Medium => 1,
        QrCodeEcc::Quartile => 2,
        QrCodeEcc::High => 3,
    }
}

pub fn encode(data: &[u8], opts: &EncodeOptions) -> Result<Symbol, String> {
    let segs = match (opts.text, std::str::from_utf8(data)) {
        (true, Ok(s)) => QrSegment::make_segments(s),
        _ => vec![QrSegment::make_bytes(data)],
    };

    // An automatic ECL is bumped as far as it goes without growing the symbol. An
    // explicit one is used as given.
    let (ecl, boost) = match opts.ecl {
        Some(e) if e <= 3 => (ecc_from_index(e), false),
        Some(e) => return Err(format!("invalid error correction level index {e}")),
        None => (QrCodeEcc::Medium, true),
    };

    let (min, max) = match opts.version {
        Some(v) if (1..=40).contains(&v) => (v, v),
        Some(v) => return Err(format!("version must be between 1 and 40, got {v}")),
        None => (1, 40),
    };

    let mask = match opts.mask {
        Some(m) if m <= 7 => Some(Mask::new(m)),
        Some(m) => return Err(format!("mask must be between 0 and 7, got {m}")),
        None => None,
    };

    let qr = QrCode::encode_segments_advanced(&segs, ecl, Version::new(min), Version::new(max), mask, boost)
        .map_err(|e| {
            let level = ECL_NAMES[ecc_to_index(ecl) as usize];
            match opts.version {
                Some(v) => format!("data does not fit into a version {v} QR code at error correction level {level} ({e})"),
                None => format!("data is too long for a QR code at error correction level {level} ({e})"),
            }
        })?;

    let size = qr.size();
    let mut modules = Vec::with_capacity((size * size) as usize);
    for y in 0..size {
        for x in 0..size {
            modules.push(qr.get_module(x, y));
        }
    }

    Ok(Symbol {
        version: qr.version().value(),
        ecl: ecc_to_index(qr.error_correction_level()),
        mask: qr.mask().value(),
        size,
        modules,
    })
}
