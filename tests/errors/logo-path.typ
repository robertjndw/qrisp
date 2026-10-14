// expect: packages cannot read files by path
#import "../../src/lib.typ": qr-code
#qr-code("x", logo: "logo.png")
