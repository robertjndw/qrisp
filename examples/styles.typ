// The snippets from the "Styles" section of the README.
#import "common.typ": *

#gallery(
  ("default", qr-code(url, width: 3cm)),
  ("style: \"rounded\"", qr-code(url, width: 3cm, style: "rounded")),
  ("style: \"dots\"", qr-code(url, width: 3cm, style: "dots")),
  ("radius: 0.5", qr-code(url, width: 3cm, style: "rounded", radius: 0.5)),
  ("finder-style: \"circle\"", qr-code(
    url,
    width: 3cm,
    style: "rounded",
    finder-style: "circle",
    finder-fill: rgb("#239dad"),
  )),
)
