mod test_utils;
use test_utils::*;
use near_sdk::json_types::{U64, U128};
use near_sdk::serde_json::json;
use near_sdk_sim::{to_yocto, call, view, STORAGE_AMOUNT};

#[test]
fn add_liquidity_even_liq_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("100000"), "carol".to_string());
    let transfer_amount = to_token_denom(100);
    transfer_unsafe(&token, &alice, bob.account_id().to_string(), transfer_amount);

    let market_id: U64 = create_market(&alice, &amm, 2, Some(U128(0)));

    assert_eq!(market_id, U64(0));

    let seed_amount = to_token_denom(10);
    let half = U128(to_token_denom(5) / 10);
    let weights = vec![half, half];
    let add_liquidity_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
            "weight_indication": weights
        }
    }).to_string();
    transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, add_liquidity_args);

    let pool_token_balance: U128 = view!(amm.get_pool_token_balance(market_id, &alice.account_id())).unwrap_json();
    assert_eq!(pool_token_balance, U128(seed_amount));
    let seeder_balance = get_balance(&token, alice.account_id().to_string());
    assert_eq!(seeder_balance, to_yocto("1") - seed_amount - transfer_amount);
    let amm_collateral_balance = get_balance(&token, "amm".to_string());
    assert_eq!(amm_collateral_balance, seed_amount);

    let join_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
        }
    }).to_string();
    transfer_with_vault(&token, &bob, "amm".to_string(), seed_amount, join_args);

    let pool_token_balance_after_join: U128 = view!(amm.get_pool_token_balance(market_id, &bob.account_id())).unwrap_json();
    assert_eq!(pool_token_balance_after_join, U128(to_token_denom(10)));

    let joiner_balance = get_balance(&token, bob.account_id().to_string());
    assert_eq!(joiner_balance, transfer_amount - seed_amount);
    let amm_collateral_balance = get_balance(&token, "amm".to_string());
    assert_eq!(amm_collateral_balance, seed_amount * 2);
}

#[test]
fn add_liquidity_uneven_liq_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("100000"), "carol".to_string());
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("100000"), "carol".to_string());
    let transfer_amount = to_token_denom(100);
    transfer_unsafe(&token, &alice, bob.account_id().to_string(), transfer_amount);

    let market_id: U64 = create_market(&alice, &amm, 3, Some(U128(0)));

    assert_eq!(market_id, U64(0));

    let target_price_a = U128(to_token_denom(60) / 100);
    let target_price_b_c = U128(to_token_denom(20) / 100);

    let weights = calc_weights_from_price(vec![target_price_a, target_price_b_c,target_price_b_c]);
    let seed_amount = to_token_denom(100);

    let add_liquidity_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
            "weight_indication": weights
        }
    }).to_string();
    transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, add_liquidity_args);

    let price_0: U128 = view!(amm.get_spot_price_sans_fee(market_id, 0)).unwrap_json();
    let price_1: U128 = view!(amm.get_spot_price_sans_fee(market_id, 1)).unwrap_json();
    let price_2: U128 = view!(amm.get_spot_price_sans_fee(market_id, 2)).unwrap_json();

    assert_eq!(price_0, target_price_a);
    assert_eq!(price_1, target_price_b_c);
    assert_eq!(price_2, target_price_b_c);

    let pool_balances_after_seed: Vec<U128> = view!(amm.get_pool_balances(market_id)).unwrap_json();

    let outcome_balance_0: U128 = view!(amm.get_share_balance(&alice.account_id(), market_id, 0)).unwrap_json();
    let outcome_balance_1: U128 = view!(amm.get_share_balance(&alice.account_id(), market_id, 1)).unwrap_json();
    let outcome_balance_2: U128 = view!(amm.get_share_balance(&alice.account_id(), market_id, 2)).unwrap_json();

    assert_eq!(u128::from(outcome_balance_0), seed_amount - u128::from(pool_balances_after_seed[0]));
    assert_eq!(outcome_balance_1, U128(0));
    assert_eq!(outcome_balance_2, U128(0));


    let creator_pool_token_balance: U128 = view!(amm.get_pool_token_balance(market_id, &alice.account_id())).unwrap_json();
    // Bob joins pool
    let join_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
        }
    }).to_string();
    transfer_with_vault(&token, &bob, "amm".to_string(), seed_amount, join_args);


    let joiner_share_balance_a: U128 = view!(amm.get_share_balance(&bob.account_id(), market_id, 0)).unwrap_json();

    assert_eq!(u128::from(joiner_share_balance_a), seed_amount - u128::from(pool_balances_after_seed[0]));

    let joiner_pool_token_balance: U128 = view!(amm.get_pool_token_balance(market_id, &bob.account_id())).unwrap_json();
    assert_eq!(creator_pool_token_balance, joiner_pool_token_balance);
}

