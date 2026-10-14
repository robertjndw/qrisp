// The "Drawing it yourself" snippet from the README: every dark module coloured by
// its column.
#import "common.typ": qr-matrix

#set page(width: auto, height: auto, margin: 8mm)

#let code = qr-matrix("https://typst.app")
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
