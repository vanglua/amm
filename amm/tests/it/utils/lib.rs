#![allow(clippy::needless_pass_by_value)]
use std::convert::TryInto;
use near_sdk::{
    PendingContractTx,
    AccountId,
    json_types::{
        U64,
        U128,
        // ValidAccountId
    },
    serde_json::json,
    // serde_json
};

use near_sdk_sim::{
    ExecutionResult,
    // call,
    // view,
    deploy, 
    init_simulator, 
    to_yocto, 
    ContractAccount, 
    UserAccount, 
    // STORAGE_AMOUNT,
    DEFAULT_GAS
};

mod amm_utils;
mod account_utils;
mod oracle_utils;
mod token_utils;
mod deposit;
mod helpers;

pub use account_utils::*;
pub use deposit::*;
pub use helpers::*;

extern crate amm;
pub use amm::*;

type OracleContract = oracle::ContractContract;
type TokenContract = token::TokenContractContract;
type AMMContract = amm::AMMContractContract;

pub const REGISTRY_STORAGE: u128 = 8_300_000_000_000_000_000_000;
pub const TOKEN_CONTRACT_ID: &str = "token";
pub const AMM_CONTRACT_ID: &str = "amm";
pub const ORACLE_CONTRACT_ID: &str = "oracle";
pub const SAFE_STORAGE_AMOUNT: u128 = 180000000000000000000000;

// Load in contract bytes
near_sdk_sim::lazy_static! {
    static ref ORACLE_WASM_BYTES: &'static [u8] = include_bytes!("../../../../res/oracle.wasm").as_ref();
    static ref AMM_WASM_BYTES: &'static [u8] = include_bytes!("../../../../res/amm.wasm").as_ref();
    static ref TOKEN_WASM_BYTES: &'static [u8] = include_bytes!("../../../../res/token.wasm").as_ref();
}

pub struct InitRes {
    pub master_account: TestAccount,
    pub amm_contract: ContractAccount<AMMContract>,
    pub oracle_contract: ContractAccount<OracleContract>,
    pub token_account_id: ContractAccount<TokenContract>,
    pub alice: account_utils::TestAccount,
    pub bob: account_utils::TestAccount,
    pub carol: account_utils::TestAccount
}


pub fn init(
    gov_id: AccountId
) -> InitRes {
    let master_account = TestAccount::new(None, None);
    let amm_init_res = amm_utils::AMMUtils::new(&master_account, gov_id.to_string()); // Init amm
    let oracle_init_res = oracle_utils::OracleUtils::new(&master_account, gov_id.to_string());  // Init oracle
    let token_init_res = token_utils::TokenUtils::new(&master_account); // Init token

    InitRes {
        alice: TestAccount::new(Some(&master_account.account), Some("alice")),
        bob: TestAccount::new(Some(&master_account.account), Some("bob")),
        carol: TestAccount::new(Some(&master_account.account), Some("carol")),
        master_account: master_account,
        amm_contract: amm_init_res.contract,
        oracle_contract: oracle_init_res.contract,
        token_account_id: token_init_res.contract, // should be doable like oracle and amm
    }
}
