// expect: data is too long for a QR code
#import "../../src/lib.typ": qr-code
#qr-code("x" * 8000)
