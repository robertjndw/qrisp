// Argument checks. Anything that gets past these is valid input, so plugin errors are
// always about the data itself (too long, logo too big).

// `alt` names the extra value a parameter accepts besides `expected`, e.g. "auto".
#let fail(name, expected, got, alt: none) = panic(
  "qrisp: `" + name + "` must be " + expected + if alt != none { " or " + alt } + ", got " + repr(got),
)

#let data(value) = {
  if type(value) == str { (bytes(value), true) }
  else if type(value) == bytes { (value, false) }
  else { fail("data", "a string or bytes", value) }
}

#let choice(name, value, options, allow-auto: false, normalize: lower) = {
  if allow-auto and value == auto { return auto }
  let v = if type(value) == str { normalize(value) } else { value }
  if v not in options {
    let names = options.map(o => "\"" + o + "\"")
    if allow-auto { names.push("auto") }
    fail(name, "one of " + names.join(", ", last: " or "), value)
  }
  v
}

#let int-range(name, value, lo, hi, allow-auto: false) = {
  if allow-auto and value == auto { return auto }
  if type(value) != int or value < lo or value > hi {
    fail(name, "an integer from " + str(lo) + " to " + str(hi), value, alt: if allow-auto { "auto" })
  }
  value
}

#let number-range(name, value, lo, hi) = {
  if type(value) not in (int, float) or value < lo or value > hi {
    fail(name, "a number from " + str(lo) + " to " + str(hi), value)
  }
  float(value)
}

#let paint(name, value, allow-none: false) = {
  if allow-none and value == none { return none }
  if type(value) not in (color, gradient, tiling) {
    fail(name, "a color, gradient or tiling", value, alt: if allow-none { "none" })
  }
  value
}

// Lengths may mix pt and em, so resolving them needs context. Checking both parts is
// enough to reject zero and negative sizes without it.
#let positive-length(name, value, allow-auto: false) = {
  if allow-auto and value == auto { return auto }
  if type(value) != length or value.abs < 0pt or value.em < 0 or (value.abs == 0pt and value.em == 0) {
    fail(name, "a positive length", value, alt: if allow-auto { "auto" })
  }
  value
}

#let logo(value) = {
  if value == none { return none }
  if type(value) == str {
    panic(
      "qrisp: `logo` must be content such as image(\"" + value + "\"); packages cannot "
        + "read files by path, so load the image in your own document and pass it in",
    )
  }
  if type(value) != content { fail("logo", "content (e.g. image(\"logo.svg\")) or none", value) }
  value
}

#let ratio-range(name, value, lo, hi, allow-auto: false) = {
  if allow-auto and value == auto { return auto }
  if type(value) != ratio or value < lo or value > hi {
    fail(name, "a ratio from " + repr(lo) + " to " + repr(hi), value, alt: if allow-auto { "auto" })
  }
  value / 100%
}
