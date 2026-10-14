#import "common.typ": gallery, qr-code

#let url = "https://github.com/robertjndw/typst-qr-code-generator"
#let logo = image("logo.svg")

#gallery(
  columns: 4,
  ("default", qr-code(url, width: 3cm)),
  ("style: \"rounded\"", qr-code(url, width: 3cm, style: "rounded")),
  ("style: \"dots\"", qr-code(url, width: 3cm, style: "dots")),
  ("logo: logo", qr-code(url, width: 3cm, style: "rounded", logo: logo)),
  ("fill + background", qr-code(url, width: 3cm, style: "rounded", fill: rgb("#1e3a5f"), background: rgb("#f5ead7"))),
  ("gradient fill", qr-code(url, width: 3cm, style: "dots", fill: gradient.linear(rgb("#7c3aed"), rgb("#db2777"), angle: 45deg))),
  ("finder-fill", qr-code(url, width: 3cm, style: "rounded", finder-style: "circle", finder-fill: rgb("#239dad"))),
  ("radius: 0.5", qr-code(url, width: 3cm, style: "rounded", radius: 0.5, logo: logo, logo-size: 18%)),
)
