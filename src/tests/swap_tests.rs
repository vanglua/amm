use super::*;

#[test]
fn calc_buy_amount_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "alice".to_string());
    let seed_amount = to_token_denom(10);
    let half = to_token_denom(5) / 10;

    let pool_id: U64 = call!(
        alice,
        amm.new_pool(2, U128(0)),
        deposit = STORAGE_AMOUNT
    ).unwrap_json();

    assert_eq!(pool_id, U64(0));

    let even_seed_pool_res = call!(
        alice,
        amm.seed_pool(pool_id, U128(seed_amount), vec![U128(half), U128(half)]),
        deposit = STORAGE_AMOUNT
    );

    let buy_amt: U128 = view!(amm.calc_buy_amount(pool_id, U128(to_token_denom(1)), 0)).unwrap_json();
    assert_eq!(u128::from(buy_amt), 1_909_090_909_090_909_091);
}

#[test]
fn calc_sell_collateral_out_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "alice".to_string());
    let seed_amount = to_token_denom(10);
    let half = to_token_denom(5) / 10;

    let pool_id: U64 = call!(
        alice,
        amm.new_pool(2, U128(0)),
        deposit = STORAGE_AMOUNT
    ).unwrap_json();

    assert_eq!(pool_id, U64(0));

    let even_seed_pool_res = call!(
        alice,
        amm.seed_pool(pool_id, U128(seed_amount), vec![U128(half), U128(half)]),
        deposit = STORAGE_AMOUNT
    );

    let collat_out: U128 = view!(amm.calc_sell_collateral_out(pool_id, U128(to_token_denom(1)), 0)).unwrap_json();
    assert_eq!(u128::from(collat_out), 2_111_111_111_111_111_111);
}

#[test]
fn basic_buy_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "alice".to_string());
    let weight = to_token_denom(1) / 2;
    let seed_amount = to_token_denom(10);
    let buy_amount = to_token_denom(1);
    let invariant = to_token_denom(100);

    let pool_id: U64 = call!(
        alice,
        amm.new_pool(2, U128(0)),
        deposit = STORAGE_AMOUNT
    ).unwrap_json();

    assert_eq!(pool_id, U64(0));

    call!(
        alice,
        amm.seed_pool(pool_id, U128(seed_amount), vec![U128(weight), U128(weight)]),
        deposit = STORAGE_AMOUNT
    );

    let finalize_args = json!({
        "function": "finalize",
        "args": {
            "pool_id": pool_id
        }
    }).to_string();
    let finalize_pool_res = transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, finalize_args);

    let buy_args = json!({
        "function": "buy",
        "args": {
            "pool_id": pool_id,
            "outcome_target": 0,
            "min_shares_out": U128(to_token_denom(15) / 10)
        }
    }).to_string();

    let buy_res = transfer_with_vault(&token, &alice, "amm".to_string(), buy_amount, buy_args);
    println!("buy res: {:?}", buy_res);

    let seeder_balance = get_balance(&token, alice.account_id().to_string());
    assert_eq!(seeder_balance, to_yocto("1") - seed_amount - buy_amount);
    let amm_balance = get_balance(&token, "amm".to_string());
    assert_eq!(amm_balance, seed_amount + buy_amount);

    let pool_balances: Vec<U128> = view!(amm.get_pool_balances(pool_id)).unwrap_json();

    let expected_target_pool_balance = invariant / 11; 
    let expected_other_outcome_pool_balance = seed_amount + buy_amount;
    assert_eq!(pool_balances[0], U128(expected_target_pool_balance));
    assert_eq!(pool_balances[1], U128(expected_other_outcome_pool_balance));

    let expected_target_buyer_balance = seed_amount + buy_amount - expected_target_pool_balance;
    let expected_other_buyer_balance = 0;

    let target_buyer_balance: U128 = view!(amm.get_share_balance(&alice.account_id(), pool_id, 0)).unwrap_json();
    let other_buyer_balance: U128 = view!(amm.get_share_balance(&alice.account_id(), pool_id, 1)).unwrap_json();

    assert_eq!(expected_target_buyer_balance, u128::from(target_buyer_balance));
    assert_eq!(expected_other_buyer_balance, u128::from(other_buyer_balance));
}

