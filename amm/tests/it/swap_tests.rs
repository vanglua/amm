use crate::utils::*;
use near_sdk::json_types::{U128};
use near_sdk_sim::{to_yocto};

#[test]
fn swap_calc_buy_amount_test() {
    let test_utils = TestUtils::init(carol());

    let market_id = 0;
    let seed_amount = to_yocto("10");

    let half = to_yocto("5") / 10;
    let weights = Some(vec![U128(half), U128(half)]);

    test_utils.alice.create_market(2, Some(U128(0)));
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);

    let buy_amount = test_utils.alice.calc_buy_amount(market_id, 0, to_yocto("1"));
    assert_eq!(buy_amount, 1909090909090909090909091);
}

#[test]
fn swap_calc_sell_collateral_out_test() {
    let test_utils = TestUtils::init(carol());

    let market_id = 0;
    let seed_amount = to_yocto("10");

    let half = to_yocto("5") / 10;
    let weights = Some(vec![U128(half), U128(half)]);

    test_utils.alice.create_market(2, Some(U128(0)));
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);

    let buy_amount = test_utils.alice.calc_sell_amount(market_id, 0, to_yocto("1"));
    assert_eq!(buy_amount, 2111111111111111111111111);
}

#[test]
fn swap_basic_buy_test() {
    let test_utils = TestUtils::init(carol());

    let market_id = 0;
    let seed_amount = to_yocto("10");
    let buy_amount = to_yocto("1");
    let invariant = to_yocto("100");
    let half = to_yocto("5") / 10;
    let weights = Some(vec![U128(half), U128(half)]);

    test_utils.alice.create_market(2, Some(U128(0)));
    let init_balance_alice = test_utils.alice.get_token_balance(None);
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);


    test_utils.alice.buy(market_id, buy_amount, 0, 0);
    
    let balance_alice = test_utils.alice.get_token_balance(None);
    assert_eq!(balance_alice, init_balance_alice - seed_amount - buy_amount);
    let balance_amm = test_utils.alice.get_token_balance(Some(AMM_CONTRACT_ID.to_string()));
    assert_eq!(balance_amm, seed_amount + buy_amount);

    let expected_target_pool_balance = invariant / 11;
    let expected_other_pool_balance = seed_amount + buy_amount;
    
    let pool_balances = test_utils.alice.get_pool_balances(market_id);

    assert_eq!(pool_balances[0], expected_target_pool_balance);
    assert_eq!(pool_balances[1], expected_other_pool_balance);

    let expected_target_buyer_balance = seed_amount + buy_amount - expected_target_pool_balance;
    let target_buyer_balance = test_utils.alice.get_outcome_balance(None, market_id, 0);

    assert_eq!(expected_target_buyer_balance, target_buyer_balance);
}

#[test]
fn swap_basic_sell_test() {
    let test_utils = TestUtils::init(carol());

    let market_id = 0;
    let seed_amount = to_yocto("10");
    let buy_amount = to_yocto("1");
    let invariant = to_yocto("100");
    let half = to_yocto("5") / 10;
    let weights = Some(vec![U128(half), U128(half)]);

    test_utils.alice.create_market(2, Some(U128(0)));
    let init_balance_alice = test_utils.alice.get_token_balance(None);
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);


    test_utils.alice.buy(market_id, buy_amount, 0, 0);
    
    let balance_alice = test_utils.alice.get_token_balance(None);
    assert_eq!(balance_alice, init_balance_alice - seed_amount - buy_amount);
    let balance_amm = test_utils.alice.get_token_balance(Some(AMM_CONTRACT_ID.to_string()));
    assert_eq!(balance_amm, seed_amount + buy_amount);

    let expected_target_pool_balance = invariant / 11;
    let expected_other_pool_balance = seed_amount + buy_amount;
    
    let pool_balances = test_utils.alice.get_pool_balances(market_id);

    assert_eq!(pool_balances[0], expected_target_pool_balance);
    assert_eq!(pool_balances[1], expected_other_pool_balance);

    let expected_target_buyer_balance = seed_amount + buy_amount - expected_target_pool_balance;
    let target_buyer_balance = test_utils.alice.get_outcome_balance(None, market_id, 0);

    assert_eq!(expected_target_buyer_balance, target_buyer_balance);

    test_utils.alice.sell(market_id, to_yocto("1"), 0, to_yocto("100"));

    let pool_balances = test_utils.alice.get_pool_balances(market_id);

    assert_eq!(pool_balances[0], seed_amount);
    assert_eq!(pool_balances[1], seed_amount);

    let expected_alice_balance_post = init_balance_alice - seed_amount;
    let alice_balance_post = test_utils.alice.get_token_balance(None);
    let amm_balance_post = test_utils.alice.get_token_balance(Some(AMM_CONTRACT_ID.to_string()));

    assert_eq!(expected_alice_balance_post, alice_balance_post);
    assert_eq!(amm_balance_post, seed_amount);
}

