mod test_utils;
use test_utils::*;
use near_sdk::json_types::{U64, U128};
use near_sdk::serde_json::json;
use near_sdk_sim::{to_yocto, view};

#[test]
fn pool_initial_pricing_test() {
    let (_master_account, amm, token, alice, _bob, _carol) = init(to_yocto("1"), "carol".to_string());
    let seed_amount = to_token_denom(20);
    let half = to_token_denom(5) / 10;

    let market_id = create_market(&alice, &amm, 2, Some(U128(0)));

    assert_eq!(market_id, U64(0));
    let weights = Some(vec![U128(half), U128(half)]);

    let add_liquidity_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
            "weight_indication": weights
        }
    }).to_string();
    transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, add_liquidity_args);

    let even_price: U128 = view!(amm.get_spot_price_sans_fee(market_id, 0)).unwrap_json();
    assert_eq!(u128::from(even_price), half);
    
    let buy_amt: U128 = view!(amm.calc_buy_amount(market_id, U128(to_token_denom(10)), 0)).unwrap_json();

}