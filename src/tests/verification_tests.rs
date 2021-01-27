use super::*;
use crate::math;

#[test]
fn pool_initial_pricing_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "alice".to_string(), "carol".to_string());
    let seed_amount = to_token_denom(20);
    let half = to_token_denom(5) / 10;

    let market_id = create_market(&alice, &amm, 2, Some(U128(0)));

    assert_eq!(market_id, U64(0));

    let even_seed_pool_res = call!(
        alice,
        amm.seed_pool(market_id, U128(seed_amount), vec![U128(half), U128(half)]),
        deposit = STORAGE_AMOUNT
    );

    let even_price: U128 = view!(amm.get_spot_price_sans_fee(market_id, 0)).unwrap_json();
    assert_eq!(u128::from(even_price), half);
    
    let buy_amt: U128 = view!(amm.calc_buy_amount(market_id, U128(to_token_denom(10)), 0)).unwrap_json();

}