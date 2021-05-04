use near_sdk::{ ext_contract, Promise, Gas, env };

#[ext_contract]
pub trait OracleContractExt {
    fn get_config() -> Promise;
}

pub fn fetch_oracle_config(oracle_contract_id: &str) -> Promise {
    oracle_contract_ext::get_config(&oracle_contract_id, 1, (env::prepaid_gas() - env::used_gas()))
}