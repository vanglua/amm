// mod test_utils;
use crate::utils::*;
use near_sdk::json_types::U128;
use near_sdk_sim::{to_yocto };

#[test]
fn multi_lp_payout_no_exit() {
    let test_utils = TestUtils::init(carol());
    let market_id = 0;
    let seed_amount_0 = to_yocto("100");
    let seed_amount_1 = 200000000000000000000000;
    let seed_amount_2 = to_yocto("1");
    let buy_amount = to_yocto("1") / 10;

    let target_price_0 = to_yocto("9999") / 10000;
    let target_price_1 = to_yocto("1") / 100;
    let weights = calc_weights_from_price(vec![target_price_1, target_price_0]);

    test_utils.alice.create_market(2, Some(U128(0)));
    test_utils.bob.add_liquidity(market_id, seed_amount_0, Some(weights));

    test_utils.bob.buy(market_id, buy_amount, 0, 0);
    test_utils.bob.buy(market_id, buy_amount, 0, 0);
    test_utils.bob.buy(market_id, buy_amount, 0, 0);

    test_utils.alice.add_liquidity(market_id, seed_amount_1, None);
    
    test_utils.alice.buy(market_id, buy_amount, 0, 0);
    test_utils.alice.add_liquidity(market_id, seed_amount_2, None);
    test_utils.alice.buy(market_id, buy_amount, 0, 0);

    test_utils.carol.resolute_market(market_id, Some(vec![U128(0), U128(to_yocto("1"))]));
    test_utils.alice.claim_earnings(market_id);
    test_utils.bob.claim_earnings(market_id);

    let amm_final_balance = test_utils.alice.get_token_balance(Some(AMM_CONTRACT_ID.to_string()));
    assert_eq!(amm_final_balance, 0);   
}