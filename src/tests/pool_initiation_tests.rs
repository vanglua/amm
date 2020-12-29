use super::*;

#[test]
fn pool_initial_state_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "alice".to_string());

    let res: U64 = call!(
        alice,
        amm.new_pool(2, U128(0)),
        deposit = STORAGE_AMOUNT
    ).unwrap_json();

    assert_eq!(res, U64(0));
}


// #[test]
// fn pool_binding_test() {
//     let context = get_context(alice(), 0);
//     testing_env!(context);
//     let mut contract = PoolFactory::init(alice());

//     let pool_id = contract.new_pool(2, swap_fee());
//     let half = to_token_denom(5) / 10;

//     contract.seed_pool(pool_id, U128(to_token_denom(100)), vec![U128(half), U128(half)]);
//     let pool_token_balance = contract.get_pool_token_balance(pool_id, &alice());
//     assert_eq!(pool_token_balance, U128(to_token_denom(100)));
// }

