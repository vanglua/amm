use crate::*;
use near_sdk::json_types::{ U64 };
use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::AccountId;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TargetContract( pub AccountId);

impl TargetContract {
    pub fn set_outcome(&self, _request_id: U64, _outcome: data_request::Outcome) -> bool {
        // TODO: Call tvl function on ext_contract
        true
    }
}