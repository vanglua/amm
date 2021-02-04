mod test_utils;
use test_utils::*;
use near_sdk::json_types::{U64, U128};
use near_sdk::serde_json::json;
use near_sdk_sim::{to_yocto, call, view, STORAGE_AMOUNT};

#[test]
fn pool_initial_pricing_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "alice".to_string(), "carol".to_string());
    let seed_amount = to_token_denom(100);
    let half = to_token_denom(5) / 10;
    let forty = to_token_denom(4) / 10;
    let sixty = to_token_denom(6) / 10;

    let market_id = create_market(&alice, &amm, 2, Some(U128(0)));

    assert_eq!(market_id, U64(0));

    let even_seed_pool_res = call!(
        alice,
        amm.seed_pool(market_id, U128(seed_amount), vec![U128(half), U128(half)]),
        deposit = STORAGE_AMOUNT
    );

    let even_price: U128 = view!(amm.get_spot_price_sans_fee(market_id, 0)).unwrap_json();
    assert_eq!(u128::from(even_price), half);
    
    let uneven_seed_pool_res = call!(
        alice,
        amm.seed_pool(market_id, U128(seed_amount), vec![U128(forty), U128(sixty)]),
        deposit = STORAGE_AMOUNT
    );

    let expected_0 = to_token_denom(6) / 10;
    let expected_1 = to_token_denom(4) / 10;

    let price_0: U128 = view!(amm.get_spot_price_sans_fee(market_id, 0)).unwrap_json();
    assert_eq!(u128::from(price_0), expected_0);
    let price_1: U128 = view!(amm.get_spot_price_sans_fee(market_id, 1)).unwrap_json();
    assert_eq!(u128::from(price_1), expected_1);
}

#[test]
fn multi_outcome_pool_pricing_test() {
    // Even pool
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "alice".to_string(), "carol".to_string());
    let seed_amount = to_token_denom(100);
    
    let market_id = create_market(&alice, &amm, 3, Some(U128(0)));
    
    let third = to_token_denom(1) / 3;
    
    let res = call!(
        alice,
        amm.seed_pool(market_id, U128(seed_amount), vec![U128(third), U128(third), U128(third + 1)]),
        deposit = STORAGE_AMOUNT
    );
    
    let even_price: U128 = view!(
        amm.get_spot_price_sans_fee(market_id, 1)
    ).unwrap_json();

    assert_eq!(even_price, U128(333_333_333_333_333_334));
    
    // Uneven pool
    let twenty = to_token_denom(2) / 10;
    let sixty = to_token_denom(6) / 10;
    let collat = to_token_denom(100);
    
    call!(
        alice,
        amm.seed_pool(market_id, U128(seed_amount), vec![U128(twenty), U128(twenty), U128(sixty)]),
        deposit = STORAGE_AMOUNT
    );

    let bal_0 = test_utils::math::mul_u128(twenty, collat);
    let bal_1 = test_utils::math::mul_u128(twenty, collat);
    let bal_2 = test_utils::math::mul_u128(sixty, collat);

    let odds_weight_0 = test_utils::math::mul_u128(bal_1, bal_2);
    let odds_weight_1 = test_utils::math::mul_u128(bal_0, bal_2);
    let odds_weight_2 = test_utils::math::mul_u128(bal_0, bal_1);
    let odds_weight_sum = odds_weight_0 + odds_weight_1 + odds_weight_2;

    let expected_mp_0 = test_utils::math::div_u128(odds_weight_0, odds_weight_sum);
    let expected_mp_1 = test_utils::math::div_u128(odds_weight_1, odds_weight_sum);
    let expected_mp_2 = test_utils::math::div_u128(odds_weight_2, odds_weight_sum);

    let wrapped_price_0: U128 = view!(amm.get_spot_price_sans_fee(market_id, 0)).unwrap_json();
    let wrapped_price_1: U128 = view!(amm.get_spot_price_sans_fee(market_id, 1)).unwrap_json();
    let wrapped_price_2: U128 = view!(amm.get_spot_price_sans_fee(market_id, 2)).unwrap_json();

    let price_0: u128 = wrapped_price_0.into();
    let price_1: u128 = wrapped_price_1.into();
    let price_2: u128 = wrapped_price_2.into();

    assert!(to_token_denom(1) - (price_0 + price_1 + price_2) < 100_000);

    assert_eq!(price_0, 428_571_428_571_428_571);
    assert_eq!(price_1, 428_571_428_571_428_571);
    assert_eq!(price_2, 142_857_142_857_142_857);

    assert_eq!(price_0, expected_mp_0);
    assert_eq!(price_1, expected_mp_1);
    assert_eq!(price_2, expected_mp_2);
}

#[test]
fn fee_test_calc() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "alice".to_string(), "carol".to_string());

    let half = to_token_denom(1) / 2;
    let seed_amount = to_token_denom(100);
    
    let market_id = create_market(&alice, &amm, 2, Some(U128(0)));

    call!(
        alice,
        amm.seed_pool(market_id, U128(seed_amount), vec![U128(half), U128(half)]),
        deposit = STORAGE_AMOUNT
    );
    
    let even_price_wrapped: U128 = view!(amm.get_spot_price_sans_fee(market_id, 1)).unwrap_json();
    let swap_fee_wrapped: U128 = view!(amm.get_pool_swap_fee(market_id)).unwrap_json();
    
    let even_price: u128 = even_price_wrapped.into();
    let swap_fee: u128 = swap_fee_wrapped.into();

    let scale = test_utils::math::div_u128(to_token_denom(1), to_token_denom(1) - swap_fee);
    let half_plus_fee = test_utils::math::mul_u128(half, scale);

    assert_eq!(even_price, half_plus_fee);

}