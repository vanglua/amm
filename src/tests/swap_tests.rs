use super::*;

#[test]
fn calc_buy_amount_test() {
    let context = get_context(alice(), 0);
    testing_env!(context);
    let mut contract = PoolFactory::init(alice());

    let pool_id = contract.new_pool(2, U128(0));
    let half = to_token_denom(1) / 2;

    contract.seed_pool(pool_id, U128(to_token_denom(10)), vec![U128(half), U128(half)]);
    
    let amt: u128 = contract.calc_buy_amount(pool_id, U128(to_token_denom(1)), 0).into();
    let expected_amt = 1_909_090_909_090_909_091;

    assert_eq!(amt, expected_amt);
}

#[test]
fn calc_sell_collateral_out_test() {
    let context = get_context(alice(), 0);
    testing_env!(context);
    let mut contract = PoolFactory::init(alice());

    let pool_id = contract.new_pool(2, U128(0));
    let half = to_token_denom(1) / 2;

    contract.seed_pool(pool_id, U128(to_token_denom(10)), vec![U128(half), U128(half)]);
    
    let amt: u128 = contract.calc_sell_collateral_out(pool_id, U128(to_token_denom(1)), 0).into();
    let expected_amt = 2111111111111111111;

    assert_eq!(amt, expected_amt);
}


// #[test]
// fn buy_test() {
//     let context = get_context(alice(), 0);
//     testing_env!(context);
//     let mut contract = PoolFactory::init(alice());

//     let pool_id = contract.new_pool(2, U128(0));
//     let half = to_token_denom(1) / 2;

//     contract.bind_pool(pool_id, U128(to_token_denom(10)), vec![U128(half), U128(half)]);

//     contract.buy(pool_id, U128(to_token_denom(1)), 0, U128(to_token_denom(15) / 10));
// }