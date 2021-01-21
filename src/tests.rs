#![allow(clippy::needless_pass_by_value)]

use near_sdk::{
    AccountId,
    VMContext, 
    testing_env,
    MockedBlockchain, 
    json_types::{
        U64,
        U128
    },
    serde_json::json
};

use near_sdk_sim::{
    ExecutionResult,
    transaction::{
        ExecutionOutcome,
        ExecutionStatus
    },
    call, 
    deploy, 
    init_simulator, 
    near_crypto::Signer, 
    to_yocto, 
    view, 
    ContractAccount, 
    UserAccount, 
    STORAGE_AMOUNT,
    DEFAULT_GAS,
    account::AccessKey
};

use crate::constants;
use crate::math;
use crate::flux_protocol::FluxProtocolContract;
const REGISTRY_STORAGE: u128 = 8_300_000_000_000_000_000_000;

/// Load in contract bytes
near_sdk_sim::lazy_static! {
    static ref AMM_WASM_BYTES: &'static [u8] = include_bytes!("../res/flux_amm.wasm").as_ref();
    static ref TOKEN_WASM_BYTES: &'static [u8] = include_bytes!("../res/vault_token_w_logs.wasm").as_ref();
}

fn init(
    initial_balance: u128,
    owner_id: String,
    gov_id: String,
) -> (UserAccount, ContractAccount<FluxProtocolContract>, UserAccount, UserAccount, UserAccount, UserAccount) {
    let master_account = init_simulator(None);
    // deploy amm
    let amm_contract = deploy!(
        // Contract Proxy
        contract: FluxProtocolContract,
        // Contract account id
        contract_id: "amm",
        // Bytes of contract
        bytes: &AMM_WASM_BYTES,
        // User deploying the contract,
        signer_account: master_account,
        // init method
        init_method: init(owner_id.to_string(), gov_id.to_string(), vec!["token".to_string()])
    );

    let token_contract = master_account.create_user("token".to_string(), to_yocto("100"));
    let tx = token_contract.create_transaction(token_contract.account_id());
    // uses default values for deposit and gas
    let res = tx
        .transfer(to_yocto("1"))
        .deploy_contract((&TOKEN_WASM_BYTES).to_vec())
        .submit();

    init_token(&token_contract, owner_id.to_string(), initial_balance);
    
    let alice = master_account.create_user("alice".to_string(), to_yocto("1000"));
    
    register(&token_contract, &alice, &"amm".to_string());

    let bob = master_account.create_user("bob".to_string(), to_yocto("100"));
    register(&token_contract, &alice, &bob.account_id());
    let carol = master_account.create_user("carol".to_string(), to_yocto("100"));
    register(&token_contract, &alice, &carol.account_id());

    (master_account, amm_contract, token_contract, alice, bob, carol)
}

fn init_token(
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

fn get_balance(token_account: &UserAccount, account_id: AccountId) -> u128 {
    let tx = token_account.create_transaction(token_account.account_id());
    let args = json!({
        "account_id": account_id
    }).to_string().as_bytes().to_vec();
    let res = tx.function_call("get_balance".into(), args, DEFAULT_GAS, 0).submit();
    let balance: U128 = res.unwrap_json();
    balance.into()
}

fn register(token_account: &UserAccount, sender: &UserAccount, to_register: &AccountId)  {
    let tx = sender.create_transaction(token_account.account_id());
    let args = json!({
        "account_id": to_register
    }).to_string().as_bytes().to_vec();
    let res = tx.function_call("register_account".into(), args, DEFAULT_GAS, REGISTRY_STORAGE).submit();
    if !res.is_ok() {
        panic!("ERR_REGISTER_FAILED: {:?}", res);
    }
}

fn transfer_unsafe(token_account: &UserAccount, from: &UserAccount, to: AccountId, amt: u128)  {
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

fn transfer_with_vault(token_account: &UserAccount, from: &UserAccount, to: AccountId, amt: u128, payload: String) -> ExecutionResult {
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

fn empty_string() -> String { "".to_string() }

fn empty_string_vec(len: u16) -> Vec<String> { 
    let mut tags: Vec<String> = vec![];
    for i in 0..len {
        tags.push(empty_string());
    }
    
    tags
}

fn env_time() -> U64{ 
    1609951265967.into()
}
fn fee() -> U128 {
    (10_u128.pow(18) / 50).into() // 2%
}

fn create_market(creator: &UserAccount, amm: &ContractAccount<FluxProtocolContract>, outcomes: u16, fee_opt: Option<U128>) -> U64 {
    call!(
        creator,
        amm.create_market(empty_string(), empty_string(), outcomes, empty_string_vec(outcomes), env_time(), "token".to_string(), fee_opt.unwrap_or(fee())),
        deposit = STORAGE_AMOUNT
    ).unwrap_json()
}

fn to_token_denom(amt: u128) -> u128 {
    amt * 10_u128.pow(18)
}

fn swap_fee() -> U128 {
    U128(to_token_denom(3) / 1000)
}

fn product_of(nums: &Vec<U128>) -> u128 {
    assert!(nums.len() > 1, "ERR_INVALID_NUMS");
    let mut product = constants::TOKEN_DENOM;

    for price in nums.to_vec() {
        product = math::mul_u128(product, u128::from(price));
    }
    
    product
}

fn calc_weights_from_price(prices: Vec<U128>) -> Vec<U128> {
    let product = product_of(&prices);
    
    prices.iter().map(|price| {
       U128(math::div_u128(u128::from(product), u128::from(*price)))
    }).collect()
}

fn unwrap_u128_vec(vec_in: &Vec<U128>) -> Vec<u128> {
    vec_in.iter().map(|n| { u128::from(*n) }).collect()
}

fn wrap_u128_vec(vec_in: &Vec<u128>) -> Vec<U128> {
    vec_in.iter().map(|n| { U128(*n) }).collect()
}

// runtime tests
mod init_tests;
mod pool_initiation_tests;
mod pricing_tests;
mod swap_tests;
mod liquidity_tests;
mod fee_tests;
mod market_end_tests;