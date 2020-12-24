use super::*;

#[test]
fn join_pool_even_liq_test() {
    let context = get_context(alice(), 0);
    testing_env!(context);
    let mut contract = PoolFactory::init(alice());

    let pool_id = contract.new_pool(2, swap_fee());
    let half = to_token_denom(5) / 10;

    contract.seed_pool(pool_id, U128(to_token_denom(100)), vec![U128(half), U128(half)]);
    
    let pool_token_balance = contract.get_pool_token_balance(pool_id, &alice());
    assert_eq!(pool_token_balance, U128(to_token_denom(100)));
    contract.finalize_pool(pool_id);

    let context = get_context(bob(), 0);
    testing_env!(context);
    contract.join_pool(pool_id, U128(to_token_denom(10)));
    let pool_token_balance = contract.get_pool_token_balance(pool_id, &bob());
    assert_eq!(pool_token_balance, U128(to_token_denom(10)));
}

#[test]
fn join_pool_uneven_liq_test() {
    let context = get_context(alice(), 0);
    testing_env!(context);
    let mut contract = PoolFactory::init(alice());

    let pool_id = contract.new_pool(3, swap_fee());
    
    let target_price_a = to_token_denom(60) / 100;
    let target_price_b_c = to_token_denom(20) / 100;

    let weights = calc_weights_from_price(vec![target_price_a, target_price_b_c,target_price_b_c]);
    let seed_amount = to_token_denom(100);
    contract.seed_pool(pool_id, U128(seed_amount), vec![U128(weights[0]), U128(weights[1]), U128(weights[2])]);

    let price_a: u128 = contract.get_spot_price_sans_fee(pool_id, 0).into();
    let price_b: u128 = contract.get_spot_price_sans_fee(pool_id, 1).into();
    let price_c: u128 = contract.get_spot_price_sans_fee(pool_id, 2).into();

    assert_eq!(price_a, target_price_a);
    assert_eq!(price_b, target_price_b_c);
    assert_eq!(price_c, target_price_b_c);

    let pool_balances = contract.get_pool_balances(pool_id);

    let outcome_balance_a: u128 = contract.get_share_balance(&alice(), pool_id, 0).into();
    let outcome_balance_b: u128 = contract.get_share_balance(&alice(), pool_id, 1).into();
    let outcome_balance_c: u128 = contract.get_share_balance(&alice(), pool_id, 2).into();

    assert_eq!(outcome_balance_a, seed_amount - u128::from(pool_balances[0]));
    assert_eq!(outcome_balance_b, 0);
    assert_eq!(outcome_balance_c, 0);

    let creator_pool_token_balance: u128 = contract.get_pool_token_balance(pool_id, &alice()).into();
    contract.finalize_pool(pool_id);

    // different account joins same pool
    let new_context = get_context(bob(), 0);
    testing_env!(new_context);
    contract.join_pool(pool_id, U128(seed_amount));

    let joiner_share_balance_a: u128 = contract.get_share_balance(&alice(), pool_id, 0).into();

    assert_eq!(joiner_share_balance_a, seed_amount - u128::from(pool_balances[0]));

    let joiner_pool_token_balance: u128 = contract.get_pool_token_balance(pool_id, &bob()).into();
    assert_eq!(creator_pool_token_balance, joiner_pool_token_balance);
}