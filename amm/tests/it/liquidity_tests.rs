use crate::utils::*;
use near_sdk::json_types::{U128};
use near_sdk_sim::{to_yocto};

#[test]
fn add_liquidity_even_liq_test() {
    let test_utils = TestUtils::init(carol());
    
    // variables
    let market_id = 0;
    let creation_bond = 100;

    let seed_amount = to_yocto("10");
    let half = U128(to_yocto("5") / 10);
    let weights = Some(vec![half, half]);

    test_utils.alice.create_market(2, Some(U128(0)));
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);

    let pool_token_balance = test_utils.alice.get_pool_token_balance(market_id, None);
    assert_eq!(pool_token_balance, seed_amount);
    let seeder_balance = test_utils.alice.get_token_balance(None);
    assert_eq!(seeder_balance, init_balance() / 2 - seed_amount - creation_bond);
    let amm_collateral_balance = test_utils.alice.get_token_balance(Some(AMM_CONTRACT_ID.to_string()));
    assert_eq!(amm_collateral_balance, seed_amount);

    test_utils.bob.add_liquidity(market_id, seed_amount, None);

    let pool_token_balance = test_utils.bob.get_pool_token_balance(market_id, None);
    assert_eq!(pool_token_balance, seed_amount);
    let joiner_balance = test_utils.bob.get_token_balance(None);
    assert_eq!(joiner_balance, init_balance() / 2 - seed_amount);
    let amm_collateral_balance = test_utils.alice.get_token_balance(Some(AMM_CONTRACT_ID.to_string()));
    assert_eq!(amm_collateral_balance, seed_amount * 2);
}

#[test]
fn add_liquidity_uneven_liq_test() {
    let test_utils = TestUtils::init(carol());
    
    // variables
    let market_id = 0;
    
    let target_price_a = to_yocto("60") / 100;
    let target_price_b_c = to_yocto("20") / 100;
    let target_prices = vec![U128(target_price_a), U128(target_price_b_c), U128(target_price_b_c)];
    let weights = Some(calc_weights_from_price(target_prices));
    let seed_amount = to_yocto("100");
    
    test_utils.alice.create_market(3, Some(U128(0)));
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);

    let price_0 = test_utils.alice.get_spot_price_sans_fee(market_id, 0);
    let price_1 = test_utils.alice.get_spot_price_sans_fee(market_id, 1);
    let price_2 = test_utils.alice.get_spot_price_sans_fee(market_id, 2);

    assert_eq!(price_0, target_price_a);
    assert_eq!(price_1, target_price_b_c);
    assert_eq!(price_2, target_price_b_c);

    let pool_balances_after_seed = test_utils.alice.get_pool_balances(market_id);

    let outcome_balance_0 = test_utils.alice.get_outcome_balance(None, market_id, 0);
    let outcome_balance_1 = test_utils.alice.get_outcome_balance(None, market_id, 1);
    let outcome_balance_2 = test_utils.alice.get_outcome_balance(None, market_id, 2);

    assert_eq!(outcome_balance_0, seed_amount - pool_balances_after_seed[0]);
    assert_eq!(outcome_balance_1, 0);
    assert_eq!(outcome_balance_2, 0);

    let creator_pool_token_balance = test_utils.alice.get_pool_token_balance(market_id, None);


    test_utils.bob.add_liquidity(market_id, seed_amount, None);

    let outcome_balance_0 = test_utils.bob.get_outcome_balance(None, market_id, 0);
    assert_eq!(outcome_balance_0, seed_amount - pool_balances_after_seed[0]);

    let joiner_pool_token_balance = test_utils.bob.get_pool_token_balance(market_id, None);
    assert_eq!(creator_pool_token_balance, joiner_pool_token_balance);
}

#[test]
fn multiple_pool_exits_test() {
    let test_utils = TestUtils::init(carol());

    let market_id = 0;
    let seed_amount = to_yocto("10");
    let join_amount0 = to_yocto("50");
    let join_amount1 = to_yocto("30");
    
    let exit_amount0 = to_yocto("10");
    let exit_amount1 = to_yocto("20");
    let exit_amount2 = to_yocto("5");
    let exit_amount3 = to_yocto("3");

    let buy_amount = to_yocto("10");

    let half = U128(to_yocto("5") / 10);
    let weights = Some(vec![half, half]);
    
    test_utils.alice.create_market(2, Some(U128(0)));
    test_utils.bob.add_liquidity(market_id, seed_amount, weights);

    let pool_token_balance = test_utils.bob.get_pool_token_balance(market_id, None);
    assert_eq!(pool_token_balance, seed_amount);

    test_utils.alice.add_liquidity(market_id, join_amount0, None);
    test_utils.alice.buy(market_id, buy_amount, 0, 0);
    test_utils.alice.buy(market_id, buy_amount, 1, 0);

    test_utils.alice.add_liquidity(market_id, join_amount1, None);
    let alice_pool_token_balance_pre_exit = test_utils.alice.get_pool_token_balance(market_id, None);

    test_utils.alice.exit_liquidity(market_id, exit_amount0);
    test_utils.alice.exit_liquidity(market_id, exit_amount1);
    test_utils.alice.exit_liquidity(market_id, exit_amount2);
    test_utils.alice.exit_liquidity(market_id, exit_amount3);

    // assert pool balances
    let alice_pool_token_balance_post_exit = test_utils.alice.get_pool_token_balance(market_id, None);
    assert_eq!(alice_pool_token_balance_post_exit, alice_pool_token_balance_pre_exit - exit_amount0 - exit_amount1 -exit_amount2 - exit_amount3);
}

