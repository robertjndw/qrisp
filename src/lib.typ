#import "validate.typ"
#import "render.typ": path-components, finder-components

#let _plugin = plugin("../qrisp.wasm")

#let _ecl-names = ("L", "M", "Q", "H")
#let _styles = ("square", "rounded", "dots")

// Option bytes shared by both plugin functions; layout documented in plugin/src/lib.rs.
#let _encode-options(text, ecl, version, mask) = bytes((
  if ecl == auto { 255 } else { _ecl-names.position(e => e == ecl) },
  if version == auto { 0 } else { version },
  if mask == auto { 255 } else { mask },
  if text { 1 } else { 0 },
))

// Validates the arguments both public functions share and packs them for the plugin.
#let _encode-args(data, ecl, version, mask) = {
  let (payload, text) = validate.data(data)
  let ecl = validate.choice("ecl", ecl, _ecl-names, allow-auto: true, normalize: upper)
  let version = validate.int-range("version", version, 1, 40, allow-auto: true)
  let mask = validate.int-range("mask", mask, 0, 7, allow-auto: true)
  (payload, _encode-options(text, ecl, version, mask))
}

#let _f32(x) = float(x).to-bytes(endian: "little", size: 4)

/// Encodes `data` and returns the raw module grid, for drawing a code yourself.
///
/// Returns a dictionary with `version`, `ecl` ("L"/"M"/"Q"/"H"), `mask`, `size` and
/// `modules`, an array of rows where `true` marks a dark module.
#let qr-matrix(data, ecl: auto, version: auto, mask: auto) = {
  let (payload, encode-opts) = _encode-args(data, ecl, version, mask)
  let out = _plugin.matrix(payload, encode-opts)
  let size = out.at(3)
  let cells = array(out.slice(4))
  (
    version: out.at(0),
    ecl: _ecl-names.at(out.at(1)),
    mask: out.at(2),
    size: size,
    modules: range(size).map(y => cells.slice(y * size, (y + 1) * size).map(c => c == 1)),
  )
}

