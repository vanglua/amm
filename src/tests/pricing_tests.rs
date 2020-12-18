use super::*;
use crate::math;

#[test]
fn pool_initial_pricing_test() {
    let context = get_context(alice(), 0);
    testing_env!(context);
    let mut contract = PoolFactory::init(alice());

    let pool_id = contract.new_pool(2, swap_fee());
    let half = to_token_denom(5) / 10;

    contract.seed_pool(pool_id, U128(to_token_denom(100)), vec![U128(half), U128(half)]);
    
    let even_price: u128 = contract.get_spot_price_sans_fee(pool_id, 0).into();
    assert_eq!(even_price, half);


    let forty = to_token_denom(4) / 10;
    let sixty = to_token_denom(6) / 10;
    
    contract.seed_pool(pool_id, U128(to_token_denom(100)), vec![U128(forty), U128(sixty)]);

    let expected_0 = 545_454_545_454_545_455;
    let expected_1 = 454_545_454_545_454_545;

    let price_0: u128 = contract.get_spot_price_sans_fee(pool_id, 0).into();
    let price_1: u128 = contract.get_spot_price_sans_fee(pool_id, 1).into();
    
    assert_eq!(expected_0, price_0);
    assert_eq!(expected_1, price_1);
}


#[test]
fn multi_outcome_pool_pricing_test() {
    // Even pool
    let context = get_context(alice(), 0);
    testing_env!(context);
    let mut contract = PoolFactory::init(alice());

    let pool_id = contract.new_pool(3, swap_fee());
    let third = to_token_denom(1) / 3;

    contract.seed_pool(pool_id, U128(to_token_denom(100)), vec![U128(third), U128(third), U128(third + 1)]);
    
    let even_price: u128 = contract.get_spot_price_sans_fee(pool_id, 1).into();
    assert_eq!(even_price, 333333333333333334);
    
    // Uneven pool
    let twenty = to_token_denom(2) / 10;
    let sixty = to_token_denom(6) / 10;
    let collat = to_token_denom(100);
    contract.seed_pool(pool_id, U128(collat), vec![U128(twenty), U128(twenty), U128(sixty)]);
    
    let bal_0 = math::mul_u128(twenty, collat);
    let bal_1 = math::mul_u128(twenty, collat);
    let bal_2 = math::mul_u128(sixty, collat);

    let odds_weight_0 = math::mul_u128(bal_1, bal_2);
    let odds_weight_1 = math::mul_u128(bal_0, bal_2);
    let odds_weight_2 = math::mul_u128(bal_0, bal_1);
    let odds_weight_sum = odds_weight_0 + odds_weight_1 + odds_weight_2;

    let expected_mp_0 = math::div_u128(odds_weight_0, odds_weight_sum);
    let expected_mp_1 = math::div_u128(odds_weight_1, odds_weight_sum);
    let expected_mp_2 = math::div_u128(odds_weight_2, odds_weight_sum);

    let price_0: u128 = contract.get_spot_price_sans_fee(pool_id, 0).into();
    let price_1: u128 = contract.get_spot_price_sans_fee(pool_id, 1).into();
    let price_2: u128 = contract.get_spot_price_sans_fee(pool_id, 2).into();

    assert!(u128::from(to_token_denom(1)) - (price_0 + price_1 + price_2) < 100000);

    assert_eq!(expected_mp_0, 428571428571428571);
    assert_eq!(expected_mp_1, 428571428571428571);
    assert_eq!(expected_mp_2, 142857142857142857);
}

#[test]
fn fee_test_calc() {
    let context = get_context(alice(), 0);
    testing_env!(context);
    let mut contract = PoolFactory::init(alice());

    let pool_id = contract.new_pool(2, swap_fee());
    let half = to_token_denom(1) / 2;

    contract.seed_pool(pool_id, U128(to_token_denom(100)), vec![U128(half), U128(half)]);
    
    let even_price: u128 = contract.get_spot_price(pool_id, 0).into();

    let swap_fee: u128 = contract.get_pool_swap_fee(pool_id).into();
    let scale = math::div_u128(to_token_denom(1), to_token_denom(1) - swap_fee);
    let half_plus_fee = math::mul_u128(half, scale);

    assert_eq!(even_price, half_plus_fee);

}