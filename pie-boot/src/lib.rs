#![no_std]

#[inline(never)]
#[unsafe(no_mangle)]
pub extern "C" fn add3333(left: u64, right: u64) -> u64 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
