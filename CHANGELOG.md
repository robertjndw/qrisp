# Changelog

## 0.1.0

Initial release.

- QR encoding through a Rust WebAssembly plugin (versions 1-40, all error correction levels, automatic mode and mask)
- Module styles `square`, `rounded` and `dots`
- Finder styles `square`, `rounded` and `circle`, optionally in their own color
- Colors, gradients or tilings for foreground and background
- Logo in the center, with error correction H and a check that rejects logos too big to scan
- `logo-size: auto` (the default) makes the logo as large as the code can take
- Any content works as a logo and keeps its proportions; the cut-out follows its aspect ratio and is the largest that still scans
- `qr-matrix` to get the raw module grid
