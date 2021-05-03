
#![allow(clippy::needless_pass_by_value)]
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    PromiseOrValue,
    Balance,
    StorageUsage,
    Gas,
    ext_contract,
    near_bindgen,
    Promise,
    PanicOnDefault,
    json_types::{
        U128, 
        U64,
        ValidAccountId
    },
    serde_json,
    AccountId,
    env,
    collections::{
        Vector,
        UnorderedMap
    },
};

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Whitelist(UnorderedMap<AccountId, u16>);

#[near_bindgen]
impl Contract {
    /**
     * @returns the whitelisted collateral tokens
     */
    pub fn get_collateral_whitelist(&self) -> Vec<(AccountId, u32)> {
        self.collateral_whitelist.to_vec()
    }
}