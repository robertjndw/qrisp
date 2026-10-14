//! Core of the qrisp Typst plugin. Everything here is plain Rust so it can be tested
//! natively; the WebAssembly bindings live in `wasm.rs`.
//!
//! Byte protocol shared with `src/lib.typ`:
//!
//! Options (both functions):
//!   [0] ecl index 0..=3, 255 = auto
//!   [1] version 1..=40, 0 = auto
//!   [2] mask 0..=7, 255 = auto
//!   [3] 1 = data is UTF-8 text, 0 = raw bytes
//! Additional options for `generate`:
//!   [4] style: 0 = square, 1 = rounded, 2 = dots
//!   [5] 1 = verify that the knockout still scans
//!   [6..10]  f32 LE rounding strength 0..=1
//!   [10..14] f32 LE logo size as a fraction of the symbol width, 0 = no logo,
//!            negative = as large as the check allows
//!   [14..18] f32 LE padding around the logo in modules
//!   [18..22] f32 LE logo aspect ratio (width / height)
//!
//! Every response starts with [version, ecl, mask, size]. `generate` then adds the
//! logo width and height in modules as f32 LE (both 0 without a logo) and the path
//! as a UTF-8 Typst expression (see `outline.rs`). `matrix` adds one byte per
//! module, row by row.

pub mod capacity;
pub mod encode;
pub mod outline;

#[cfg(target_arch = "wasm32")]
mod wasm;

use capacity::{Knockout, LogoSize};
use encode::{EncodeOptions, Symbol};
use outline::Style;

/// How many versions above the smallest fitting one to try when a logo does not fit.
const MAX_EXTRA_VERSIONS: u8 = 4;

fn opt_u8(v: u8, none: u8) -> Option<u8> {
    (v != none).then_some(v)
}

fn read_f32(bytes: &[u8], at: usize) -> f32 {
    f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn parse_encode_options(opts: &[u8]) -> Result<EncodeOptions, String> {
    if opts.len() < 4 {
        return Err("malformed options passed to the qrisp plugin".into());
    }
    Ok(EncodeOptions {
        ecl: opt_u8(opts[0], 255),
        version: opt_u8(opts[1], 0),
        mask: opt_u8(opts[2], 255),
        text: opts[3] == 1,
    })
}

#[derive(Clone, Copy)]
enum Logo {
    None,
    Fixed(f32),
    Auto,
}

struct LogoOptions {
    logo: Logo,
    /// Width / height of the logo, so the cleared area can hug it.
    aspect: f32,
    padding: f32,
    check: bool,
}

/// Where the logo goes in a symbol of this version and level: the cleared rectangle
/// and the logo size in modules.
fn place_logo(logo_opts: &LogoOptions, version: u8, ecl: u8) -> Result<Option<(Knockout, LogoSize)>, String> {
    let size = capacity::symbol_size(version);
    match logo_opts.logo {
        Logo::None => Ok(None),
        Logo::Fixed(fraction) => {
            let logo = LogoSize::from_long_side(fraction * size as f32, logo_opts.aspect);
            let ko = Knockout::around(size, logo, logo_opts.padding);
            if logo_opts.check {
                capacity::check_knockout(version, ecl, ko)?;
            }
            Ok(Some((ko, logo)))
        }
        // Sized by the check even with logo-check off; without it there is no limit.
        Logo::Auto => Knockout::largest(version, ecl, logo_opts.aspect, logo_opts.padding).map(Some).ok_or_else(|| {
            format!("there is no room for a logo in this {size}x{size} QR code; reduce logo-padding or use a larger version")
        }),
    }
}

fn header(sym: &Symbol) -> [u8; 4] {
    [sym.version, sym.ecl, sym.mask, sym.size as u8]
}

/// Whether (x, y) belongs to one of the three 7x7 finder patterns, which the Typst
/// side draws itself so they can be styled independently.
fn in_finder(size: i32, x: i32, y: i32) -> bool {
    let near = |v: i32| v < 7;
    let far = |v: i32| v >= size - 7;
    (near(x) && near(y)) || (far(x) && near(y)) || (near(x) && far(y))
}

pub fn generate(data: &[u8], opts: &[u8]) -> Result<Vec<u8>, String> {
    if opts.len() < 22 {
        return Err("malformed options passed to the qrisp plugin".into());
    }
    let style = match opts[4] {
        0 => Style::Square,
        1 => Style::Rounded,
        2 => Style::Dots,
        s => return Err(format!("unknown style index {s}")),
    };
    let radius = read_f32(opts, 6);
    let aspect = read_f32(opts, 18);
    let logo_opts = LogoOptions {
        logo: match read_f32(opts, 10) {
            f if f < 0.0 => Logo::Auto,
            f if f > 0.0 => Logo::Fixed(f),
            _ => Logo::None,
        },
        aspect: if aspect.is_finite() && aspect > 0.0 { aspect } else { 1.0 },
        padding: read_f32(opts, 14),
        check: opts[5] == 1,
    };

    let mut encode_opts = parse_encode_options(opts)?;
    // A logo hides modules, so an automatic level means High.
    if !matches!(logo_opts.logo, Logo::None) && encode_opts.ecl.is_none() {
        encode_opts.ecl = Some(3);
    }
    let mut sym = encode::encode(data, &encode_opts)?;
    let placed = match place_logo(&logo_opts, sym.version, sym.ecl) {
        Ok(placed) => placed,
        Err(first) if encode_opts.version.is_some() => return Err(first),
        Err(first) => {
            // How much of each block the logo hides depends on the block layout, which
            // changes between versions. A slightly bigger symbol often takes a logo the
            // smallest one can't, so try a few before giving up. The level is fixed
            // (High, or what the caller asked for), so only the winner gets encoded.
            let (version, placed) = (sym.version + 1..=(sym.version + MAX_EXTRA_VERSIONS).min(40))
                .find_map(|v| Some((v, place_logo(&logo_opts, v, sym.ecl).ok()?)))
                .ok_or(first)?;
            encode_opts.version = Some(version);
            sym = encode::encode(data, &encode_opts)?;
            placed
        }
    };
    let knockout = placed.map(|(ko, _)| ko);
    let logo = placed.map_or(LogoSize { w: 0.0, h: 0.0 }, |(_, logo)| logo);

    // The tracer asks about every cell several times, so resolve the rules once.
    let size = sym.size;
    sym.clear_where(|x, y| in_finder(size, x, y) || knockout.is_some_and(|k| k.contains(x, y)));
    let path = outline::build(size, style, radius, |x, y| sym.get(x, y));

    let mut out = Vec::with_capacity(12 + path.len());
    out.extend(header(&sym));
    out.extend(logo.w.to_le_bytes());
    out.extend(logo.h.to_le_bytes());
    out.extend_from_slice(&path);
    Ok(out)
}

pub fn matrix(data: &[u8], opts: &[u8]) -> Result<Vec<u8>, String> {
    let sym = encode::encode(data, &parse_encode_options(opts)?)?;
    let mut out = header(&sym).to_vec();
    out.extend(sym.modules().iter().map(|&m| m as u8));
    Ok(out)
}
