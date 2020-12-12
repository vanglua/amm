use super::*;

// use crate::constants::{
//     // INIT_POOL_SUPPLY
// };

#[test]
fn pool_initial_state_test() {
    let context = get_context(alice(), 0);
    testing_env!(context);
    let mut contract = PoolFactory::init(alice());

    let pool_id = contract.new_pool(2, swap_fee());

    assert_eq!(pool_id, U64(0));
}


#[test]
fn pool_binding_test() {
    let context = get_context(alice(), 0);
    testing_env!(context);
    let mut contract = PoolFactory::init(alice());

    let pool_id = contract.new_pool(2, swap_fee());
    let half = to_token_denom(5) / 10;

    contract.bind_pool(pool_id, U128(to_token_denom(100)), vec![U128(half), U128(half)]);

}