#[test]
fn swap_complex_buy_test() {
    let test_utils = TestUtils::init(carol());

    let market_id = 0;
    let seed_amount = to_yocto("10");
    let buy_amount = to_yocto("1");
    let weights = Some(
        calc_weights_from_price(
            vec![
                to_yocto("60"), 
                to_yocto("30"),
                to_yocto("10")
                ]
            )
        );

    test_utils.alice.create_market(3, Some(U128(0)));
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);

    let init_pool_balances = test_utils.alice.get_pool_balances(market_id);
    let init_invariant = product_of(&init_pool_balances);
    
    test_utils.bob.buy(market_id, buy_amount, 0, 0);

    let post_trade_pool_balances = test_utils.alice.get_pool_balances(market_id);
    let post_trade_invariant = product_of(&post_trade_pool_balances);
    assert!(init_invariant - post_trade_invariant <  1000);

    let target_pool_balance = post_trade_pool_balances[0];
    let target_buyer_balance = test_utils.bob.get_outcome_balance(None, market_id, 0);
    let inverse_balances = vec![post_trade_pool_balances[1], post_trade_pool_balances[2]];
    let product_of_inverse = product_of(&inverse_balances);

    let expected_pool_target_balance = math::complex_div_u128(to_yocto("1"), post_trade_invariant, product_of_inverse);
    let expected_buyer_target_balance = init_pool_balances[0] + buy_amount - expected_pool_target_balance;

    assert_eq!(expected_buyer_target_balance, target_buyer_balance);
    assert_eq!(expected_pool_target_balance, target_pool_balance);
}

#[test]
fn swap_multi_sell_test() {
    let test_utils = TestUtils::init(carol());

    let market_id = 0;
    let seed_amount = to_yocto("10");
    let buy_amount = to_yocto("1");
    let precision = to_yocto("1") / 100; // 1 "token_cent" precision

    let expected_bob_share_bal = 1909090909090909090909091_u128;
    let expected_carol_share_bal = 1757575757575757575757576_u128;
    let expected_carol_final_bal = 499911500000000000000000000_u128;

    let half = U128(to_yocto("5") / 10);
    let weights = Some(vec![half, half]);

    test_utils.alice.create_market(2, Some(U128(0)));
    let bob_initial_balance = test_utils.bob.get_token_balance(None);
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);

    test_utils.bob.buy(market_id, buy_amount, 0, 0);
    test_utils.carol.buy(market_id, buy_amount, 0, 0);

    let bob_share_balance = test_utils.bob.get_outcome_balance(None, market_id, 0);
    let carol_share_balance = test_utils.carol.get_outcome_balance(None, market_id, 0);

    assert_eq!(bob_share_balance, expected_bob_share_bal);
    assert_eq!(carol_share_balance, expected_carol_share_bal);

    test_utils.bob.sell(market_id, buy_amount + buy_amount / 13, 0, to_yocto("100"));
    test_utils.carol.sell(market_id, to_yocto("9115") / 10000, 0, to_yocto("100"));
    
    let bob_balance = test_utils.bob.get_token_balance(None);
    let carol_balance = test_utils.carol.get_token_balance(None);

    assert!(bob_initial_balance - bob_balance < precision);
    println!("{:?}", carol_balance);
    assert!(expected_carol_final_bal - carol_balance < precision);
}

