use crate::utils::*;
use near_sdk::json_types::{U128};
use near_sdk_sim::{to_yocto};

#[test]
fn test_uneven_lp_shares_solvency_tests() {
    let test_utils = TestUtils::init(carol());

    let market_id = 0;
    let seed_amount = to_yocto("10");
    let buy_amount = to_yocto("1");
    
    let weight_0 = to_yocto("3") / 10;
    let weight_1 = to_yocto("7") / 10;
    let weights = calc_weights_from_price(vec![weight_0, weight_1]);
    
    test_utils.alice.create_market(2, Some(U128(0)));
    let alice_init_balance = test_utils.alice.get_token_balance(None);
    let bob_init_balance = test_utils.bob.get_token_balance(None);
    let carol_init_balance = test_utils.carol.get_token_balance(None);
    
    test_utils.alice.add_liquidity(market_id, seed_amount, Some(weights));

    test_utils.alice.buy(market_id, buy_amount, 0, 0);
    test_utils.bob.buy(market_id, buy_amount, 0, 0);
    test_utils.carol.buy(market_id, buy_amount, 0, 0);

    test_utils.alice.sell(market_id, to_yocto("25") / 100, 0, to_yocto("100")); 

    test_utils.carol.resolute_market(market_id, None);
    test_utils.bob.claim_earnings(market_id);
    test_utils.carol.claim_earnings(market_id);
    test_utils.alice.claim_earnings(market_id);
    
    let alice_final_balance = test_utils.alice.get_token_balance(None);
    let bob_final_balance = test_utils.bob.get_token_balance(None);
    let carol_final_balance = test_utils.carol.get_token_balance(None);
    let amm_final_balance = test_utils.alice.get_token_balance(Some(AMM_CONTRACT_ID.to_string()));
        
    // Assert that all balances are back to where they started
    assert_eq!(alice_final_balance, alice_init_balance);
    assert_eq!(bob_final_balance, bob_init_balance);
    assert_eq!(carol_final_balance, carol_init_balance);
    assert_eq!(amm_final_balance, 0);
}