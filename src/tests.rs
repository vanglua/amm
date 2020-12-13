use near_sdk::{
    AccountId, 
    VMContext, 
    testing_env, 
    MockedBlockchain, 
    collections::{
        Vector
    },
    json_types::{
        U128, 
        U64
    }
};

use crate::pool_factory::PoolFactory;

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
        current_account_id: alice(),
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
        prepaid_gas: 30000000000000000,
        random_seed: vec![0, 1, 2],
        output_data_receivers: vec![],
    }
}

mod init_tests;
mod pool_initiation_tests;
mod pricing_tests;