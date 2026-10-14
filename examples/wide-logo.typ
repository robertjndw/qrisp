// A 16:9 logo drawn in Typst rather than loaded from a file. qrisp measures it and
// clears a matching 16:9 area instead of a square. Its rounded corners are
// transparent, so the code's background shows through there.
#import "common.typ": gallery, qr-code

#let accent = rgb("#239dad")

#let badge(fill) = box(width: 16em, height: 9em, radius: 2.2em, fill: fill, align(center + horizon,
  text(fill: white, size: 5.2em, weight: "bold", "qrisp")))

#let url = "https://typst.app/universe/package/qrisp"

#gallery(
  ("logo-size: auto", qr-code(url, width: 4cm, style: "rounded", logo: badge(accent))),
  ("colored background", qr-code(
    url,
    width: 4cm,
    style: "rounded",
    fill: rgb("#1e3a5f"),
    background: rgb("#f5ead7"),
    logo: badge(rgb("#1e3a5f")),
  )),
  ("logo-padding: 0", qr-code(url, width: 4cm, style: "dots", logo: badge(accent), logo-padding: 0)),
)
