use crate::utils::*;
use near_sdk::json_types::{U64, U128};
use near_sdk_sim::{to_yocto, view};

#[test]
fn pool_initial_state_test() {
    let test_utils = TestUtils::init(carol());
    let oracle = test_utils.oracle_contract;
    
    // Test that data_request is created at market creation
    let creation_bond = 100;
    test_utils.alice.create_market(2, Some(U128(0)));
    let dr_exists: bool = view!(oracle.dr_exists(U64(0))).unwrap_json();
    assert!(dr_exists, "data request was not successfully created");
    
    let seed_amount = to_yocto("100");
    let half = to_yocto("5") / 10;
    let weights = Some(vec![U128(half), U128(half)]);

    test_utils.alice.add_liquidity(0, seed_amount, weights);

    let seeder_balance = test_utils.alice.get_token_balance(None);
    assert_eq!(seeder_balance, init_balance() / 2 - seed_amount - creation_bond);
    let amm_collateral_balance = test_utils.alice.get_token_balance(Some(AMM_CONTRACT_ID.to_string()));
    assert_eq!(amm_collateral_balance, seed_amount);
    let oracle_collateral_balance = test_utils.alice.get_token_balance(Some(ORACLE_CONTRACT_ID.to_string()));
    assert_eq!(oracle_collateral_balance, creation_bond);

    let pool_balances: Vec<u128> = test_utils.alice.get_pool_balances(0);

    assert_eq!(pool_balances[0], pool_balances[1]);
    assert_eq!(pool_balances[0], seed_amount);
    assert_eq!(pool_balances[1], seed_amount);
}
