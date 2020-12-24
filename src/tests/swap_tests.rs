use super::*;

#[test]
fn calc_buy_amount_test() {
    let context = get_context(alice(), 0);
    testing_env!(context);
    let mut contract = PoolFactory::init(alice());

    let pool_id = contract.new_pool(2, U128(0));
    let half = to_token_denom(1) / 2;

    contract.seed_pool(pool_id, U128(to_token_denom(10)), vec![U128(half), U128(half)]);
    
    let amt: u128 = contract.calc_buy_amount(pool_id, U128(to_token_denom(1)), 0).into();
    let expected_amt = 1_909_090_909_090_909_091;

    assert_eq!(amt, expected_amt);
}

#[test]
fn calc_sell_collateral_out_test() {
    let context = get_context(alice(), 0);
    testing_env!(context);
    let mut contract = PoolFactory::init(alice());

    let pool_id = contract.new_pool(2, U128(0));
    let half = to_token_denom(1) / 2;

    contract.seed_pool(pool_id, U128(to_token_denom(10)), vec![U128(half), U128(half)]);
    
    let amt: u128 = contract.calc_sell_collateral_out(pool_id, U128(to_token_denom(1)), 0).into();
    let expected_amt = 2_111_111_111_111_111_111;

    assert_eq!(amt, expected_amt);
}


#[test]
fn basic_buy_test() {
    let context = get_context(alice(), 0);
    testing_env!(context);
    let mut contract = PoolFactory::init(alice());

    let pool_id = contract.new_pool(2, U128(0));
    let weight = to_token_denom(1) / 2;
    let seed_amt = to_token_denom(10);
    let buy_amount = to_token_denom(1);
    let invariant = to_token_denom(100);

    contract.seed_pool(pool_id, U128(seed_amt), vec![U128(weight), U128(weight)]);
    contract.finalize_pool(pool_id);

    contract.buy(pool_id, U128(to_token_denom(1)), 0, U128(to_token_denom(15) / 10));

    let expected_target_pool_balance = invariant / 11; 
    let expected_other_outcome_pool_balance = seed_amt + buy_amount;

    let target_pool_balance: u128 = contract.get_share_balance(&contract_id(), pool_id, 0).into();
    let other_pool_balance: u128 = contract.get_share_balance(&contract_id(), pool_id, 1).into();

    assert_eq!(expected_target_pool_balance, target_pool_balance);
    assert_eq!(other_pool_balance, expected_other_outcome_pool_balance);

    let expected_target_buyer_balance = seed_amt + buy_amount - expected_target_pool_balance;
    let expected_other_buyer_balance = 0;

    let target_buyer_balance: u128 = contract.get_share_balance(&alice(), pool_id, 0).into();
    let other_buyer_balance: u128 = contract.get_share_balance(&alice(), pool_id, 1).into();

    assert_eq!(expected_target_buyer_balance, target_buyer_balance);
    assert_eq!(expected_other_buyer_balance, other_buyer_balance);
}

#[test]
fn basic_sell_test() {
    let context = get_context(alice(), 0);
    testing_env!(context);
    let mut contract = PoolFactory::init(alice());

    let pool_id = contract.new_pool(2, U128(0));
    let weight = to_token_denom(1) / 2;
    let seed_amt = to_token_denom(10);
    let buy_amount = to_token_denom(1);
    let invariant = to_token_denom(100);

    contract.seed_pool(pool_id, U128(seed_amt), vec![U128(weight), U128(weight)]);
    contract.finalize_pool(pool_id);

    contract.buy(pool_id, U128(to_token_denom(1)), 0, U128(to_token_denom(15) / 10));

    let expected_target_pool_balance = invariant / 11; 
    let expected_other_outcome_pool_balance = seed_amt + buy_amount;

    let target_pool_balance: u128 = contract.get_share_balance(&contract_id(), pool_id, 0).into();
    let other_pool_balance: u128 = contract.get_share_balance(&contract_id(), pool_id, 1).into();

    assert_eq!(expected_target_pool_balance, target_pool_balance);
    assert_eq!(other_pool_balance, expected_other_outcome_pool_balance);

    let expected_target_buyer_balance = seed_amt + buy_amount - expected_target_pool_balance;
    let expected_other_buyer_balance = 0;

    let target_buyer_balance: u128 = contract.get_share_balance(&alice(), pool_id, 0).into();
    let other_buyer_balance: u128 = contract.get_share_balance(&alice(), pool_id, 1).into();

    assert_eq!(expected_target_buyer_balance, target_buyer_balance);
    assert_eq!(expected_other_buyer_balance, other_buyer_balance);

    contract.sell(pool_id, U128(to_token_denom(1)), 0, U128(expected_target_buyer_balance));

    let pool_balances = contract.get_pool_balances(pool_id);
    assert_eq!(pool_balances[0], U128(seed_amt));
    assert_eq!(pool_balances[1], U128(seed_amt));

}

// Check price after uneven swaps 
#[test]
fn complex_buy_test() {
    let context = get_context(alice(), 0);
    testing_env!(context);
    let mut contract = PoolFactory::init(alice());

    let pool_id = contract.new_pool(3, U128(0));
    let weights = calc_weights_from_price(
        vec![
            to_token_denom(60), 
            to_token_denom(30), 
            to_token_denom(10)
        ]
    ); // [300, 600, 1800]
    let seed_amt = to_token_denom(10);
    let buy_amount = to_token_denom(1);

    contract.seed_pool(pool_id, U128(seed_amt), vec![U128(weights[0]), U128(weights[1]), U128(weights[2])]);
    let init_balances = unwrap_u128_vec(&contract.get_pool_balances(pool_id));
    let init_invariant = product_of(&init_balances);

    contract.finalize_pool(pool_id);
    
    // Swap to bob as signer
    let context = get_context(bob(), 0);
    testing_env!(context);
    
    contract.buy(pool_id, U128(buy_amount), 0, U128(to_token_denom(8) / 10));
    
    let post_trade_balances = unwrap_u128_vec(&contract.get_pool_balances(pool_id));
    
    let post_trade_invariant = product_of(&init_balances);
    assert_eq!(init_invariant, post_trade_invariant);
    
    let target_pool_balance: u128 = contract.get_share_balance(&contract_id(), pool_id, 0).into();
    let target_buyer_balance: u128 = contract.get_share_balance(&bob(), pool_id, 0).into();
    
    let inverse_balances: Vec<u128> = vec![post_trade_balances[1], post_trade_balances[2]];
    let product_of_inverse = product_of(&inverse_balances);

    let expected_pool_target_balance = math::div_u128(init_invariant, product_of_inverse);
    let expected_buyer_target_balance = init_balances[0] + buy_amount - expected_pool_target_balance;

    assert_eq!(expected_buyer_target_balance, target_buyer_balance);
    assert_eq!(expected_pool_target_balance, target_pool_balance);
}