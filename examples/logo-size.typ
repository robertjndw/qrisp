// logo-size: auto (the default) next to fixed sizes, from the "Logo" section.
#import "common.typ": *

#let logo = image("logo.svg")

#gallery(
  ("logo-size: auto", qr-code(url, width: 3cm, style: "rounded", logo: logo)),
  ("logo-size: 20%", qr-code(url, width: 3cm, style: "rounded", logo: logo, logo-size: 20%)),
  ("logo-size: 12%", qr-code(url, width: 3cm, style: "rounded", logo: logo, logo-size: 12%)),
  ("logo-padding: 0", qr-code(url, width: 3cm, style: "rounded", logo: logo, logo-padding: 0)),
)
