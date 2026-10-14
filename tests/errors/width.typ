// expect: `width` must be a positive length or auto
#import "../../src/lib.typ": qr-code
#qr-code("x", width: -1cm)
