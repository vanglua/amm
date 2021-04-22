use near_sdk_sim::to_yocto;
use near_sdk::json_types::U128;
use crate::test_utils::*;

#[test]
fn test_near_wrap() {
    let (master_account, amm, token, alice, bob, carol) = init("carol".to_string());
    let owner_balance = ft_balance_of(&alice, &alice.account_id());
    assert_eq!(owner_balance, U128(1000000000000000000000000000));
}

// #[test]
// #[should_panic(expected = "The account ID is invalid")]
// fn test_contract_initiation_invalid_account_id() {
//     init(to_yocto("100"), "{}{}carol".to_string());
// }
