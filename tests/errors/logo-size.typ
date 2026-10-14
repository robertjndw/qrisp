// expect: `logo-size` must be a ratio from 1% to 50% or auto
#import "../../src/lib.typ": qr-code
#qr-code("x", logo: rect(), logo-size: 0.3)
