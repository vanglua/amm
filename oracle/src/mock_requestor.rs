use near_sdk::json_types::{ U64, U128 };
use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::AccountId;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Requestor( pub AccountId);

impl Requestor {
    pub fn get_tvl(&self, _request_id: U64) -> U128 {
        // TODO: Call tvl function on ext_contract
        5.into()
    }
}