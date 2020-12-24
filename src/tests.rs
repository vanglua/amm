#![allow(clippy::needless_pass_by_value)]

use near_sdk::{
    AccountId, 
    VMContext, 
    testing_env, 
    MockedBlockchain, 
    json_types::{
        U64,
        U128
    }
};

use crate::constants;
use crate::math;
use crate::pool_factory::PoolFactory;

fn contract_id() -> String {
    "contract".to_string()
}

fn alice() -> String {
    "alice".to_string()
}

fn bob() -> String {
    "bob".to_string()
}

fn carol() -> String {
    "carol".to_string()
}

fn token_a() -> String {
    "t1".to_string()
}

fn token_b() -> String {
    "t2".to_string()
}

fn token_c() -> String {
    "t3".to_string()
}

fn to_token_denom(amt: u128) -> u128 {
    amt * 10_u128.pow(18)
}

fn swap_fee() -> U128 {
    U128(to_token_denom(3) / 1000)
}

fn get_context(
    predecessor_account_id: AccountId, 
    block_timestamp: u64
) -> VMContext {

    VMContext {
        current_account_id: "contract".to_string(),
        signer_account_id: bob(),
        signer_account_pk: vec![0, 1, 2],
        predecessor_account_id,
        input: vec![],
        block_index: 0,
        epoch_height: 0,
        account_balance: 0,
        is_view: false,
        storage_usage: 10000,
        block_timestamp,
        account_locked_balance: 0,
        attached_deposit: 0,
        prepaid_gas: 30_000_000_000_000_000,
        random_seed: vec![0, 1, 2],
        output_data_receivers: vec![],
    }
}

fn product_of(nums: &Vec<u128>) -> u128 {
    assert!(nums.len() > 1, "ERR_INVALID_NUMS");
    let mut product = constants::TOKEN_DENOM;

    for price in nums.to_vec() {
        product = math::mul_u128(product, price);
    }
    
    product
}

fn calc_weights_from_price(prices: Vec<u128>) -> Vec<u128> {
    let product = product_of(&prices);
    
    prices.iter().map(|price| {
       math::div_u128(product, *price)
    }).collect()
}

fn unwrap_u128_vec(vec_in: &Vec<U128>) -> Vec<u128> {
    vec_in.iter().map(|n| { u128::from(*n) }).collect()
}

fn wrap_u128_vec(vec_in: &Vec<u128>) -> Vec<U128> {
    vec_in.iter().map(|n| { U128(*n) }).collect()
}

mod init_tests;
mod pool_initiation_tests;
mod pricing_tests;
mod swap_tests;
mod liquidity_tests;
mod fee_tests;