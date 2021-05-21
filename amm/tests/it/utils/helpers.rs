use crate::utils::*;

pub fn alice() -> AccountId {
    "alice".to_string()
}

pub fn bob() -> AccountId {
    "bob".to_string()
}

pub fn carol() -> AccountId {
    "carol".to_string()
}

pub fn empty_string() -> String { "".to_string() }

pub fn empty_string_vec(len: u16) -> Vec<String> { 
    let mut tags: Vec<String> = vec![];
    for i in 0..len {
        tags.push(empty_string());
    }
    
    tags
}

pub fn env_time() -> U64{ 
    1609951265967.into()
}

pub fn fee() -> U128 {
    (10_u128.pow(24) / 50).into() // 2%
}

pub fn transfer_call_storage_amount() -> u128 {
    1
}

pub fn product_of(nums: &Vec<U128>) -> u128 {
    assert!(nums.len() > 1, "ERR_INVALID_NUMS");
    nums.iter().fold(to_yocto("1"), |prod, &num| {
        let num_u128: u128 = num.into();
        math::complex_mul_u128(to_yocto("1"), prod, num_u128)
    })
}

pub fn calc_weights_from_price(prices: Vec<U128>) -> Vec<U128> {
    let product = product_of(&prices);
    
    prices.iter().map(|price| {
       U128(math::complex_div_u128(to_yocto("1"), u128::from(product), u128::from(*price)))
    }).collect()
}