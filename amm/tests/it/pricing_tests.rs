use crate::utils::*;
use near_sdk::json_types::{U128};
use near_sdk_sim::{to_yocto};

#[test]
fn pool_initial_pricing_test() {
    let test_utils = TestUtils::init(carol());

    let market_id_0 = 0;
    let market_id_1 = 1;
    let seed_amount = to_yocto("100");
    let half = to_yocto("5") / 10;
    let forty = to_yocto("4") / 10;
    let sixty = to_yocto("6") / 10;
    let even_weights = Some(vec![U128(half), U128(half)]);
    let uneven_weights = Some(vec![U128(forty), U128(sixty)]);

    test_utils.alice.create_market(2, Some(U128(0)));
    test_utils.alice.add_liquidity(market_id_0, seed_amount, even_weights);

    let price_0 = test_utils.alice.get_spot_price_sans_fee(market_id_0, 0);
    assert_eq!(price_0, half);
    
    test_utils.alice.create_market(2, Some(U128(0)));
    test_utils.alice.add_liquidity(market_id_1, seed_amount, uneven_weights);

    let price_0 = test_utils.alice.get_spot_price_sans_fee(market_id_1, 0);
    let price_1 = test_utils.alice.get_spot_price_sans_fee(market_id_1, 1);
    assert_eq!(price_0, sixty);
    assert_eq!(price_1, forty);
}

#[test]
fn pricing_multi_outcome_pool_test() {
    let test_utils = TestUtils::init(carol());

    let market_id = 0;
    let seed_amount = to_yocto("100");
    let third = to_yocto("1") / 3;
    let twenty = to_yocto("2") / 10;
    let sixty = to_yocto("6") / 10;
    let even_weights = Some(vec![U128(third), U128(third), U128(third + 1)]);

    test_utils.alice.create_market(3, Some(U128(0)));
    test_utils.alice.add_liquidity(market_id, seed_amount, even_weights);

    let price_0 = test_utils.alice.get_spot_price_sans_fee(market_id, 1);
    assert_eq!(price_0, 333333333333333333333334);

    test_utils.alice.exit_liquidity(market_id, seed_amount);

    let uneven_weights = Some(vec![U128(twenty), U128(twenty), U128(sixty)]);
    test_utils.alice.add_liquidity(market_id, seed_amount, uneven_weights);

    let bal_0 = math::complex_mul_u128(to_yocto("1"), twenty, to_yocto("1"));
    let bal_1 = math::complex_mul_u128(to_yocto("1"), twenty, to_yocto("1"));
    let bal_2 = math::complex_mul_u128(to_yocto("1"), sixty, to_yocto("1"));

    let odds_weight_0 = math::complex_mul_u128(to_yocto("1"), bal_1, bal_2);
    let odds_weight_1 = math::complex_mul_u128(to_yocto("1"), bal_0, bal_2);
    let odds_weight_2 = math::complex_mul_u128(to_yocto("1"), bal_0, bal_1);
    let odds_weight_sum = odds_weight_0 + odds_weight_1 + odds_weight_2;

    let expected_mp_0 = math::complex_div_u128(to_yocto("1"), odds_weight_0, odds_weight_sum);
    let expected_mp_1 = math::complex_div_u128(to_yocto("1"), odds_weight_1, odds_weight_sum);
    let expected_mp_2 = math::complex_div_u128(to_yocto("1"), odds_weight_2, odds_weight_sum);

    let wrapped_price_0 = test_utils.alice.get_spot_price_sans_fee(market_id, 0);
    let wrapped_price_1 = test_utils.alice.get_spot_price_sans_fee(market_id, 1);
    let wrapped_price_2 = test_utils.alice.get_spot_price_sans_fee(market_id, 2);

    let price_0: u128 = wrapped_price_0.into();
    let price_1: u128 = wrapped_price_1.into();
    let price_2: u128 = wrapped_price_2.into();

    assert!(to_yocto("1") - (price_0 + price_1 + price_2) < 100_000);

    assert_eq!(price_0, 428571428571428571428571);
    assert_eq!(price_1, 428571428571428571428571);
    assert_eq!(price_2, 142857142857142857142857);

    assert_eq!(price_0, expected_mp_0);
    assert_eq!(price_1, expected_mp_1);
    assert_eq!(price_2, expected_mp_2);
}

#[test]
fn pricing_fee_test_calc() {
    let test_utils = TestUtils::init(carol());
    let market_id = 0;

    let seed_amount = to_yocto("100");
    let half = to_yocto("5") / 10;
    let weights = Some(vec![U128(half), U128(half)]);

    test_utils.alice.create_market(2, Some(fee()));
    test_utils.alice.add_liquidity(market_id, seed_amount, weights);

    let price = test_utils.alice.get_spot_price(market_id, 0);
    let swap_fee: u128 = fee().into();

    let scale = math::complex_div_u128(to_yocto("1"), to_yocto("1"), to_yocto("1") - swap_fee);
    let scaled_half = math::complex_mul_u128( to_yocto("1"), half, scale);

    assert_eq!(price, scaled_half);
}