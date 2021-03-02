use crate::u256;

/*** operators that take decimals into account ***/
pub fn complex_mul_u128(base: u128, a: u128, b: u128) -> u128 {
    let a_u256 = u256::from(a);
    let b_u256 = u256::from(b);
    let base_u256 = u256::from(base);

    let c0 = a_u256 * b_u256;
    let c1 = c0 + (base_u256 / 2);
    (c1 / base_u256).as_u128()
}

pub fn complex_div_u128(base: u128, a: u128, b: u128) -> u128 {
    let a_u256 = u256::from(a);
    let b_u256 = u256::from(b);
    let base_u256 = u256::from(base);

    let c0 = a_u256 * base_u256;
    let c1 = c0 + (b_u256 / 2);
    (c1 / b_u256).as_u128()
}

/*** operators that does not take decimals into account ***/
pub fn simple_mul_u128(base: u128, a: u128, b: u128) -> u128 {
    let a_u256 = u256::from(a);
    let b_u256 = u256::from(b);
    let base_u256 = u256::from(base);

    (a_u256 * b_u256 / base).as_u128()
}

pub fn simple_div_u128(base: u128, a: u128, b: u128) -> u128 {
    let a_u256 = u256::from(a);
    let b_u256 = u256::from(b);
    let base_u256 = u256::from(base);

    (a_u256 * base_u256 / b_u256).as_u128()
}