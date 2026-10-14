// One code per page, with the expected text as metadata. plugin/tests/e2e.rs
// renders each page and checks that it decodes to that text.
#import "../../src/lib.typ": qr-code, qr-matrix

#set page(width: auto, height: auto, margin: 0pt)

#let url = "https://typst.app/universe/package/qrisp"
#let logo = rect(width: 100%, height: 100%, fill: red, radius: 20%)

#let case(data, ..args) = page({
  metadata(data)
  qr-code(data, module-size: 1mm, ..args)
})

#for style in ("square", "rounded", "dots") {
  for finder-style in (auto, "square", "rounded", "circle") {
    case(url, style: style, finder-style: finder-style)
    case(url, style: style, finder-style: finder-style, logo: logo)
  }
}

#for radius in (0, 0.3, 0.7) {
  case(url, style: "rounded", radius: radius)
  case(url, style: "dots", radius: radius)
}

#for size in (10%, 20%, 25%) {
  case(url, style: "rounded", logo: logo, logo-size: size)
}
#case(url, logo: logo, logo-padding: 0)
#case(url, logo: logo, logo-padding: 3, logo-size: 15%)

#for ecl in ("L", "M", "Q", "H") { case(url, ecl: ecl) }
#for v in (1, 2, 7, 10) { case("12345", version: v, style: "rounded") }
#case("A" * 2000, style: "rounded", ecl: "L")
#case("Größenwahn - 日本語 - emoji 🎉", style: "rounded")
#case("0123456789" * 30)
#case("HELLO WORLD $%*+-./:")
#case(url, fill: gradient.linear(blue, purple, angle: 45deg), style: "rounded")
#case(url, fill: rgb("#1e3a5f"), background: rgb("#f5ead7"), style: "dots")
#case(url, background: none, style: "rounded")
#case(url, finder-fill: orange, style: "rounded", finder-style: "circle")
#case(url, quiet-zone: 2)
#for m in range(8) { case("mask", mask: m) }

// logo-size: auto (the default) at different sizes and paddings
#for v in (5, 8, 12, 20) { case(url, version: v, style: "rounded", logo: logo) }
#case(url, logo: logo, logo-padding: 0)
#case(url, logo: logo, logo-padding: 2.5, style: "dots")
#case("0123456789" * 30, logo: logo, style: "rounded")

// Logos that aren't square get a cut-out with the same aspect ratio.
#let wide = box(width: 16em, height: 9em, radius: 2em, fill: red)
#let tall = box(width: 9em, height: 16em, radius: 2em, fill: red)
#for style in ("square", "rounded", "dots") {
  case(url, style: style, logo: wide)
  case(url, style: style, logo: tall)
}
#case(url, logo: wide, logo-size: 30%)
#case(url, logo: box(width: 40em, height: 9em, fill: red), logo-padding: 0)
#case(url, version: 12, logo: wide)

// Generic content as a logo: the cut-out follows whatever shape it measures as.
#for logo in (
  text(size: 20pt, weight: "bold", fill: red, "ACME"),
  rect(width: 100%, height: 50%, fill: red),
  rect(width: 100%, height: 1cm, fill: red),
  circle(radius: 1cm, fill: red),
  stack(dir: ltr, spacing: 2pt, circle(radius: 6pt, fill: red), text(14pt, fill: red)[Typst]),
  box(width: 10cm, height: 1cm, fill: red),
  box(width: 1cm, height: 10cm, fill: red),
  place(circle(radius: 5pt, fill: red)),
) {
  case(url, style: "rounded", logo: logo)
}

// qr-matrix drawn by hand, as in the README's "Drawing it yourself" example.
#page({
  metadata(url)
  let code = qr-matrix(url)
  let m = 1mm
  box(fill: white, inset: 4 * m, box(width: code.size * m, height: code.size * m, {
    for (y, row) in code.modules.enumerate() {
      for (x, dark) in row.enumerate() {
        if dark {
          let hue = x * 300deg / code.size
          place(dx: x * m, dy: y * m, circle(radius: m / 2, fill: color.hsl(hue, 80%, 40%)))
        }
      }
    }
  }))
})
