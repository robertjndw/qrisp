//! The logo check must never accept a knockout that breaks decoding. For each version
//! and level, take the largest logo size the check accepts, invert every hidden
//! module (worse than any real logo) and make sure rxing still decodes it.

mod common;

use qrisp_plugin::capacity::{check_knockout, symbol_size, Coverage, Knockout, LogoSize};
use qrisp_plugin::encode::{encode, EncodeOptions};

const SCALE: u32 = 4;
const QUIET: i32 = 4;

fn decode_with_knockout(sym: &qrisp_plugin::encode::Symbol, ko: Knockout) -> Result<String, String> {
    let total = (sym.size + 2 * QUIET) as u32;
    let px = total * SCALE;
    let mut luma = vec![255u8; (px * px) as usize];
    for y in 0..sym.size {
        for x in 0..sym.size {
            if sym.get(x, y) ^ ko.contains(x, y) {
                for dy in 0..SCALE {
                    for dx in 0..SCALE {
                        let (gx, gy) = ((x + QUIET) as u32 * SCALE + dx, (y + QUIET) as u32 * SCALE + dy);
                        luma[(gy * px + gx) as usize] = 0;
                    }
                }
            }
        }
    }
    common::decode_luma(luma, px, px)
}

#[test]
fn largest_accepted_knockout_still_decodes() {
    let data = "https://typst.app/universe/package/qrisp";
    let mut failures = Vec::new();
    let mut tested = 0;
    for ecl in [2u8, 3] {
        for version in 3..=40u8 {
            let opts = EncodeOptions { ecl: Some(ecl), version: Some(version), text: true, ..Default::default() };
            let Ok(sym) = encode(data.as_bytes(), &opts) else { continue };
            // Square, 16:9 and portrait logos, sized the way `logo-size: auto` does it.
            for aspect in ASPECTS {
                let Some((ko, _)) = Knockout::largest(sym.version, sym.ecl, aspect, 1.0) else { continue };
                tested += 1;
                match decode_with_knockout(&sym, ko) {
                    Ok(text) if text == data => {}
                    other => failures.push(format!("version {version} ecl {ecl} aspect {aspect} {ko:?}: {other:?}")),
                }
            }
        }
    }
    assert!(tested >= 480, "only {tested} symbols had an acceptable knockout");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

const ASPECTS: [f32; 8] = [0.1, 0.5, 0.75, 1.0, 4.0 / 3.0, 16.0 / 9.0, 4.0, 10.0];

/// Brute force over every centred rectangle: the search has to find a logo as large as
/// the best one any passing rectangle can hold.
#[test]
fn best_fit_matches_brute_force() {
    for version in (1..=12u8).chain([15, 20, 27, 33, 40]) {
        let size = symbol_size(version);
        for ecl in [1u8, 3] {
            let cov = Coverage::new(version, ecl);
            for padding in [0.0f32, 1.0] {
                for aspect in ASPECTS {
                    let mut brute: Option<f32> = None;
                    for w in (1..=size).step_by(2) {
                        for h in (1..=size).step_by(2) {
                            let ko = Knockout { x: (size - w) / 2, y: (size - h) / 2, w, h };
                            if cov.check(ko).is_err() {
                                continue;
                            }
                            if let Some(logo) = LogoSize::fit(w as f32 - 2.0 * padding, h as f32 - 2.0 * padding, aspect) {
                                brute = Some(brute.map_or(logo.w, |b| b.max(logo.w)));
                            }
                        }
                    }
                    let found = cov.best_fit(aspect, padding);
                    let ctx = format!("version {version} ecl {ecl} padding {padding} aspect {aspect}");
                    match (brute, found) {
                        (None, None) => {}
                        (Some(b), Some((ko, logo))) => {
                            assert!((logo.w - b).abs() < 1e-3, "{ctx}: found {} but {b} fits", logo.w);
                            assert!((logo.w / logo.h - aspect).abs() < 1e-3 * aspect, "{ctx}: aspect changed");
                            assert!(cov.check(ko).is_ok(), "{ctx}: returned knockout fails the check");
                            assert!(ko.w as f32 >= logo.w + 2.0 * padding - 1e-3, "{ctx}: logo wider than knockout");
                            assert!(ko.h as f32 >= logo.h + 2.0 * padding - 1e-3, "{ctx}: logo taller than knockout");
                            assert_eq!((ko.x * 2 + ko.w, ko.y * 2 + ko.h), (size, size), "{ctx}: not centred");
                        }
                        other => panic!("{ctx}: brute force and search disagree: {other:?}"),
                    }
                }
            }
        }
    }
}

#[test]
fn oversized_knockout_is_rejected() {
    let opts = EncodeOptions { ecl: Some(3), version: Some(5), text: true, ..Default::default() };
    let sym = encode(b"hello", &opts).unwrap();
    let ko = Knockout::around(sym.size, LogoSize::from_long_side(0.45 * sym.size as f32, 1.0), 1.0);
    let err = check_knockout(sym.version, sym.ecl, ko).unwrap_err();
    assert!(err.contains("hides too much"), "{err}");
}

#[test]
fn knockout_is_centred_on_the_module_grid() {
    for size in (21..=177).step_by(4) {
        for f in [0.1f32, 0.2, 0.3] {
            for aspect in [1.0f32, 16.0 / 9.0, 0.5] {
                let logo = LogoSize::from_long_side(f * size as f32, aspect);
                let ko = Knockout::around(size, logo, 1.0);
                assert_eq!(ko.x * 2 + ko.w, size);
                assert_eq!(ko.y * 2 + ko.h, size);
                assert!(ko.w as f32 >= logo.w + 2.0 - 1e-3 && ko.h as f32 >= logo.h + 2.0 - 1e-3);
            }
        }
    }
}
