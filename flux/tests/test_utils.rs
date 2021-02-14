#![allow(clippy::needless_pass_by_value)]
use std::convert::TryInto;
use near_sdk::{
    AccountId,
    json_types::{
        U64,
        U128,
        ValidAccountId
    },
    serde_json::json
};

use near_sdk_sim::{
    ExecutionResult,
    call,
    view,
    deploy, 
    init_simulator, 
    to_yocto, 
    ContractAccount, 
    UserAccount, 
    STORAGE_AMOUNT,
    DEFAULT_GAS
};

extern crate flux;

pub use flux::*;
use token::*;
use flux::protocol::ProtocolContract;

const REGISTRY_STORAGE: u128 = 8_300_000_000_000_000_000_000;

// Load in contract bytes
near_sdk_sim::lazy_static! {
    static ref AMM_WASM_BYTES: &'static [u8] = include_bytes!("./wasm/flux.wasm").as_ref();
    static ref TOKEN_WASM_BYTES: &'static [u8] = include_bytes!("./wasm/token.wasm").as_ref();
}

pub fn init(
    initial_balance: u128,
    gov_id: AccountId,
) -> (UserAccount, ContractAccount<ProtocolContract>, ContractAccount<ContractContract>, UserAccount, UserAccount, UserAccount) {
    let master_account = init_simulator(None);

    // deploy amm
    let amm_contract = deploy!(
        // Contract Proxy
        contract: ProtocolContract,
        // Contract account id
        contract_id: "amm",
        // Bytes of contract
        bytes: &AMM_WASM_BYTES,
        // User deploying the contract,
        signer_account: master_account,
        deposit: to_yocto("1000"),
        // init method
        init_method: init(
            gov_id.try_into().unwrap(),
            vec!["token".try_into().unwrap()]
        )
    );

    // deploy token
    let token_contract = deploy!(
        // Contract Proxy
        contract: ContractContract,
        // Contract account id
        contract_id: "token",
        // Bytes of contract
        bytes: &TOKEN_WASM_BYTES,
        // User deploying the contract,
        signer_account: master_account,
        deposit: to_yocto("1000"),
        // init method
        init_method: new()
    );

    // I need to access `storage_minimum_balance`
    let storage_amount: U128 = view!(token_contract.storage_minimum_balance()).unwrap_json();
    println!("sa {:?}", storage_amount);
    let alice = master_account.create_user("alice".to_string(), to_yocto("100"));
    let res = call!(
        alice,
        token_contract.storage_deposit(None),
        deposit = storage_amount.into()
    );
    println!("r1 {:?}", res);
    let bob = master_account.create_user("bob".to_string(), to_yocto("100"));
    // token_contract.storage_deposit(Some("bob".to_string().try_into().unwrap()));
    let carol = master_account.create_user("carol".to_string(), to_yocto("100"));
    // token_contract.storage_deposit(Some("carol".to_string().try_into().unwrap()));

    (master_account, amm_contract, token_contract, alice, bob, carol)
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
    (10_u128.pow(18) / 50).into() // 2%
}

pub fn create_market(creator: &UserAccount, amm: &ContractAccount<ProtocolContract>, outcomes: u16, fee_opt: Option<U128>) -> U64 {
    call!(
        creator,
        amm.create_market(empty_string(), empty_string(), outcomes, empty_string_vec(outcomes), empty_string_vec(2), env_time(), "token".to_string(), fee_opt.unwrap_or(fee())),
        deposit = STORAGE_AMOUNT
    ).unwrap_json()
}

pub fn to_token_denom(amt: u128) -> u128 {
    amt * 10_u128.pow(18)
}

pub fn swap_fee() -> U128 {
    U128(to_token_denom(2) / 100)
}

pub fn product_of(nums: &Vec<U128>) -> u128 {
    assert!(nums.len() > 1, "ERR_INVALID_NUMS");
    let mut product = constants::TOKEN_DENOM;

    for price in nums.to_vec() {
        product = math::mul_u128(product, u128::from(price));
    }
    
    product
}

pub fn calc_weights_from_price(prices: Vec<U128>) -> Vec<U128> {
    let product = product_of(&prices);
    
    prices.iter().map(|price| {
       U128(math::div_u128(u128::from(product), u128::from(*price)))
    }).collect()
}

pub fn unwrap_u128_vec(vec_in: &Vec<U128>) -> Vec<u128> {
    vec_in.iter().map(|n| { u128::from(*n) }).collect()
}

pub fn wrap_u128_vec(vec_in: &Vec<u128>) -> Vec<U128> {
    vec_in.iter().map(|n| { U128(*n) }).collect()
}