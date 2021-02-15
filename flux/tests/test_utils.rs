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
use flux::protocol::ProtocolContract;

const REGISTRY_STORAGE: u128 = 8_300_000_000_000_000_000_000;

struct InitRes {
    root: UserAccount,
    amm_contract: ContractAccount<protocol::ProtocolContract>,
    token_account: UserAccount,
    user_accounts: Vec<UserAccount>
}

// Load in contract bytes
near_sdk_sim::lazy_static! {
    static ref AMM_WASM_BYTES: &'static [u8] = include_bytes!("./wasm/flux.wasm").as_ref();
    static ref TOKEN_WASM_BYTES: &'static [u8] = include_bytes!("./wasm/vault_token_w_logs.wasm").as_ref();
}

pub fn init(
    initial_balance: u128,
    gov_id: AccountId,
) -> (UserAccount, ContractAccount<ProtocolContract>, UserAccount, UserAccount, UserAccount, UserAccount) {
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
            vec!["token".try_into().unwrap()],
            vec![24]
        )
    );

    let token_contract = master_account.create_user("token".to_string(), to_yocto("100"));
    let tx = token_contract.create_transaction(token_contract.account_id());
    // uses default values for deposit and gas
    let res = tx
        .transfer(to_yocto("1"))
        .deploy_contract((&TOKEN_WASM_BYTES).to_vec())
        .submit();

    init_token(&token_contract, "alice".to_string(), initial_balance);
    
    let alice = master_account.create_user("alice".to_string(), to_yocto("1000"));
    
    register(&token_contract, &alice, &"amm".to_string());

    let bob = master_account.create_user("bob".to_string(), to_yocto("100"));
    register(&token_contract, &alice, &bob.account_id());
    let carol = master_account.create_user("carol".to_string(), to_yocto("100"));
    register(&token_contract, &alice, &carol.account_id());

    (master_account, amm_contract, token_contract, alice, bob, carol)
}

pub fn token_denom() -> u128 {
    to_yocto("1")
}

pub fn init_token(
    token_contract: &UserAccount,
    owner_id: AccountId,
    initial_balance: u128
) {
    let tx = token_contract.create_transaction(token_contract.account_id());
    let args = json!({
        "owner_id": owner_id,
        "total_supply": U128(initial_balance)

    }).to_string().as_bytes().to_vec();
    let res = tx.function_call("init".into(), args, DEFAULT_GAS, 0).submit();
    if !res.is_ok() {
        panic!("token initiation failed: {:?}", res);
    }
}

pub fn get_balance(token_account: &UserAccount, account_id: AccountId) -> u128 {
    let tx = token_account.create_transaction(token_account.account_id());
    let args = json!({
        "account_id": account_id
    }).to_string().as_bytes().to_vec();
    let res = tx.function_call("get_balance".into(), args, DEFAULT_GAS, 0).submit();
    let balance: U128 = res.unwrap_json();
    balance.into()
}

pub fn register(token_account: &UserAccount, sender: &UserAccount, to_register: &AccountId)  {
    let tx = sender.create_transaction(token_account.account_id());
    let args = json!({
        "account_id": to_register
    }).to_string().as_bytes().to_vec();
    let res = tx.function_call("register_account".into(), args, DEFAULT_GAS, REGISTRY_STORAGE).submit();
    if !res.is_ok() {
        panic!("ERR_REGISTER_FAILED: {:?}", res);
    }
}

pub fn transfer_unsafe(token_account: &UserAccount, from: &UserAccount, to: AccountId, amt: u128)  {
    let tx = from.create_transaction(token_account.account_id());
    let args = json!({
        "receiver_id": to,
        "amount": U128(amt)
    }).to_string().as_bytes().to_vec();

    let res = tx.function_call("transfer".into(), args, DEFAULT_GAS, 0).submit();
    if !res.is_ok() {
        panic!("ERR_TRANSFER_FAILED: {:?}", res);
    }
}

pub fn transfer_with_vault(token_account: &UserAccount, from: &UserAccount, to: AccountId, amt: u128, payload: String) -> ExecutionResult {
    let tx = from.create_transaction(token_account.account_id());
    let args = json!({
        "receiver_id": to,
        "amount": U128(amt),
        "payload": payload
    }).to_string().as_bytes().to_vec();
    
    let res = tx.function_call("transfer_with_vault".into(), args, DEFAULT_GAS, STORAGE_AMOUNT).submit();
    if !res.is_ok() {
        panic!("tx failed: {:?}", res);
    }
    res
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

pub fn create_market(creator: &UserAccount, amm: &ContractAccount<ProtocolContract>, outcomes: u16, fee_opt: Option<U128>) -> U64 {
    call!(
        creator,
        amm.create_market(empty_string(), empty_string(), outcomes, empty_string_vec(outcomes), empty_string_vec(2), env_time(), "token".to_string(), fee_opt.unwrap_or(fee())),
        deposit = STORAGE_AMOUNT
    ).unwrap_json()
}

pub fn to_token_denom(amt: u128) -> u128 {
    amt * 10_u128.pow(24)
}

pub fn swap_fee() -> U128 {
    U128(to_token_denom(2) / 100)
}

pub fn product_of(nums: &Vec<U128>) -> u128 {
    assert!(nums.len() > 1, "ERR_INVALID_NUMS");
    let mut product = 0;
    for price in nums.to_vec() {
        product = math::mul_u128(token_denom(),token_denom(), u128::from(price));
    }
    product
}

pub fn calc_weights_from_price(prices: Vec<U128>) -> Vec<U128> {
    let product = product_of(&prices);
    
    prices.iter().map(|price| {
       U128(math::div_u128(token_denom(), u128::from(product), u128::from(*price)))
    }).collect()
}

pub fn unwrap_u128_vec(vec_in: &Vec<U128>) -> Vec<u128> {
    vec_in.iter().map(|n| { u128::from(*n) }).collect()
}

pub fn wrap_u128_vec(vec_in: &Vec<u128>) -> Vec<U128> {
    vec_in.iter().map(|n| { U128(*n) }).collect()
}