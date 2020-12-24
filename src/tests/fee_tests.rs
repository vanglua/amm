use super::*;

#[test]
fn lp_fee_test() {
    let context = get_context(alice(), 0);
    testing_env!(context);
    let funder = &alice();
    let joiner = &bob();
    let trader = &carol();
    let mut contract = PoolFactory::init(funder.to_string());
    let swap_fee = to_token_denom(2) / 100;
    let pool_id = contract.new_pool(2, U128(swap_fee));
    

    let target_price_a = to_token_denom(5) / 10;
    let target_price_b = to_token_denom(5) / 10;

    let buy_amt = to_token_denom(100);
    let weights = calc_weights_from_price(vec![target_price_a, target_price_b]);
    let seed_amount = to_token_denom(1000);
    
    // SEED
    contract.seed_pool(pool_id, U128(seed_amount), wrap_u128(weights));
    
    let creator_pool_token_balance: u128 = contract.get_pool_token_balance(pool_id, &alice()).into();
    
    contract.finalize_pool(pool_id);

    // $1000 in swaps at 2% fee
    testing_env!(get_context(trader.to_string(), 0));
    contract.buy(pool_id, U128(buy_amt), 0, U128(0));
    contract.buy(pool_id, U128(buy_amt), 1, U128(0));
    contract.buy(pool_id, U128(buy_amt), 0, U128(0));
    contract.buy(pool_id, U128(buy_amt), 1, U128(0));
    contract.buy(pool_id, U128(buy_amt), 0, U128(0));
    contract.buy(pool_id, U128(buy_amt), 1, U128(0));
    contract.buy(pool_id, U128(buy_amt), 0, U128(0));
    contract.buy(pool_id, U128(buy_amt), 1, U128(0));
    contract.buy(pool_id, U128(buy_amt), 0, U128(0));
    contract.buy(pool_id, U128(buy_amt), 1, U128(0));

    // Switch context to Bob
    testing_env!(get_context(joiner.to_string(), 0));
    contract.join_pool(pool_id, U128(seed_amount));
    let joiner_pool_token_balance: u128 = contract.get_pool_token_balance(pool_id, &bob()).into();

    let expected_claimable_by_funder = to_token_denom(20);
    let claimable_by_funder = contract.get_fees_withdrawable(pool_id, funder);
    
    let claimable_by_leech = contract.get_fees_withdrawable(pool_id, joiner);

    contract.exit_pool(pool_id, U128(joiner_pool_token_balance));
    testing_env!(get_context(funder.to_string(), 0));
    contract.exit_pool(pool_id, U128(creator_pool_token_balance));

    assert_eq!(U128(expected_claimable_by_funder), claimable_by_funder);
    assert_eq!(claimable_by_leech, U128(0));

    let funder_pool_token_bal: u128 = contract.get_pool_token_balance(pool_id, funder).into();
    let joiner_pool_token_bal: u128 = contract.get_pool_token_balance(pool_id, joiner).into();

    assert_eq!(funder_pool_token_bal, 0);
    assert_eq!(joiner_pool_token_bal, 0);
}
