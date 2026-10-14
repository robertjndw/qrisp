// The snippets from the "Colors" section of the README. The transparent code sits
// on a checkerboard so you can see that the background really is see-through.
#import "common.typ": *

#let checker = tiling(size: (8pt, 8pt), {
  place(square(size: 4pt, fill: luma(225)))
  place(dx: 4pt, dy: 4pt, square(size: 4pt, fill: luma(225)))
})

#gallery(
  ("fill + background", qr-code(url, width: 3cm, fill: rgb("#1e3a5f"), background: rgb("#f5ead7"))),
  ("gradient fill", qr-code(url, width: 3cm, fill: gradient.linear(purple, fuchsia, angle: 45deg))),
  ("background: none", box(fill: checker, qr-code(url, width: 3cm, background: none))),
)
