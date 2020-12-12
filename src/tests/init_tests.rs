use super::*;

#[test]
fn test_contract_initiation() {
    let context = get_context(alice(), 0);
    testing_env!(context);
    PoolFactory::init(alice());
}


#[test]
#[should_panic(expected = "ERR_INVALID_ACCOUNT_ID")]
fn test_contract_initiation_invalid_account_id() {
    let context = get_context(alice(), 0);
    testing_env!(context);
    PoolFactory::init("{}adjkbjksd_".to_string());
}