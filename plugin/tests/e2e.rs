//! Renders every page of tests/e2e/decode.typ to PNG with the typst CLI and decodes it
//! with rxing (a Rust port of ZXing). The result has to match the text stored in the
//! page's metadata. rqrr was tried first but can't find rounded finder patterns.
//!
//! Needs `typst` on PATH and a fresh qrisp.wasm (`make wasm`).

mod common;

use std::path::{Path, PathBuf};

fn typst(args: &[&str]) -> Vec<u8> {
    let out = common::typst(args).output().expect(common::TYPST_MISSING);
    assert!(out.status.success(), "typst {args:?} failed:\n{}", String::from_utf8_lossy(&out.stderr));
    out.stdout
}

fn decode(path: &Path) -> Result<String, String> {
    // Transparent backgrounds are composited onto white like a scanner would see them.
    let img = image::open(path).map_err(|e| e.to_string())?.to_rgba8();
    let luma = image::GrayImage::from_fn(img.width(), img.height(), |x, y| {
        let p = img.get_pixel(x, y).0;
        let a = p[3] as f32 / 255.0;
        let l = 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
        image::Luma([(l * a + 255.0 * (1.0 - a)).round() as u8])
    });
    common::decode_luma(luma.into_raw(), img.width(), img.height())
}

#[test]
fn rendered_codes_decode() {
    let doc = "tests/e2e/decode.typ";
    let expected: Vec<String> =
        serde_json::from_slice(&typst(&["query", "--root", ".", doc, "metadata", "--field", "value"])).unwrap();

    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("e2e");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let pattern = dir.join("page-{0p}.png");
    typst(&["compile", "--root", ".", "--ppi", "150", doc, pattern.to_str().unwrap()]);

    let mut pages: Vec<PathBuf> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).collect();
    pages.sort();
    assert_eq!(pages.len(), expected.len(), "one page per test case");

    // Decoding dominates the run time and every page is independent.
    let cases: Vec<_> = pages.iter().zip(&expected).enumerate().collect();
    let workers = std::thread::available_parallelism().map_or(4, |n| n.get());
    let failures: Vec<String> = std::thread::scope(|s| {
        let handles: Vec<_> = cases
            .chunks(cases.len().div_ceil(workers).max(1))
            .map(|chunk| {
                s.spawn(move || {
                    chunk
                        .iter()
                        .filter_map(|&(i, (page, want))| match decode(page) {
                            Ok(got) if &got == want => None,
                            Ok(got) => Some(format!("page {}: decoded {got:?}, expected {want:?}", i + 1)),
                            Err(e) => Some(format!("page {}: {e} ({})", i + 1, page.display())),
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    assert!(failures.is_empty(), "{} of {} codes failed:\n{}", failures.len(), pages.len(), failures.join("\n"));
}
