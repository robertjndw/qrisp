// Shared layout for the example pictures in docs/: a row of codes, each with the
// option that makes it different printed underneath.
#import "../src/lib.typ": qr-code, qr-matrix

#let url = "https://typst.app"

#let gallery(columns: auto, ..items) = {
  set page(width: auto, height: auto, margin: 8mm)
  set text(font: "New Computer Modern", size: 9pt)
  let items = items.pos()
  grid(
    columns: if columns == auto { items.len() } else { columns },
    column-gutter: 8mm,
    row-gutter: 6mm,
    ..items.map(((label, code)) => stack(spacing: 2mm, align(center, code), align(center, raw(label)))),
  )
}
