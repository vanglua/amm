use near_sdk::{
    json_types::{
        U128, 
        U64
    },
    serde_json,
    serde_json::Value,
};

use near_sdk::serde::{
    Serialize, 
    Deserialize,
    de::{
        DeserializeOwned
    }
};

#[derive(Serialize, Deserialize)]
pub struct InitStruct {
    pub function: String,
    pub args: Value
}

#[derive(Serialize, Deserialize)]
pub struct SeedPool {
    pub market_id: U64,
    pub denorm_weights: Vec<U128>,
}

#[derive(Serialize, Deserialize)]
pub struct LPPool {
    pub market_id: U64,
}

#[derive(Serialize, Deserialize)]
pub struct Buy {
    pub market_id: U64,
    pub outcome_target: u16,
    pub min_shares_out: U128
}

#[derive(Serialize, Deserialize)]
pub struct Sell  {
    pub market_id: U64,
    pub outcome_target: u16,
    pub max_shares_in: U128
}


pub fn from_args<T: DeserializeOwned>(args: Value) -> T {
    serde_json::from_value(args).expect("ERR_INVALID_ARGS")
}