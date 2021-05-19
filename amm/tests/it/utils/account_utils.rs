use crate::utils::*;

pub fn init_balance() -> u128 {
    to_yocto("1000")
}

pub struct TestAccount {
    pub account: UserAccount
}

impl TestAccount {
    pub fn new(
        master_account: Option<&UserAccount>, 
        account_id: Option<&str>
    ) -> Self {
        match master_account {
            Some(master_account) => {
                let account = master_account.create_user(account_id.expect("expected account id").to_string(), init_balance());
                storage_deposit(TOKEN_CONTRACT_ID, &account, SAFE_STORAGE_AMOUNT, None);
                storage_deposit(ORACLE_CONTRACT_ID, &account, SAFE_STORAGE_AMOUNT, None);
                near_deposit(&account, init_balance() * 5 / 10);
                Self {
                    account
                }
            },
            None => Self { account: init_simulator(None) }
        }
    }

    pub fn create_market(&self, outcomes: u16, fee_opt: Option<U128>) -> ExecutionResult {
        let msg = json!({
            "CreateMarketArgs": {
                "description": empty_string(),
                "extra_info": empty_string(),
                "outcomes": outcomes,
                "outcome_tags": empty_string_vec(outcomes),
                "categories": empty_string_vec(outcomes),
                "end_time": env_time(),
                "resolution_time": env_time(),
                "collateral_token_id": TOKEN_CONTRACT_ID,
                "swap_fee": fee_opt,
                "is_scalar": false
            }
        }).to_string();
        self.ft_transfer_call(AMM_CONTRACT_ID.to_string(), to_yocto("100"), msg)
    }

    pub fn ft_transfer_call(
        &self,
        receiver: String,
        amount: u128,
        msg: String
    ) -> ExecutionResult {        
        let res = self.account.call(
            PendingContractTx::new(
                TOKEN_CONTRACT_ID, 
                "ft_transfer_call", 
                json!({
                    "receiver_id": receiver,
                    "amount": U128(amount),
                    "msg": msg,
                    "memo": "".to_string()
                }), 
                true
            ),
            1,
            DEFAULT_GAS
        );
    
        assert!(res.is_ok(), "ft_transfer_call failed with res: {:?}", res);
        res
    }
}