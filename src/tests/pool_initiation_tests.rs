use super::*;

#[test]
fn pool_initial_state_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "alice".to_string());

    let pool_id: U64 = call!(
        alice,
        amm.new_pool(2, U128(0)),
        deposit = STORAGE_AMOUNT
    ).unwrap_json();

    assert_eq!(pool_id, U64(0));


    let seed_amount = to_token_denom(100);
    let half = to_token_denom(5) / 10;

    let seed_pool_res = call!(
        alice,
        amm.seed_pool(pool_id, U128(seed_amount), vec![U128(half), U128(half)]),
        deposit = STORAGE_AMOUNT
    );

    let finalize_args = json!({
        "function": "finalize",
        "args": {
            "pool_id": pool_id
        }
    }).to_string();
    let finalize_pool_res = transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, finalize_args);

    let seeder_balance = get_balance(&token, alice.account_id().to_string());
    assert_eq!(seeder_balance, to_yocto("1") - seed_amount);
    let amm_collateral_balance = get_balance(&token, "amm".to_string());
    assert_eq!(amm_collateral_balance, seed_amount);

    let pool_balances: Vec<U128> = view!(amm.get_pool_balances(pool_id)).unwrap_json();

    assert_eq!(pool_balances[0], pool_balances[1]);
    assert_eq!(pool_balances[0], U128(seed_amount));
    assert_eq!(pool_balances[1], U128(seed_amount));
}