#[test]
fn basic_sell_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "alice".to_string());
    let weight = to_token_denom(1) / 2;
    let seed_amount = to_token_denom(10);
    let buy_amount = to_token_denom(1);
    let invariant = to_token_denom(100);

    let pool_id: U64 = call!(
        alice,
        amm.new_pool(2, U128(0)),
        deposit = STORAGE_AMOUNT
    ).unwrap_json();

    assert_eq!(pool_id, U64(0));

    call!(
        alice,
        amm.seed_pool(pool_id, U128(seed_amount), vec![U128(weight), U128(weight)]),
        deposit = STORAGE_AMOUNT
    );

    let finalize_args = json!({
        "function": "finalize",
        "args": {
            "pool_id": pool_id
        }
    }).to_string();
    let finalize_pool_res = transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, finalize_args);

    let buy_args = json!({
        "function": "buy",
        "args": {
            "pool_id": pool_id,
            "outcome_target": 0,
            "min_shares_out": U128(to_token_denom(15) / 10)
        }
    }).to_string();

    transfer_with_vault(&token, &alice, "amm".to_string(), buy_amount, buy_args);

    let seeder_balance = get_balance(&token, alice.account_id().to_string());
    assert_eq!(seeder_balance, to_yocto("1") - seed_amount - buy_amount);
    let amm_balance = get_balance(&token, "amm".to_string());
    assert_eq!(amm_balance, seed_amount + buy_amount);

    let expected_target_pool_balance = invariant / 11; 

    let expected_target_buyer_balance = seed_amount + buy_amount - expected_target_pool_balance;
    let expected_other_buyer_balance = 0;

    let target_buyer_balance: U128 = view!(amm.get_share_balance(&alice.account_id(), pool_id, 0)).unwrap_json();
    let other_buyer_balance: U128 = view!(amm.get_share_balance(&alice.account_id(), pool_id, 1)).unwrap_json();

    assert_eq!(expected_target_buyer_balance, u128::from(target_buyer_balance));
    assert_eq!(expected_other_buyer_balance, u128::from(other_buyer_balance));
    let seeder_balance = get_balance(&token, alice.account_id().to_string());
    let res = call!(
        alice,
        amm.sell(pool_id, U128(to_token_denom(1)), 0, U128(expected_target_buyer_balance)),
        deposit = STORAGE_AMOUNT
    );

    let pool_balances: Vec<U128> = view!(amm.get_pool_balances(pool_id)).unwrap_json();
    assert_eq!(pool_balances[0], U128(seed_amount));
    assert_eq!(pool_balances[1], U128(seed_amount));

    let seeder_balance = get_balance(&token, alice.account_id().to_string());
    assert_eq!(seeder_balance, to_yocto("1") - seed_amount);
    let amm_balance = get_balance(&token, "amm".to_string());
    assert_eq!(amm_balance, seed_amount);
}

// Check price after uneven swaps 
#[test]
fn complex_buy_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "alice".to_string());
    transfer_unsafe(&token, &alice, bob.account_id().to_string(), to_token_denom(100));

    let seed_amount = to_token_denom(10);
    let buy_amount = to_token_denom(1);
    let invariant = to_token_denom(10);

    let pool_id: U64 = call!(
        alice,
        amm.new_pool(3, U128(0)),
        deposit = STORAGE_AMOUNT
    ).unwrap_json();

    let weights = calc_weights_from_price(
        vec![
            U128(to_token_denom(60)), 
            U128(to_token_denom(30)), 
            U128(to_token_denom(10))
        ]
    );

    assert_eq!(pool_id, U64(0));

    call!(
        alice,
        amm.seed_pool(pool_id, U128(seed_amount), weights),
        deposit = STORAGE_AMOUNT
    );

    let finalize_args = json!({
        "function": "finalize",
        "args": {
            "pool_id": pool_id
        }
    }).to_string();
    transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, finalize_args);

    let init_balances: Vec<U128> = view!(amm.get_pool_balances(pool_id)).unwrap_json();
    let init_invariant = product_of(&init_balances);

    let buy_args = json!({
        "function": "buy",
        "args": {
            "pool_id": pool_id,
            "outcome_target": 0,
            "min_shares_out": U128(to_token_denom(8) / 10)
        }
    }).to_string();
    transfer_with_vault(&token, &bob, "amm".to_string(), buy_amount, buy_args);
    
    let post_trade_balances: Vec<U128> = view!(amm.get_pool_balances(pool_id)).unwrap_json();
    let post_trade_invariant = product_of(&post_trade_balances);
    assert!(init_invariant - post_trade_invariant <  1000);

    let target_pool_balance: U128 = view!(amm.get_share_balance(&"amm".to_string(), pool_id, 0)).unwrap_json();
    let target_buyer_balance: U128 = view!(amm.get_share_balance(&bob.account_id(), pool_id, 0)).unwrap_json();
    let inverse_balances: Vec<U128> = vec![post_trade_balances[1], post_trade_balances[2]];
    let product_of_inverse = product_of(&inverse_balances);

    let expected_pool_target_balance = math::div_u128(post_trade_invariant, product_of_inverse);
    let expected_buyer_target_balance = u128::from(init_balances[0]) + buy_amount - expected_pool_target_balance;

    assert_eq!(U128(expected_buyer_target_balance), target_buyer_balance);
    assert_eq!(U128(expected_pool_target_balance), target_pool_balance);

    // TODO: doublecheck collateralization after trade simulation
}