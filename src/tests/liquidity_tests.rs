use super::*;

#[test]
fn join_pool_even_liq_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "alice".to_string());
    let transfer_amount = to_token_denom(100);
    transfer_unsafe(&token, &alice, bob.account_id().to_string(), transfer_amount);

    let pool_id: U64 = call!(
        alice,
        amm.new_pool(2, U128(0)),
        deposit = STORAGE_AMOUNT
    ).unwrap_json();

    assert_eq!(pool_id, U64(0));


    let seed_amount = to_token_denom(10);
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
    transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, finalize_args);

    let pool_token_balance: U128 = view!(amm.get_pool_token_balance(pool_id, &alice.account_id())).unwrap_json();
    assert_eq!(pool_token_balance, U128(seed_amount));
    let seeder_balance = get_balance(&token, alice.account_id().to_string());
    assert_eq!(seeder_balance, to_yocto("1") - seed_amount - transfer_amount);
    let amm_collateral_balance = get_balance(&token, "amm".to_string());
    assert_eq!(amm_collateral_balance, seed_amount);

    let join_args = json!({
        "function": "join_pool",
        "args": {
            "pool_id": pool_id,
        }
    }).to_string();
    transfer_with_vault(&token, &bob, "amm".to_string(), seed_amount, join_args);

    let pool_token_balance_after_join: U128 = view!(amm.get_pool_token_balance(pool_id, &bob.account_id())).unwrap_json();
    assert_eq!(pool_token_balance_after_join, U128(to_token_denom(10)));

    let joiner_balance = get_balance(&token, bob.account_id().to_string());
    assert_eq!(joiner_balance, transfer_amount - seed_amount);
    let amm_collateral_balance = get_balance(&token, "amm".to_string());
    assert_eq!(amm_collateral_balance, seed_amount * 2);
}

#[test]
fn join_pool_uneven_liq_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "alice".to_string());
    let transfer_amount = to_token_denom(100);
    transfer_unsafe(&token, &alice, bob.account_id().to_string(), transfer_amount);

    let pool_id: U64 = call!(
        alice,
        amm.new_pool(3, swap_fee()),
        deposit = STORAGE_AMOUNT
    ).unwrap_json();

    assert_eq!(pool_id, U64(0));

    let target_price_a = U128(to_token_denom(60) / 100);
    let target_price_b_c = U128(to_token_denom(20) / 100);

    let weights = calc_weights_from_price(vec![target_price_a, target_price_b_c,target_price_b_c]);
    let seed_amount = to_token_denom(100);

    let seed_pool_res = call!(
        alice,
        amm.seed_pool(pool_id, U128(seed_amount), weights),
        deposit = STORAGE_AMOUNT
    );

    let join_args = json!({
        "function": "join_pool",
        "args": {
            "pool_id": pool_id,
        }
    }).to_string();
    transfer_with_vault(&token, &bob, "amm".to_string(), seed_amount, join_args);

    let price_0: U128 = view!(amm.get_spot_price_sans_fee(pool_id, 0)).unwrap_json();
    let price_1: U128 = view!(amm.get_spot_price_sans_fee(pool_id, 1)).unwrap_json();
    let price_2: U128 = view!(amm.get_spot_price_sans_fee(pool_id, 2)).unwrap_json();

    assert_eq!(price_0, target_price_a);
    assert_eq!(price_1, target_price_b_c);
    assert_eq!(price_2, target_price_b_c);

    let pool_balances_after_seed: Vec<U128> = view!(amm.get_pool_balances(pool_id)).unwrap_json();

    let outcome_balance_0: U128 = view!(amm.get_share_balance(&alice.account_id(), pool_id, 0)).unwrap_json();
    let outcome_balance_1: U128 = view!(amm.get_share_balance(&alice.account_id(), pool_id, 1)).unwrap_json();
    let outcome_balance_2: U128 = view!(amm.get_share_balance(&alice.account_id(), pool_id, 2)).unwrap_json();

    assert_eq!(u128::from(outcome_balance_0), seed_amount - u128::from(pool_balances_after_seed[0]));
    assert_eq!(outcome_balance_1, U128(0));
    assert_eq!(outcome_balance_2, U128(0));


    let creator_pool_token_balance: U128 = view!(amm.get_pool_token_balance(pool_id, &alice.account_id())).unwrap_json();
    let finalize_args = json!({
        "function": "finalize",
        "args": {
            "pool_id": pool_id
        }
    }).to_string();
    transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, finalize_args);

    // Bob joins pool
    let join_args = json!({
        "function": "join_pool",
        "args": {
            "pool_id": pool_id,
        }
    }).to_string();
    transfer_with_vault(&token, &bob, "amm".to_string(), seed_amount, join_args);
    

    let joiner_share_balance_a: U128 = view!(amm.get_share_balance(&bob.account_id(), pool_id, 0)).unwrap_json();

    assert_eq!(u128::from(joiner_share_balance_a), seed_amount - u128::from(pool_balances_after_seed[0]));

    let joiner_pool_token_balance: U128 = view!(amm.get_pool_token_balance(pool_id, &bob.account_id())).unwrap_json();
    assert_eq!(creator_pool_token_balance, joiner_pool_token_balance);
}