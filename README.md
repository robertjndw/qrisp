# qrisp

QR codes for [Typst](https://typst.app) with square, rounded or dotted modules, an optional logo in the middle, and your own colors.

![Showcase](https://raw.githubusercontent.com/robertjndw/qrisp/main/docs/showcase.png)

```typ
#import "@local/qrisp:0.1.0": qr-code

#qr-code("https://typst.app")
```

The encoder is [qrcodegen](https://www.nayuki.io/page/qr-code-generator-library), compiled to a small WebAssembly plugin. The plugin also merges all dark modules into a single vector path, so the PDF output stays small, there are no thin gaps between modules, and `fill` can be a gradient or tiling as well as a color.

When you add a logo, the error correction level goes up to H and qrisp checks how many codewords the logo covers in each block. If that's more than the code can recover, you get a compile error instead of a code that doesn't scan. Before giving up it tries a few larger versions, since those sometimes fit the same logo.

## Installing from GitHub

qrisp isn't on Typst Universe yet. Until then you can install it as a local package:

macOS and Linux:

```sh
curl -fsSL https://raw.githubusercontent.com/robertjndw/qrisp/main/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/robertjndw/qrisp/main/install.ps1 | iex
```

Then import it from the `local` namespace:

```typ
#import "@local/qrisp:0.1.0": qr-code
```

The script copies the package to Typst's [local package directory](https://github.com/typst/packages#local-packages), or to `TYPST_PACKAGE_PATH` if that's set. For a specific tag or branch, use `sh -s -- --ref v0.1.0` (on Windows, set `$env:QRISP_REF` first). In a clone, `make install` installs your working tree.

## Usage

### Styles

```typ
#qr-code("https://typst.app", style: "rounded")
#qr-code("https://typst.app", style: "dots")
#qr-code("https://typst.app", style: "rounded", radius: 0.5) // softer rounding
```

The three big corner squares (finder patterns) follow the module style unless you set them yourself:

```typ
#qr-code(
  "https://typst.app",
  style: "rounded",
  finder-style: "circle",
  finder-fill: rgb("#239dad"),
)
```

![Styles](https://raw.githubusercontent.com/robertjndw/qrisp/main/docs/styles.png)

### Colors

`fill` and `background` take any Typst paint. Use `background: none` for a transparent code.

```typ
#qr-code("https://typst.app", fill: rgb("#1e3a5f"), background: rgb("#f5ead7"))
#qr-code("https://typst.app", fill: gradient.linear(purple, fuchsia, angle: 45deg))
#qr-code("https://typst.app", background: none)
```

![Colors](https://raw.githubusercontent.com/robertjndw/qrisp/main/docs/colors.png)

Keep the foreground darker than the background. A lot of scanners can't read light-on-dark codes.

### Logo

Pass the logo as content. A package can't read files from your project, so `logo: "logo.svg"` won't work.

```typ
#qr-code("https://typst.app", style: "rounded", logo: image("logo.svg"))
```

Any content works: images, text, shapes, or a logo you build in Typst. qrisp measures it and keeps its proportions. Content sized relative to its container, like `rect(width: 100%, height: 50%)`, is measured inside a square, so that one comes out 2:1. The modules under the logo (plus `logo-padding`, 1 module by default) are left empty.

![Generic content as logos](https://raw.githubusercontent.com/robertjndw/qrisp/main/docs/generic-content.png)

These are all plain Typst content, see [`examples/generic-content.typ`](examples/generic-content.typ).

The cleared area has the same aspect ratio as the logo, so a 16:9 logo gets a 16:9 hole instead of a square with empty bands above and below. Transparent parts of the logo show the code's background. The whole bounding box is still cleared, though, so crop away transparent margins around a logo before using it.

![Wide logo](https://raw.githubusercontent.com/robertjndw/qrisp/main/docs/wide-logo.png)

The logo in this picture is drawn in Typst, see [`examples/wide-logo.typ`](examples/wide-logo.typ).

By default the logo gets as big as the code allows while still scanning reliably. qrisp tries every centred cut-out and picks the one that holds the largest logo of that shape. Set `logo-size` for a fixed size instead. It sets the logo's longer side, relative to the code width:

```typ
#qr-code("https://typst.app", logo: image("logo.svg"), logo-size: 20%)
```

![Logo sizes](https://raw.githubusercontent.com/robertjndw/qrisp/main/docs/logo-size.png)

A fixed size that's too big gives you an error saying how much it covers. Make it smaller, or set `logo-check: false` if you've tested the code with a scanner yourself. Less padding leaves more room for the logo, and so does a larger `version`.

### Size

```typ
#qr-code("https://typst.app", width: 4cm)          // total width including the quiet zone
#qr-code("https://typst.app", module-size: 0.6mm)  // or size per module
```

For print, modules of 0.4 mm or more scan reliably. Only shrink the quiet zone (4 modules by default) if the area around the code is already empty.

### Drawing it yourself

`qr-matrix` gives you the raw module grid:

```typ
#import "@local/qrisp:0.1.0": qr-matrix

#let code = qr-matrix("https://typst.app")
// (version: 2, ecl: "Q", mask: 5, size: 25, modules: ((true, true, ...), ...))
```

For example, dots coloured by column. The inset is the quiet zone, which you have to add yourself here:

```typ
#let m = 1.2mm
#box(fill: white, inset: 4 * m, box(width: code.size * m, height: code.size * m, {
  for (y, row) in code.modules.enumerate() {
    for (x, dark) in row.enumerate() {
      if dark {
        let hue = x * 300deg / code.size
        place(dx: x * m, dy: y * m, circle(radius: m / 2, fill: color.hsl(hue, 80%, 40%)))
      }
    }
  }
}))
```

![Custom drawing](https://raw.githubusercontent.com/robertjndw/qrisp/main/docs/custom.png)

## Reference

```typ
#qr-code(
  data,                // str or bytes
  width: auto,         // total width including the quiet zone; auto uses module-size
  module-size: 0.5mm,
  style: "square",     // "square" | "rounded" | "dots"
  radius: 1,           // rounding strength 0..1 ("rounded" corners, "dots" diameter)
  finder-style: auto,  // "square" | "rounded" | "circle"; auto follows style
  fill: black,         // color, gradient or tiling
  finder-fill: auto,   // auto uses fill
  background: white,   // color, gradient, tiling or none
  logo: none,          // content, e.g. image("logo.svg")
  logo-size: auto,     // longer side, 1% to 50% of the code width; auto = as large as still scans
  logo-padding: 1,     // modules
  logo-check: true,    // refuse logos that make the code unreliable
  ecl: auto,           // "L" | "M" | "Q" | "H"
  quiet-zone: 4,       // modules
  version: auto,       // 1..40
  mask: auto,          // 0..7
)
```

With `ecl: auto` you get H if there's a logo. Otherwise it starts at M and goes as high as it can without making the code bigger.

Strings use numeric, alphanumeric or byte (UTF-8) mode, whichever is shortest. Bytes are encoded as they are.

## Development

You need Rust with the `wasm32-unknown-unknown` target, [binaryen](https://github.com/WebAssembly/binaryen) (`wasm-opt`) and the Typst CLI.

```sh
make wasm       # build plugin/ into qrisp.wasm
make test       # rebuild and run all tests
make examples   # render the pictures in docs/ from examples/
```

The Typst code is in `src/`, the Rust plugin in `plugin/`. Tests live in `plugin/tests/`: a Reed-Solomon check of the codeword mapping used by the logo check, a stress test for that check, error message tests (`tests/errors/`) and end-to-end decoding of `tests/e2e/decode.typ` with rxing.

`qrisp.wasm` is checked in because the package ships it. Run `make wasm` after changing anything in `plugin/`.

## License

MIT
