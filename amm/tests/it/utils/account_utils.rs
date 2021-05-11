use crate::utils::*;

pub fn init_account(
    master_account: Option<&UserAccount>, 
    account_id: Option<&str>
) -> UserAccount {
    match master_account {
        Some(master_account) => {
            let user = master_account.create_user(account_id.expect("expected account id").to_string(), to_yocto("1000"));
            storage_deposit(TOKEN_CONTRACT_ID, &user, SAFE_STORAGE_AMOUNT, None);
            storage_deposit(ORACLE_CONTRACT_ID, &user, SAFE_STORAGE_AMOUNT, None);
            user
        },
        None => init_simulator(None)
    }
}