#[test]
fn multiple_pool_exits_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("100000"), "carol".to_string());
    let market_id: U64 = create_market(&bob, &amm, 2, Some(U128(0)));
    let transfer_amount = to_token_denom(100);
    transfer_unsafe(&token, &alice, bob.account_id().to_string(), transfer_amount);
    assert_eq!(market_id, U64(0));

    let seed_amount = to_token_denom(100);
    let join_amount0 = to_token_denom(500);
    let join_amount1 = to_token_denom(300);

    let exit_amount0 = to_token_denom(100);
    let exit_amount1 = to_token_denom(200);
    let exit_amount2 = to_token_denom(50);
    let exit_amount3 = to_token_denom(33);

    let buy_amount = to_token_denom(10);

    let half = U128(to_token_denom(5) / 10);
    let weights = Some(vec![half, half]);
    let add_liquidity_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
            "weight_indication": weights
        }
    }).to_string();
    transfer_with_vault(&token, &bob, "amm".to_string(), seed_amount, add_liquidity_args);

    let pool_token_balance: U128 = view!(amm.get_pool_token_balance(market_id, &bob.account_id())).unwrap_json();
    assert_eq!(pool_token_balance, U128(seed_amount));

    let join_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
        }
    }).to_string();
    transfer_with_vault(&token, &alice, "amm".to_string(), join_amount0, join_args.to_string());


    let buy_args = json!({
        "function": "buy",
        "args": {
            "market_id": market_id,
            "outcome_target": 0,
            "min_shares_out": U128(to_token_denom(15) / 10)
        }
    }).to_string();
    let buy_args2 = json!({
        "function": "buy",
        "args": {
            "market_id": market_id,
            "outcome_target": 1,
            "min_shares_out": U128(to_token_denom(15) / 10)
        }
    }).to_string();

    let buy_res = transfer_with_vault(&token, &alice, "amm".to_string(), buy_amount, buy_args);
    let buy_res = transfer_with_vault(&token, &alice, "amm".to_string(), buy_amount, buy_args2);

    transfer_with_vault(&token, &alice, "amm".to_string(), join_amount1, join_args);
    let alice_pool_token_balance_pre_exit: U128 = view!(amm.get_pool_token_balance(market_id, &alice.account_id())).unwrap_json();


    let alice_exit_res = call!(
        alice,
        amm.exit_pool(market_id, U128(exit_amount0)),
        deposit = STORAGE_AMOUNT
    );
    assert!(alice_exit_res.is_ok());

    let alice_exit_res1 = call!(
        alice,
        amm.exit_pool(market_id, U128(exit_amount1)),
        deposit = STORAGE_AMOUNT
    );
    assert!(alice_exit_res1.is_ok());

    let alice_exit_res2 = call!(
        alice,
        amm.exit_pool(market_id, U128(exit_amount2)),
        deposit = STORAGE_AMOUNT
    );
    assert!(alice_exit_res2.is_ok());

    let alice_exit_res3 = call!(
        alice,
        amm.exit_pool(market_id, U128(exit_amount3)),
        deposit = STORAGE_AMOUNT
    );
    assert!(alice_exit_res3.is_ok());

    // assert pool balances
    let alice_pool_token_balance_post_exit: U128 = view!(amm.get_pool_token_balance(market_id, &alice.account_id())).unwrap_json();
    assert_eq!(alice_pool_token_balance_post_exit, U128(u128::from(alice_pool_token_balance_pre_exit) - exit_amount0 - exit_amount1 -exit_amount2 - exit_amount3));
}

