use crate::utils::*;
use amm::types::Source;

const AMM_DEPOSIT: u128 = 50000000000000000000000;
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
                storage_deposit(AMM_CONTRACT_ID, &master_account, AMM_DEPOSIT, Some(account.account_id()));
                storage_deposit(TOKEN_CONTRACT_ID, &master_account, SAFE_STORAGE_AMOUNT, Some(account.account_id()));
                storage_deposit(ORACLE_CONTRACT_ID, &master_account, SAFE_STORAGE_AMOUNT, Some(account.account_id()));
                near_deposit(&account, init_balance() / 2);
                Self {
                    account
                }
            },
            None => Self { account: init_simulator(None) }
        }
    }
    /*** Getters ***/
    pub fn get_token_balance(&self, account_id: Option<String>) -> u128 {
        let account_id = match account_id {
            Some(account_id) => account_id,
            None => self.account.account_id()
        };

        let res: U128 = self.account.view(
            PendingContractTx::new(
                TOKEN_CONTRACT_ID, 
                "ft_balance_of", 
                json!({
                    "account_id": account_id
                }), 
                true
            )
        ).unwrap_json();

        res.into()
    }
    
    pub fn get_pool_token_balance(&self, market_id: u64, account_id: Option<String>) -> u128 {
        let account_id = match account_id {
            Some(account_id) => account_id,
            None => self.account.account_id()
        };

        let res: U128 = self.account.view(
            PendingContractTx::new(
                AMM_CONTRACT_ID, 
                "get_pool_token_balance", 
                json!({
                    "market_id": U64(market_id),
                    "account_id": account_id
                }), 
                true
            )
        ).unwrap_json();

        res.into()
    }

    pub fn get_pool_balances(&self, market_id: u64) -> Vec<u128> {
        let wrapped_balances: Vec<U128> = self.account.view(
            PendingContractTx::new(
                AMM_CONTRACT_ID, 
                "get_pool_balances", 
                json!({
                    "market_id": U64(market_id)
                }), 
                true
            )
        ).unwrap_json();

        wrapped_balances.into_iter().map(|wrapped_balances| { wrapped_balances.into() }).collect()
    }

    pub fn get_outcome_balance(&self, account_id: Option<AccountId>, market_id: u64, outcome: u16) -> u128 {
        let account_id = match account_id {
            Some(account_id) => account_id,
            None => self.account.account_id()
        };

        let wrapped_balance: U128 = self.account.view(
            PendingContractTx::new(
                AMM_CONTRACT_ID, 
                "get_share_balance", 
                json!({
                    "account_id": account_id, 
                    "market_id": U64(market_id),
                    "outcome": outcome
                }), 
                true
            )
        ).unwrap_json();

        wrapped_balance.into()
    }

    pub fn get_spot_price_sans_fee(&self, market_id: u64, outcome: u16) -> u128 {
        let wrapped_balance: U128 = self.account.view(
            PendingContractTx::new(
                AMM_CONTRACT_ID, 
                "get_spot_price_sans_fee", 
                json!({
                    "market_id": U64(market_id),
                    "outcome": outcome
                }), 
                true
            )
        ).unwrap_json();

        wrapped_balance.into()
    }

    pub fn get_spot_price(&self, market_id: u64, outcome: u16) -> u128 {
        let wrapped_balance: U128 = self.account.view(
            PendingContractTx::new(
                AMM_CONTRACT_ID, 
                "get_spot_price", 
                json!({
                    "market_id": U64(market_id),
                    "outcome": outcome
                }), 
                true
            )
        ).unwrap_json();

        wrapped_balance.into()
    }

    pub fn calc_buy_amount(&self, market_id: u64, outcome: u16, collateral_in: u128) -> u128 {
        let wrapped_balance: U128 = self.account.view(
            PendingContractTx::new(
                AMM_CONTRACT_ID, 
                "calc_buy_amount", 
                json!({
                    "market_id": U64(market_id),
                    "collateral_in": U128(collateral_in),
                    "outcome_target": outcome
                }), 
                true
            )
        ).unwrap_json();

        wrapped_balance.into()
    }

    pub fn calc_sell_amount(&self, market_id: u64, outcome: u16, collateral_out: u128) -> u128 {
        let wrapped_balance: U128 = self.account.view(
            PendingContractTx::new(
                AMM_CONTRACT_ID, 
                "calc_sell_collateral_out", 
                json!({
                    "market_id": U64(market_id),
                    "collateral_out": U128(collateral_out),
                    "outcome_target": outcome
                }), 
                true
            )
        ).unwrap_json();

        wrapped_balance.into()
    }

    pub fn get_fees_withdrawable(&self, market_id: u64, account_id: Option<AccountId>) -> u128 {
        let account_id = match account_id {
            Some(account_id) => account_id,
            None => self.account.account_id()
        };

        let wrapped_balance: U128 = self.account.view(
            PendingContractTx::new(
                AMM_CONTRACT_ID, 
                "get_fees_withdrawable", 
                json!({
                    "market_id": U64(market_id),
                    "account_id": account_id,
                }), 
                true
            )
        ).unwrap_json();

        wrapped_balance.into()
    }

    /*** Setters ***/
    pub fn create_market(&self, outcomes: u16, fee_opt: Option<U128>) -> ExecutionResult {
        let msg = json!({
            "CreateMarketArgs": {
                "description": empty_string(),
                "extra_info": empty_string(),
                "outcomes": outcomes,
                "outcome_tags": empty_string_vec(outcomes),
                "categories": empty_string_vec(outcomes),
                "end_time": env_time(),
                "sources": vec![Source {
                    end_point: empty_string(),
                    source_path: empty_string()
                }],
                "challenge_period": U64(1000),
                "resolution_time": env_time(),
                "collateral_token_id": TOKEN_CONTRACT_ID,
                "swap_fee": fee_opt,
                "is_scalar": false
            }
        }).to_string();
        self.ft_transfer_call(AMM_CONTRACT_ID.to_string(), to_yocto("100"), msg)
    }

    pub fn add_liquidity(&self, market_id: u64, amount: u128, weights: Option<Vec<U128>>) -> ExecutionResult {
        let msg  = json!({
            "AddLiquidityArgs": {
                "market_id": market_id.to_string(),
                "weight_indication": weights,
            }
        }).to_string();
        self.ft_transfer_call(AMM_CONTRACT_ID.to_string(), amount, msg)
    }

    pub fn exit_liquidity(&self, market_id: u64, total_in: u128) -> ExecutionResult {
        let res = self.account.call(
            PendingContractTx::new(
                AMM_CONTRACT_ID, 
                "exit_pool", 
                json!({
                    "market_id": U64(market_id),
                    "total_in": U128(total_in)
                }), 
                true
            ),
            STORAGE_AMOUNT,
            DEFAULT_GAS
        );
        println!("{:?}", res);
        assert!(res.is_ok(), "ft_transfer_call failed with res: {:?}", res);
        res
    }

    pub fn buy(&self, market_id: u64, amount: u128, outcome: u16, min_amount_out: u128) -> ExecutionResult {
        let msg  = json!({
            "BuyArgs": {
                "market_id": U64(market_id),
                "outcome_target": outcome,
                "min_shares_out": U128(min_amount_out)
            }
        }).to_string();
        self.ft_transfer_call(AMM_CONTRACT_ID.to_string(), amount, msg)
    }
    
    pub fn sell(&self, market_id: u64, amount_out: u128, outcome: u16, max_shares_in: u128) -> ExecutionResult {
        let res = self.account.call(
            PendingContractTx::new(
                AMM_CONTRACT_ID, 
                "sell", 
                json!({
                    "market_id": U64(market_id),
                    "collateral_out": U128(amount_out),
                    "outcome_target": outcome,
                    "max_shares_in": U128(max_shares_in)
                }), 
                true
            ),
            STORAGE_AMOUNT,
            DEFAULT_GAS
        );
        assert!(res.is_ok(), "sell failed with res: {:?}", res);
        res
    }

    pub fn redeem_collateral(&self, market_id: u64, amount_out: u128) -> ExecutionResult {
        let res = self.account.call(
            PendingContractTx::new(
                AMM_CONTRACT_ID, 
                "burn_outcome_tokens_redeem_collateral", 
                json!({
                    "market_id": U64(market_id),
                    "to_burn": U128(amount_out)
                }), 
                true
            ),
            STORAGE_AMOUNT,
            DEFAULT_GAS
        );
        assert!(res.is_ok(), "redeem_collateral failed with res: {:?}", res);
        res
    }

    pub fn resolute_market(&self, market_id: u64, payout_numerator: Option<Vec<U128>>) -> ExecutionResult {
        let res = self.account.call(
            PendingContractTx::new(
                AMM_CONTRACT_ID, 
                "resolute_market", 
                json!({
                    "market_id": U64(market_id),
                    "payout_numerator": payout_numerator
                }), 
                true
            ),
            STORAGE_AMOUNT,
            DEFAULT_GAS
        );
        assert!(res.is_ok(), "redeem_collateral failed with res: {:?}", res);
        res
    }

    pub fn claim_earnings(&self, market_id: u64) -> ExecutionResult {
        let res = self.account.call(
            PendingContractTx::new(
                AMM_CONTRACT_ID, 
                "claim_earnings", 
                json!({
                    "market_id": U64(market_id),
                }), 
                true
            ),
            STORAGE_AMOUNT,
            DEFAULT_GAS
        );
        assert!(res.is_ok(), "redeem_collateral failed with res: {:?}", res);
        res
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