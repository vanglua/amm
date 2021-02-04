mod test_utils;
use test_utils::*;
use near_sdk::json_types::{U64, U128};
use near_sdk::serde_json::json;
use near_sdk_sim::{to_yocto, call, view, STORAGE_AMOUNT};

#[test]
fn pool_initial_state_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "alice".to_string(), "carol".to_string());

    let market_id = create_market(&alice, &amm, 2, Some(U128(0)));

    assert_eq!(market_id, U64(0));


    let seed_amount = to_token_denom(100);
    let half = to_token_denom(5) / 10;

    let seed_pool_res = call!(
        alice,
        amm.seed_pool(market_id, U128(seed_amount), vec![U128(half), U128(half)]),
        deposit = STORAGE_AMOUNT
    );

    let publish_args = json!({
        "function": "publish",
        "args": {
            "market_id": market_id
        }
    }).to_string();
    let publish_pool_res = transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, publish_args);

    let seeder_balance = get_balance(&token, alice.account_id().to_string());
    assert_eq!(seeder_balance, to_yocto("1") - seed_amount);
    let amm_collateral_balance = get_balance(&token, "amm".to_string());
    assert_eq!(amm_collateral_balance, seed_amount);

    let pool_balances: Vec<U128> = view!(amm.get_pool_balances(market_id)).unwrap_json();

    assert_eq!(pool_balances[0], pool_balances[1]);
    assert_eq!(pool_balances[0], U128(seed_amount));
    assert_eq!(pool_balances[1], U128(seed_amount));
}
