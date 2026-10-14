// Logos don't have to be image files. Any content works, and the cleared area
// follows whatever shape it measures as.
#import "common.typ": gallery, qr-code

#let accent = rgb("#239dad")
#let url = "https://typst.app/universe/package/qrisp"

#let item(label, logo) = (label, qr-code(url, width: 3.4cm, style: "rounded", logo: logo))

#gallery(
  columns: 4,
  item("text", text(size: 20pt, weight: "bold", fill: accent, "ACME")),
  item("math", text(fill: accent, $e^(i pi) + 1 = 0$)),
  item("emoji", text(size: 24pt, emoji.rocket)),
  item("circle", circle(radius: 1cm, fill: gradient.radial(accent.lighten(40%), accent))),
  item("stack", stack(
    dir: ltr,
    spacing: 3pt,
    box(width: 14pt, height: 14pt, radius: 4pt, fill: accent),
    text(size: 14pt, weight: "bold")[Typst],
  )),
  item("rect(height: 50%)", rect(width: 100%, height: 50%, radius: 15%, fill: accent)),
  item("two lines", align(center, text(fill: accent, weight: "bold")[
    #text(size: 16pt)[Summer]\
    #text(size: 9pt)[Fest 2026]
  ])),
  item("rotate", rotate(-12deg, reflow: true, box(
    inset: 4pt,
    radius: 3pt,
    stroke: 1.5pt + red,
    text(fill: red, weight: "bold", size: 12pt, "SAMPLE"),
  ))),
)
