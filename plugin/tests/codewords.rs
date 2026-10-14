//! Checks the placement and interleaving code in `capacity.rs` against real symbols
//! from qrcodegen: read the codewords back out of the matrix, split them into blocks
//! and check the Reed-Solomon parity of each. A wrong table entry or placement rule
//! makes the parity mismatch.

use qrisp_plugin::capacity::{block_of_codeword, ecc_per_block, function_map, num_blocks, placement};
use qrisp_plugin::encode::{encode, EncodeOptions};

fn gf_mul(mut a: u8, mut b: u8) -> u8 {
    let mut r = 0u8;
    while b != 0 {
        if b & 1 != 0 {
            r ^= a;
        }
        let carry = a & 0x80 != 0;
        a <<= 1;
        if carry {
            a ^= 0x1D;
        }
        b >>= 1;
    }
    r
}

fn rs_divisor(degree: usize) -> Vec<u8> {
    let mut result = vec![0u8; degree - 1];
    result.push(1);
    let mut root = 1u8;
    for _ in 0..degree {
        for j in 0..degree {
            result[j] = gf_mul(result[j], root);
            if j + 1 < degree {
                result[j] ^= result[j + 1];
            }
        }
        root = gf_mul(root, 0x02);
    }
    result
}

fn rs_remainder(data: &[u8], divisor: &[u8]) -> Vec<u8> {
    let mut result = vec![0u8; divisor.len()];
    for &b in data {
        let factor = b ^ result.remove(0);
        result.push(0);
        for (x, &y) in result.iter_mut().zip(divisor) {
            *x ^= gf_mul(y, factor);
        }
    }
    result
}

fn masked(mask: u8, x: i32, y: i32) -> bool {
    match mask {
        0 => (x + y) % 2 == 0,
        1 => y % 2 == 0,
        2 => x % 3 == 0,
        3 => (x + y) % 3 == 0,
        4 => (x / 3 + y / 2) % 2 == 0,
        5 => x * y % 2 + x * y % 3 == 0,
        6 => (x * y % 2 + x * y % 3) % 2 == 0,
        _ => ((x + y) % 2 + x * y % 3) % 2 == 0,
    }
}

#[test]
fn every_block_is_a_valid_reed_solomon_codeword() {
    for version in 1..=40u8 {
        for ecl in 0..4u8 {
            let opts = EncodeOptions { ecl: Some(ecl), version: Some(version), text: false, ..Default::default() };
            let sym = encode(b"qrisp", &opts).unwrap();
            assert_eq!(sym.ecl, ecl);

            let functions = function_map(version);
            let order = placement(version, &functions);
            let size = sym.size;
            let mut codewords = vec![0u8; order.len() / 8];
            for (bit, &idx) in order.iter().enumerate() {
                let (x, y) = (idx as i32 % size, idx as i32 / size);
                if sym.get(x, y) ^ masked(sym.mask, x, y) {
                    codewords[bit / 8] |= 0x80 >> (bit % 8);
                }
            }

            let owner = block_of_codeword(version, ecl, codewords.len());
            let ecc = ecc_per_block(version, ecl);
            let divisor = rs_divisor(ecc);
            for block in 0..num_blocks(version, ecl) {
                let bytes: Vec<u8> =
                    codewords.iter().zip(&owner).filter(|(_, &b)| b as usize == block).map(|(c, _)| *c).collect();
                let (data, parity) = bytes.split_at(bytes.len() - ecc);
                assert_eq!(rs_remainder(data, &divisor), parity, "version {version}, ecl {ecl}, block {block}");
            }
        }
    }
}
