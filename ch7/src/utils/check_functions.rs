#[allow(dead_code)]
pub fn parity_check(bytes: &[u8]) -> u8 {
    let mut n_ones: u32 = 0;

    for byte in bytes {
        let ones = byte.count_ones();
        n_ones += ones;
    }

    (n_ones % 2 == 0) as u8
}

#[cfg(test)]
pub mod tests {
    use crate::utils::check_functions::parity_check;

    #[test]
    pub fn test_parity_check() {
        let abc = b"abc";
        assert_eq!(parity_check(abc), 1);

        let abcd = b"abcd";
        assert_eq!(parity_check(abcd), 0);
    }
}