#[test]
fn swap_selling_uneven_lp_shares_binary_test() {
    let test_utils = TestUtils::init(carol());

    let market_id = 0;
    let seed_amount = to_yocto("10");

    let amount_out_expected = to_yocto("932") / 1000;
    let balance_after_seed = 3_333_333_333_333_333_333_333_333_333_u128;

    let weights = Some(calc_weights_from_price(vec![to_yocto("55") / 100, to_yocto("45") / 100]));

    test_utils.alice.create_market(2, Some(U128(0)));
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);

    test_utils.alice.sell(market_id, amount_out_expected, 0, balance_after_seed);
}

#[test]
fn swap_selling_uneven_lp_shares_categorical_test() {
    let test_utils = TestUtils::init(carol());

    let market_id = 0;
    let seed_amount = to_yocto("10");
    let amount_out_expected = 838_054_961_715_504_818;
    let balance_after_seed = 3_333_333_333_333_333_333;
    let weights = Some(vec![U128(12_000_000_000), U128(12_000_000_000), U128(18_000_000_000), U128(18_000_000_000)]);

    test_utils.alice.create_market(4, Some(U128(0)));
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);

    test_utils.alice.sell(market_id, amount_out_expected, 0, balance_after_seed);
}

fn redeem_collat_helper(target_price_a: u128, target_price_b: u128, token_value_80_20: u128) {
    let test_utils = TestUtils::init(carol());

    let market_id = 0;
    let seed_amount = to_yocto("10");
    let buy_amount = to_yocto("1");

    let weights = Some(calc_weights_from_price(vec![target_price_a, target_price_b]));
    test_utils.alice.create_market(2, Some(U128(0)));
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);

    test_utils.bob.buy(market_id, buy_amount, 0, 0);
    let expected_target_buyer_balance = token_value_80_20;
    let expected_other_buyer_balance = 0;

    let target_buyer_balance = test_utils.bob.get_outcome_balance(None, market_id, 0);
    let other_buyer_balance = test_utils.bob.get_outcome_balance(None, market_id, 1);

    assert_eq!(expected_target_buyer_balance, u128::from(target_buyer_balance));
    assert_eq!(expected_other_buyer_balance, u128::from(other_buyer_balance));

    test_utils.alice.exit_liquidity(market_id, seed_amount);

    // add liquidity with unequal weights reversed
    let weights = Some(calc_weights_from_price(vec![target_price_b, target_price_a]));
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);

    test_utils.bob.buy(market_id, buy_amount, 1, 0);
    let expected_target_buyer_balance = token_value_80_20;
    let expected_other_buyer_balance = token_value_80_20;

    let target_buyer_balance = test_utils.bob.get_outcome_balance(None, market_id, 0);
    let other_buyer_balance = test_utils.bob.get_outcome_balance(None, market_id, 1);

    assert_eq!(expected_target_buyer_balance, target_buyer_balance);
    assert_eq!(expected_other_buyer_balance, other_buyer_balance);

    let pre_redeem_balance = test_utils.bob.get_token_balance(None);
    // Redeem liquidity
    test_utils.bob.redeem_collateral(market_id, token_value_80_20);

    // Assert collateral balance
    let expected_collateral_balance = std::cmp::min(999999999999999999999999998, u128::from(pre_redeem_balance)  + token_value_80_20);
    let collateral_balance: u128 = test_utils.bob.get_token_balance(None);
    assert_eq!(collateral_balance, expected_collateral_balance);

    test_utils.alice.exit_liquidity(market_id, seed_amount);

    test_utils.carol.resolute_market(market_id, None);
    test_utils.bob.claim_earnings(market_id);
    test_utils.alice.claim_earnings(market_id);
}

#[test]
fn redeem_collat_with_bought_tokens_for_higher_price() {
    let token_value_80_20 = 1227272727272727272727273;
    let target_price_a = to_yocto("80") / 100;
    let target_price_b = to_yocto("20") / 100;
    // bob bought 2 times, and redeemed 1.22 again (loss of 0.8 tokens)
    redeem_collat_helper(target_price_a, target_price_b, token_value_80_20);
}