#[test]
fn join_zero_liq_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("100000"), "carol".to_string());
    let market_id: U64 = create_market(&bob, &amm, 2, Some(U128(0)));
    let transfer_amount = to_token_denom(100);
    transfer_unsafe(&token, &alice, bob.account_id().to_string(), transfer_amount);
    assert_eq!(market_id, U64(0));

    let seed_amount = to_token_denom(100);
    let join_amount0 = to_token_denom(500);

    let half = U128(to_token_denom(5) / 10);
    let weights = Some(vec![half, half]);

    let add_liquidity_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
            "weight_indication": weights
        }
    }).to_string();
    transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, add_liquidity_args);

    let pool_token_balance: U128 = view!(amm.get_pool_token_balance(market_id, &alice.account_id())).unwrap_json();
    assert_eq!(pool_token_balance, U128(seed_amount));

    let seed_exit_res = call!(
        alice,
        amm.exit_pool(market_id, U128(seed_amount)),
        deposit = STORAGE_AMOUNT
    );
    assert!(seed_exit_res.is_ok());

    let join_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
            "weight_indication": weights
        }
    }).to_string();
    let join_res = transfer_with_vault(&token, &alice, "amm".to_string(), join_amount0, join_args.to_string());
    assert!(join_res.is_ok());
}

#[test]
fn add_liquidity_redeem() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("100000"), "carol".to_string());

    // Fund Bob
    let transfer_amount = to_token_denom(100);
    transfer_unsafe(&token, &alice, bob.account_id().to_string(), transfer_amount);

    // Create / validate market
    let market_id: U64 = create_market(&bob, &amm, 2, Some(U128(0)));
    assert_eq!(market_id, U64(0));

    // Seed params
    let seed_amount = to_token_denom(10);
    let half = U128(to_token_denom(5) / 10);
    let weights = vec![half, half];

    // Add liquidity
    let add_liquidity_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
            "weight_indication": weights
        }
    }).to_string();
    transfer_with_vault(&token, &bob, "amm".to_string(), seed_amount, add_liquidity_args);

    // Exit pool
    let bob_exit_res = call!(
        bob,
        amm.exit_pool(market_id, U128(seed_amount)),
        deposit = STORAGE_AMOUNT
    );
    assert!(bob_exit_res.is_ok());

    // Redeem liquidity
    let redeem_call = call!(
        bob,
        amm.burn_outcome_tokens_redeem_collateral(market_id, U128(seed_amount)),
        deposit = STORAGE_AMOUNT
    );
    assert!(redeem_call.is_ok());

    // Assert pool token balance
    let pool_token_balance: U128 = view!(amm.get_pool_token_balance(market_id, &bob.account_id())).unwrap_json();
    assert_eq!(pool_token_balance, U128(0));
 
    // Assert collateral balance
    let collateral_balance = get_balance(&token, bob.account_id());
    assert_eq!(collateral_balance, transfer_amount);
    
    // Assert if shares are burned
    let outcome_balance_0: U128 = view!(amm.get_share_balance(&bob.account_id(), market_id, 0)).unwrap_json();
    let outcome_balance_1: U128 = view!(amm.get_share_balance(&bob.account_id(), market_id, 1)).unwrap_json();
    assert_eq!(outcome_balance_0, U128(0));
    assert_eq!(outcome_balance_1, U128(0));
}