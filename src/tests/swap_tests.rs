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
    let expected_amt = 2111111111111111111;

    assert_eq!(amt, expected_amt);
}


#[test]
fn buy_test() {
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

    let target_pool_balance: u128 = contract.get_outcome_balance(&contract_id(), pool_id, 0).into();
    let other_pool_balance: u128 = contract.get_outcome_balance(&contract_id(), pool_id, 1).into();

    assert_eq!(expected_target_pool_balance, target_pool_balance);
    assert_eq!(other_pool_balance, expected_other_outcome_pool_balance);

    let expected_target_buyer_balance = seed_amt + buy_amount - expected_target_pool_balance;
    let expected_other_buyer_balance = 0;

    let target_buyer_balance: u128 = contract.get_outcome_balance(&alice(), pool_id, 0).into();
    let other_buyer_balance: u128 = contract.get_outcome_balance(&alice(), pool_id, 1).into();

    assert_eq!(expected_target_buyer_balance, target_buyer_balance);
    assert_eq!(expected_other_buyer_balance, other_buyer_balance);
}