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
    // TODO: Calc expected liquidity / pool token minting
    // TODO: Create uneven pool, join the pool
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

// TODO: Add even pool and rebalance with swaps, then add liquidity.