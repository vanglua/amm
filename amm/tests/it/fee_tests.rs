use crate::utils::*;
use near_sdk::json_types::{U128};
use near_sdk_sim::{to_yocto};

#[test]
fn fee_valid_market_lp_fee_test() {
    let test_utils = TestUtils::init(carol());
    
    // variables
    let market_id = 0;

    let target_price = to_yocto("5") / 10;
    let seed_amount = to_yocto("100");
    let buy_amount = to_yocto("1");
    let weights = Some(calc_weights_from_price(vec![target_price, target_price]));
    let swap_fee = to_yocto("2") / 100;
    test_utils.alice.create_market(2, Some(U128(swap_fee)));
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);

    let payout_num = vec![U128(0), U128(to_yocto("1"))];

    test_utils.carol.buy(market_id, buy_amount, 0, 0);
    test_utils.carol.buy(market_id, buy_amount, 1, 0);
    test_utils.carol.buy(market_id, buy_amount, 0, 0);
    test_utils.carol.buy(market_id, buy_amount, 1, 0);
    test_utils.carol.buy(market_id, buy_amount, 0, 0);
    test_utils.carol.buy(market_id, buy_amount, 1, 0);
    test_utils.carol.buy(market_id, buy_amount, 0, 0);
    test_utils.carol.buy(market_id, buy_amount, 1, 0);
    test_utils.carol.buy(market_id, buy_amount, 0, 0);
    test_utils.carol.buy(market_id, buy_amount, 1, 0);
    
    test_utils.bob.add_liquidity(market_id, seed_amount, None);

    let expected_claimable_by_alice = to_yocto("2") / 10;
    let claimable_by_alice = test_utils.alice.get_fees_withdrawable(market_id, None);
    let claimable_by_bob = test_utils.bob.get_fees_withdrawable(market_id, None);
    assert_eq!(claimable_by_alice, expected_claimable_by_alice);
    assert_eq!(claimable_by_bob, 0);
}

#[test]
fn fee_invalid_market_lp_fee_test() {
    let test_utils = TestUtils::init(carol());
    
    // variables
    let market_id = 0;

    let target_price = to_yocto("5") / 10;
    let seed_amount = to_yocto("100");
    let buy_amount = to_yocto("1");
    let weights = Some(calc_weights_from_price(vec![target_price, target_price]));
    let swap_fee = to_yocto("2") / 100;
    test_utils.alice.create_market(2, Some(U128(swap_fee)));
    let alice_init_balance = test_utils.alice.get_token_balance(None);
    let bob_init_balance = test_utils.bob.get_token_balance(None);
    let carol_init_balance = test_utils.carol.get_token_balance(None);
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);

    let payout_num = vec![U128(0), U128(to_yocto("1"))];

    test_utils.carol.buy(market_id, buy_amount, 0, 0);
    test_utils.carol.buy(market_id, buy_amount, 1, 0);
    test_utils.carol.buy(market_id, buy_amount, 0, 0);
    test_utils.carol.buy(market_id, buy_amount, 1, 0);
    test_utils.carol.buy(market_id, buy_amount, 0, 0);
    test_utils.carol.buy(market_id, buy_amount, 1, 0);
    test_utils.carol.buy(market_id, buy_amount, 0, 0);
    test_utils.carol.buy(market_id, buy_amount, 1, 0);
    test_utils.carol.buy(market_id, buy_amount, 0, 0);
    test_utils.carol.buy(market_id, buy_amount, 1, 0);
    
    test_utils.carol.sell(market_id, buy_amount, 0, to_yocto("100"));
    test_utils.carol.sell(market_id, buy_amount, 0, to_yocto("100"));
    
    test_utils.bob.add_liquidity(market_id, seed_amount, None);

    let expected_claimable_by_alice = to_yocto("24") / 100;
    let claimable_by_alice = test_utils.alice.get_fees_withdrawable(market_id, None);
    let claimable_by_bob = test_utils.bob.get_fees_withdrawable(market_id, None);
    assert_eq!(claimable_by_alice, expected_claimable_by_alice);
    assert_eq!(claimable_by_bob, 0);

    let pool_token_balance_bob = test_utils.bob.get_pool_token_balance(market_id, None);

    test_utils.alice.exit_liquidity(market_id, seed_amount);
    test_utils.bob.exit_liquidity(market_id, pool_token_balance_bob);

    test_utils.carol.resolute_market(market_id, None);

    test_utils.bob.claim_earnings(market_id);
    test_utils.alice.claim_earnings(market_id);
    test_utils.carol.claim_earnings(market_id);
    
    let alice_final_balance = test_utils.alice.get_token_balance(None);
    let bob_final_balance = test_utils.bob.get_token_balance(None);
    let carol_final_balance = test_utils.carol.get_token_balance(None);
    let amm_final_balance = test_utils.carol.get_token_balance(Some(AMM_CONTRACT_ID.to_string()));
    
    let expected_alice_final_balance = alice_init_balance + u128::from(claimable_by_alice) - 1;
    let expected_bob_final_balance = bob_init_balance + 1;
    let expected_carol_final_balance = carol_init_balance - u128::from(claimable_by_alice);

    assert_eq!(alice_final_balance, expected_alice_final_balance);
    assert_eq!(bob_final_balance, expected_bob_final_balance);
    assert_eq!(carol_final_balance, expected_carol_final_balance);
    assert_eq!(amm_final_balance, 0);
}

#[test]
fn test_specific_fee_scenario() {
    let test_utils = TestUtils::init(carol());
    
    // variables
    let market_id = 0;

    let target_price = to_yocto("5") / 10;
    let seed_amount = to_yocto("10");
    let buy_amount = to_yocto("1");
    let weights = Some(calc_weights_from_price(vec![target_price, target_price]));
    let swap_fee = to_yocto("2") / 100;
    test_utils.alice.create_market(2, Some(U128(swap_fee)));
    let alice_init_balance = test_utils.alice.get_token_balance(None);
    let bob_init_balance = test_utils.bob.get_token_balance(None);
    let carol_init_balance = test_utils.carol.get_token_balance(None);
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);

    let fee_payed_t1 = to_yocto("2") / 100 + to_yocto("117") * 2 / 10000;
    let fee_payed_t2 = to_yocto("6") / 100;

    let expected_bob_balance = bob_init_balance - fee_payed_t1;
    let expected_carol_balance = carol_init_balance - fee_payed_t2;
    let expected_alice_balance = alice_init_balance + fee_payed_t1 + fee_payed_t2;

    let buy_amt_t1 = to_yocto("1");
    let buy_amt_t2 = to_yocto("3");

    test_utils.bob.buy(market_id, buy_amt_t1, 0, 0);
    test_utils.carol.buy(market_id, buy_amt_t2, 0, 0);

    test_utils.bob.sell(market_id, to_yocto("117") / 100, 0, to_yocto("100"));

    test_utils.carol.resolute_market(market_id, None);

    test_utils.bob.claim_earnings(market_id);
    test_utils.carol.claim_earnings(market_id);
    test_utils.alice.claim_earnings(market_id);
    
    let amm_bal = test_utils.bob.get_token_balance(Some(AMM_CONTRACT_ID.to_string()));
    let bob_bal = test_utils.bob.get_token_balance(None);
    let carol_bal = test_utils.carol.get_token_balance(None);
    let alice_bal = test_utils.alice.get_token_balance(None);

    assert_eq!(amm_bal, 0);
    assert_eq!(bob_bal, expected_bob_balance);
    assert_eq!(carol_bal, expected_carol_balance);
    assert_eq!(alice_bal, expected_alice_balance);
}