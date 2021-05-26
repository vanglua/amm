use crate:: *;
use near_sdk::serde::{Serialize, Deserialize};

pub type Timestamp = u64;
pub type WrappedTimestamp = U64;
pub type WrappedBalance = U128;

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize, Debug, PartialEq, Clone)]
pub enum Outcome {
    Answer(String),
    Invalid
}

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize)]
pub struct Source {
    pub end_point: String,
    pub source_path: String
}