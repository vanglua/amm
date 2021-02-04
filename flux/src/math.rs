use crate::u256;

use crate::constants::{
    TOKEN_DENOM,
};

/*** operators that take decimals into account ***/
pub fn div_u128(a: u128, b: u128) -> u128 {
    let a_u256 = u256::from(a);
    
    let token_denom_u256 = u256::from(TOKEN_DENOM);
    let c0 = a_u256 * token_denom_u256;
    let c1 = c0 + (b / 2);

    (c1 / b).as_u128()
}

pub fn mul_u128(a: u128, b: u128) -> u128 {
    let a_u256 = u256::from(a);
    let b_u256 = u256::from(b);
    let token_denom_u256 = u256::from(TOKEN_DENOM);

    let c0: u256 = a_u256 * b_u256;

    let c1 = c0 + (token_denom_u256 / 2);

    (c1 / token_denom_u256).as_u128()
}