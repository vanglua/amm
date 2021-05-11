use crate::utils::*;

pub fn storage_deposit(
    receiver: &str, 
    sender: &UserAccount, 
    deposit: u128, 
    to_register: Option<AccountId>
) {
    let res = sender.call(
        PendingContractTx::new(
            receiver,
            "storage_deposit",
            json!({
                "account_id": to_register
            }),
            false
        ),
        deposit,
        DEFAULT_GAS
    );
    assert!(res.is_ok(), "storage deposit failed with res: {:?}", res);
}

pub fn near_deposit(sender: &UserAccount, deposit: u128) {
    let res = sender.call(
        PendingContractTx::new(
            TOKEN_CONTRACT_ID,
            "near_deposit",
            json!({}),
            false
        ),
        deposit,
        DEFAULT_GAS
    );
    assert!(res.is_ok(), "wnear deposit failed with res: {:?}", res);
}