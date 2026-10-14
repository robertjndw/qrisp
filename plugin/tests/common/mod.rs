//! Helpers shared by the integration tests. Each test crate compiles this module on
//! its own and uses only part of it.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// Repository root, where the typst CLI has to run so `--root .` covers the package.
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

pub fn typst(args: &[&str]) -> Command {
    let mut cmd = Command::new("typst");
    cmd.args(args).current_dir(root());
    cmd
}

pub const TYPST_MISSING: &str = "the typst CLI must be on PATH";

/// Decodes a QR code from an 8-bit grayscale image.
pub fn decode_luma(luma: Vec<u8>, width: u32, height: u32) -> Result<String, String> {
    rxing::helpers::detect_in_luma(luma, width, height, Some(rxing::BarcodeFormat::QR_CODE))
        .map(|r| r.getText().to_string())
        .map_err(|e| e.to_string())
}