/// Draws a QR code.
///
/// - data (str, bytes): Content to encode. Strings use the most compact mode
///   (numeric, alphanumeric or UTF-8 bytes); bytes are encoded as-is.
/// - width (length, auto): Total width including the quiet zone. With `auto` the size
///   follows from `module-size`.
/// - module-size (length): Size of one module when `width` is `auto`.
/// - style (str): Module shape: `"square"`, `"rounded"` (neighbouring modules merge
///   into smooth shapes) or `"dots"`.
/// - radius (int, float): Rounding strength from 0 to 1. For `"rounded"` it scales the
///   corner radius up to half a module; for `"dots"` the dot diameter.
/// - finder-style (str, auto): Shape of the three position markers: `"square"`,
///   `"rounded"` or `"circle"`. `auto` matches `style`.
/// - fill (color, gradient, tiling): Foreground paint.
/// - finder-fill (color, gradient, tiling, auto): Paint for the position markers.
///   `auto` uses `fill`.
/// - background (color, gradient, tiling, none): Background including the quiet zone.
///   `none` is transparent.
/// - logo (content, none): Placed in the centre: an image, text, shapes or any
///   other content. It keeps its proportions, and the modules behind it are cleared.
/// - logo-size (ratio, auto): Size of the logo's longer side relative to the code
///   (without quiet zone). The cleared area follows the logo's aspect ratio.
///   `auto` makes it as large as the code can take and still scan reliably.
/// - logo-padding (int, float): Cleared margin around the logo, in modules.
/// - logo-check (bool): Refuse logos that would make the code unreliable to scan.
/// - ecl (str, auto): Error correction level `"L"`, `"M"`, `"Q"` or `"H"`. `auto`
///   uses `"H"` with a logo and otherwise the highest level that fits the same size
///   as `"M"`.
/// - quiet-zone (int): Light border in modules. The standard asks for 4.
/// - version (int, auto): Symbol version 1-40 (size 21 to 177 modules). `auto` picks
///   the smallest version that fits.
/// - mask (int, auto): Mask pattern 0-7. `auto` picks the best one.
/// -> content
#let qr-code(
  data,
  width: auto,
  module-size: 0.5mm,
  style: "square",
  radius: 1,
  finder-style: auto,
  fill: black,
  finder-fill: auto,
  background: white,
  logo: none,
  logo-size: auto,
  logo-padding: 1,
  logo-check: true,
  ecl: auto,
  quiet-zone: 4,
  version: auto,
  mask: auto,
) = {
  let (payload, encode-opts) = _encode-args(data, ecl, version, mask)
  let width = validate.positive-length("width", width, allow-auto: true)
  let module-size = validate.positive-length("module-size", module-size)
  let style = validate.choice("style", style, _styles)
  let radius = validate.number-range("radius", radius, 0, 1)
  let finder-style = validate.choice(
    "finder-style",
    finder-style,
    ("square", "rounded", "circle"),
    allow-auto: true,
  )
  let fill = validate.paint("fill", fill)
  let finder-fill = if finder-fill == auto { fill } else { validate.paint("finder-fill", finder-fill) }
  let background = validate.paint("background", background, allow-none: true)
  let logo = validate.logo(logo)
  let logo-size = validate.ratio-range("logo-size", logo-size, 1%, 50%, allow-auto: true)
  // A negative size asks the plugin for the largest logo that still scans.
  let logo-fraction = if logo == none { 0 } else if logo-size == auto { -1 } else { logo-size }
  let logo-padding = validate.number-range("logo-padding", logo-padding, 0, 10)
  assert(type(logo-check) == bool, message: "qrisp: `logo-check` must be a boolean")
  let quiet-zone = validate.int-range("quiet-zone", quiet-zone, 0, 40)

  if finder-style == auto {
    finder-style = (square: "square", rounded: "rounded", dots: "circle").at(style)
  }

  // `shape` is the size the logo is laid out at before scaling, or none if it has
  // no measurable size. `region` is the area it was measured in, if any.
  let render(shape, region: none) = {
    let aspect = if shape == none { 1 } else { shape.width / shape.height }
    let opts = (
      encode-opts
        + bytes((_styles.position(s => s == style), if logo-check { 1 } else { 0 }))
        + _f32(radius)
        + _f32(logo-fraction)
        + _f32(logo-padding)
        + _f32(aspect)
    )
    let out = _plugin.generate(payload, opts)
    let size = out.at(3)

    let total = size + 2 * quiet-zone
    let m = if width == auto { module-size } else { width / total }
    let side = total * m

    let logo-w = float.from-bytes(out.slice(4, 8), endian: "little") * m
    let logo-h = float.from-bytes(out.slice(8, 12), endian: "little") * m
    let modules = path-components(str(out.slice(12)), m)
    let finders = finder-components(size, finder-style, m)
    let symbol = if finder-fill == fill {
      curve(fill: fill, stroke: none, ..modules, ..finders)
    } else {
      curve(fill: fill, stroke: none, ..modules)
      place(top + left, curve(fill: finder-fill, stroke: none, ..finders))
    }

    box(width: side, height: side, fill: background, {
      // The box gives the separately filled finder curve the same origin as the modules.
      place(dx: quiet-zone * m, dy: quiet-zone * m, box(width: size * m, height: size * m, symbol))
      if logo != none {
        let fitted = if shape == none {
          set image(width: 100%, height: 100%, fit: "contain")
          logo
        } else {
          // Lay the logo out at its measured size first, so relative and fixed parts
          // keep their proportions, then scale it as one piece.
          let factor = calc.min(logo-w / shape.width, logo-h / shape.height)
          // Relative sizes have to resolve against the same region as when measuring,
          // or `height: 50%` would shrink a second time inside the measured box.
          let laid-out = if region == none { logo } else {
            place(top + left, box(width: region.width, height: region.height, logo))
          }
          scale(factor * 100%, reflow: true, box(width: shape.width, height: shape.height, laid-out))
        }
        place(center + horizon, box(width: logo-w, height: logo-h, align(center + horizon, fitted)))
      }
    })
  }

  if logo == none {
    render(none)
  } else {
    // The cleared area follows the logo's shape, so a wide wordmark doesn't get a
    // square hole with empty bands above and below.
    context {
      let valid(s) = s.width > 0pt and s.height > 0pt
      // Images, text and fixed-size shapes have a natural size. Content sized
      // relative to its container (e.g. `rect(width: 100%, height: 50%)`) measures
      // as zero there, so give it a square to resolve against and read its shape
      // from that. Anything still empty, like bare `place`d content, is drawn square.
      let natural = measure(logo)
      if valid(natural) { render(natural) } else {
        let region = (width: 100pt, height: 100pt)
        let probe = measure(logo, ..region)
        if valid(probe) { render(probe, region: region) } else { render(none) }
      }
    }
  }
}
