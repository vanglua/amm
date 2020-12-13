use crate::constants::{
    TOKEN_DENOM,
    MIN_POW_BASE,
    MAX_POW_BASE,
    POW_PRECISION
};
use crate::u256;

/***
token_amt = token_out * 10 ** 18
for poolbalances {
    if not my pool balance {
        bal_i = outcome_token[i].balance
        token_amt = token_amt * bal_i / (bal_i + (inventment_amt - fee)))
    }

    return token_out_b + investment_amt - (token_amt / 10 ** 18)
}
***/

/**********************************************************************************************
// calcOutGivenIn                                                                            //
// aO = tokenAmountOut                                                                       //
// bO = tokenBalanceOut                                                                      //
// bI = tokenBalanceIn              /      /            bI             \    (wI / wO) \      //
// aI = tokenAmountIn    aO = bO * |  1 - | --------------------------  | ^            |     //
// wI = tokenWeightIn               \      \ ( bI + ( aI * ( 1 - sF )) /              /      //
// wO = tokenWeightOut                                                                       //
// sF = swapFee                                                                              //
**********************************************************************************************/

pub fn calc_out_given_in(
    token_balance_in: u128,
    token_weight_in: u128,
    token_balance_out: u128,
    token_weight_out: u128,
    token_amount_in: u128,
    swap_fee: u128
) -> u128 {
    let weight_ratio = div_u128(token_weight_in, token_weight_out);
    let mut adjusted_in = TOKEN_DENOM - swap_fee;
    adjusted_in = mul_u128(token_amount_in, adjusted_in);
    let y = div_u128(token_balance_in, token_balance_in + adjusted_in);
    let pow_res = pow_u128(y, weight_ratio);
    let balance_out_ratio = TOKEN_DENOM - pow_res;
    
    
    mul_u128(token_balance_out, balance_out_ratio)
}

/*** Internal math helper functions ***/
fn btoi(a: u128) -> u128 {
    a / TOKEN_DENOM
}

fn floor_u128(a: u128) -> u128 {
    btoi(a) * TOKEN_DENOM
}

fn sub_sign(a: u128, b: u128) -> (u128, bool) {
    if a >= b {
        (a - b, false)
    } else {
        (b- a, true)
    }
}

fn pow_i_u128(mut a: u128, mut n: u128) -> u128 {
    let mut z = if n % 2 == 0 { TOKEN_DENOM } else { a };
    n /= 2;
    while n != 0 {
        a = mul_u128(a, a);
        if n % 2 != 0 {
            z = mul_u128(z, a);
        }
        n /= 2;
    }
    z
}

fn pow_approx (base: u128, exp: u128, precision: u128) -> u128 {
    let a = exp;
    let (x, xneg) = sub_sign(base, TOKEN_DENOM);
    let mut term = TOKEN_DENOM;
    let mut sum = term;
    let mut negative = false;

    // term(k) = numer / denom 
    //         = (product(a - i - 1, i=1-->k) * x^k) / (k!)
    // each iteration, multiply previous term by (a-(k-1)) * x / k
    // continue until term is less than precision

    let mut i = 1;
    while term >= precision {
        let big_k = i * TOKEN_DENOM;
        let (c, cneg) = sub_sign(a, big_k - TOKEN_DENOM);

        term = mul_u128(term, mul_u128(c, x));
        term = div_u128(term, big_k);
        if term == 0 { break; }

        if xneg { negative = !negative };
        if cneg { negative = !negative };
        if negative {
            sum -= term;
        }
         else {
             sum += term;
         }
        i += 1;
    }

    sum
}

/*** operators that take decimals into account ***/

pub fn pow_u128(
    base: u128, 
    exp: u128
) -> u128 {
    assert!(base >= MIN_POW_BASE, "ERR_MIN_POW_BASE");
    assert!(base <= MAX_POW_BASE, "ERR_MAX_POW_BASE");

    let whole = floor_u128(base);
    let remain = exp - whole;

    let whole_pow = pow_i_u128(base, btoi(whole));

    if remain == 0 {
        return whole_pow
    }

    let partial_result = pow_approx(base, remain, POW_PRECISION);

    mul_u128(whole_pow, partial_result)
}


pub fn div_u128(a: u128, b: u128) -> u128 {
    let a_u256 = u256::from(a);
    
    let token_denom_u256 = u256::from(TOKEN_DENOM);
    let c0 = a_u256 * token_denom_u256;
    let c1 = c0 + (b / 2);

    (c1 / b).as_u128()
}

pub fn div_u256_to_u128(a: u256, b: u256) -> u128 {
    let token_denom_u256 = u256::from(TOKEN_DENOM);
    let c0 = a * token_denom_u256;
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

pub fn mul_u256(a: u256, b: u256) -> u256 {
    let token_denom_u256 = u256::from(TOKEN_DENOM);

    let c0: u256 = a * b;

    let c1 = c0 + (token_denom_u256 / 2);

    c1 / token_denom_u256
}