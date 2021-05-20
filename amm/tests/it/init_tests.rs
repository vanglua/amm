use near_sdk_sim::to_yocto;
use near_sdk::json_types::U128;
use crate::utils::*;

#[test]
fn test_near_wrap() {
    let test_utils = TestUtils::init(carol());
    let owner_balance = test_utils.alice.get_token_balance(None);
    assert_eq!(owner_balance, init_balance() / 2);
}