#[test]
fn join_zero_liq_test() {
    let test_utils = TestUtils::init(carol());

    let market_id = 0;
    let seed_amount = to_yocto("10");
    let join_amount = to_yocto("50");
    let half = U128(to_yocto("5") / 10);
    let weights = Some(vec![half, half]);
    
    test_utils.bob.create_market(2, Some(U128(0)));
    test_utils.bob.add_liquidity(market_id, seed_amount, weights);

    let pool_token_balance = test_utils.bob.get_pool_token_balance(market_id, None);
    assert_eq!(pool_token_balance, seed_amount);
    
    test_utils.bob.exit_liquidity(market_id, seed_amount);
    let pool_token_balance = test_utils.bob.get_pool_token_balance(market_id, None);
    assert_eq!(pool_token_balance, 0);
    
    test_utils.bob.add_liquidity(market_id, join_amount, None);
    let pool_token_balance = test_utils.bob.get_pool_token_balance(market_id, None);
    assert_eq!(pool_token_balance, 0);
}

#[test]
fn add_liquidity_redeem() {
    let test_utils = TestUtils::init(carol());

    let market_id = 0;
    let seed_amount = to_yocto("10");
    let half = U128(to_yocto("5") / 10);
    let weights = Some(vec![half, half]);
    
    test_utils.bob.create_market(2, Some(U128(0)));
    let balance_pre_lp = test_utils.bob.get_token_balance(None);
    test_utils.bob.add_liquidity(market_id, seed_amount, weights);

    let pool_token_balance = test_utils.bob.get_pool_token_balance(market_id, None);
    assert_eq!(pool_token_balance, seed_amount);
    
    
    test_utils.bob.exit_liquidity(market_id, seed_amount);
    let pool_token_balance = test_utils.bob.get_pool_token_balance(market_id, None);
    assert_eq!(pool_token_balance, 0);
    
    test_utils.bob.redeem_collateral(market_id, seed_amount);

    let collateral_balance = test_utils.bob.get_token_balance(None);
    assert_eq!(collateral_balance, balance_pre_lp);

    let outcome_balance_0 = test_utils.bob.get_outcome_balance(None, market_id, 0);
    let outcome_balance_1 = test_utils.bob.get_outcome_balance(None, market_id, 1);
    assert_eq!(outcome_balance_0, 0);
    assert_eq!(outcome_balance_1, 0);
}

#[test]
fn liquidity_exit_scenario() {
    let test_utils = TestUtils::init(carol());

    let market_id = 0;
    let seed_amount = 20000000000000000000;
    let weights = Some(vec![U128(70000000), U128(30000000)]);    

    test_utils.alice.create_market(2, Some(U128(0)));
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);

    test_utils.alice.buy(market_id, 100000000000000000, 0, 0);
    test_utils.alice.buy(market_id, 1000000000000000000000000, 0, 0);

    test_utils.alice.add_liquidity(market_id, 100000000000000000, None);
    test_utils.alice.add_liquidity(market_id, 1000000000000000000000000, None);
    
    let pool_token_balance = test_utils.alice.get_pool_token_balance(market_id, None);
    test_utils.alice.exit_liquidity(market_id, pool_token_balance);
}


#[test]
fn liquidity_exit_after_swap() {
    let test_utils = TestUtils::init(carol());

    let market_id = 0;
    let seed_amount = to_yocto("24");
    let buy_amount = to_yocto("5");
    let half = U128(to_yocto("5") / 10);
    let weights = Some(vec![half, half]);

    test_utils.bob.create_market(2, Some(U128(0)));
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);

    test_utils.alice.buy(market_id, buy_amount, 0, 0);
    let pool_token_balance = test_utils.alice.get_pool_token_balance(market_id, None);
    test_utils.alice.exit_liquidity(market_id, pool_token_balance);
}