// Turns plugin output into Typst curves. Coordinates arrive in module units and are
// scaled by the module size `m` here.

// The plugin returns the module path as a Typst array expression in module units
// (see plugin/src/outline.rs for why it's text).
#let path-components(code, m) = eval(code, scope: (
  M: curve.move,
  L: curve.line,
  C: curve.cubic,
  // The path already returns to its start, so a straight close only seals it. The
  // default "smooth" mode would add a bogus curve segment.
  Z: curve.close(mode: "straight"),
  u: m,
))

#let kappa = 0.5522848

// Rounded rectangle as curve components, in module units. Clockwise adds area and
// counter-clockwise cuts a hole (nonzero rule), same as the plugin's contours, so the
// finders can go into the same curve as the modules.
#let rounded-rect(x, y, w, h, r, m, clockwise: true) = {
  let corners = (
    ((x + w, y), (1, 0), (0, 1)),
    ((x + w, y + h), (0, 1), (-1, 0)),
    ((x, y + h), (-1, 0), (0, -1)),
    ((x, y), (0, -1), (1, 0)),
  )
  if not clockwise {
    corners = corners.rev().map(((p, din, dout)) => (p, dout.map(v => -v), din.map(v => -v)))
  }
  let pt(p) = (p.at(0) * m, p.at(1) * m)
  let ends(c) = {
    let ((px, py), (ix, iy), (ox, oy)) = c
    let a = (px - ix * r, py - iy * r)
    let b = (px + ox * r, py + oy * r)
    (
      a: a,
      c1: (a.at(0) + ix * r * kappa, a.at(1) + iy * r * kappa),
      c2: (b.at(0) - ox * r * kappa, b.at(1) - oy * r * kappa),
      b: b,
    )
  }
  let out = (curve.move(pt(ends(corners.last()).b)),)
  for c in corners {
    let e = ends(c)
    out.push(curve.line(pt(e.a)))
    if r > 0 { out.push(curve.cubic(pt(e.c1), pt(e.c2), pt(e.b))) }
  }
  out.push(curve.close(mode: "straight"))
  out
}

// Corner radii (outer frame, frame hole, centre ball) in modules per finder style.
// The hole radius is one module less than the frame so the ring keeps an even width.
#let finder-radii = (
  square: (0, 0, 0),
  rounded: (2, 1, 1),
  circle: (3.5, 2.5, 1.5),
)

#let finder-components(size, style, m) = {
  let (outer, hole, ball) = finder-radii.at(style)
  let out = ()
  for (x, y) in ((0, 0), (size - 7, 0), (0, size - 7)) {
    out += rounded-rect(x, y, 7, 7, outer, m)
    out += rounded-rect(x + 1, y + 1, 5, 5, hole, m, clockwise: false)
    out += rounded-rect(x + 2, y + 2, 3, 3, ball, m)
  }
  out
}
