// expect: does not fit into a version 1 QR code
#import "../../src/lib.typ": qr-code
#qr-code("x" * 100, version: 1)
