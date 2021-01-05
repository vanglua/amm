#![allow(clippy::needless_pass_by_value)]
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    PromiseOrValue,
    Balance,
    StorageUsage,
    Gas,
    ext_contract,
    near_bindgen,
    Promise,
    PanicOnDefault,
    json_types::{
        U128, 
        U64
    },
    serde_json,
    AccountId, 
    env,
    collections::{
        UnorderedMap
    },
};

use crate::pool::Pool;
use crate::payload_structs;

const GAS_BASE_COMPUTE: Gas = 5_000_000_000_000;
const STORAGE_PRICE_PER_BYTE: Balance = 100_000_000_000_000_000_000;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct PoolFactory {
    owner: AccountId, // The owner of the contract
    collateral_token: AccountId,
    nonce: u64, // Incrementing number that's used to define a pool's id
    pools: UnorderedMap<u64, Pool> // Maps pool ids to pool
}


#[ext_contract]
pub trait CollateralToken {
    fn withdraw_from_vault(&mut self, vault_id: u64, receiver_id: AccountId, amount: U128);
    fn transfer(&mut self, receiver_id: AccountId, amount: U128);
}


#[near_bindgen]
impl PoolFactory {

    /**
     * @notice Initialize the contract by setting the owner
     * @param owner The `account_id` that's going to have owner privileges
     */
    #[init]
    pub fn init(owner: AccountId, collateral_token: AccountId) -> Self {
        assert!(!env::state_exists(), "ERR_CONTRACT_IS_INITIALIZED");
        assert!(env::is_valid_account_id(owner.as_bytes()), "ERR_INVALID_ACCOUNT_ID");
        
        Self {
            owner: owner,
            collateral_token,
            nonce: 0,
            pools: UnorderedMap::new(b"pools".to_vec())
        }
    }

    pub fn get_share_balance(&self, account_id: &AccountId, pool_id: U64, outcome: u16) -> U128 {
        let pool = self.pools.get(&pool_id.into()).expect("ERR_NO_POOL");
        U128(pool.get_share_balance(account_id, outcome))
    }

    pub fn get_pool_token_balance(&self, pool_id: U64, owner_id: &AccountId) -> U128 {
        let pool = self.pools.get(&pool_id.into()).expect("ERR_NO_POOL");
        U128(pool.get_pool_token_balance(owner_id))
    }

    pub fn get_pool_swap_fee(&self, pool_id: U64) -> U128 {
        let pool = self.pools.get(&pool_id.into()).expect("ERR_NO_POOL");
        U128(pool.get_swap_fee())
    }

    pub fn get_fees_withdrawable(&self, pool_id: U64, account_id: &AccountId) -> U128 {
        let pool = self.pools.get(&pool_id.into()).expect("ERR_NO_POOL");
        U128(pool.get_fees_withdrawable(account_id))
    }

    #[payable]
    pub fn new_pool(
        &mut self, 
        outcomes: u16,
        swap_fee: U128
    ) -> U64 {
        let initial_storage = env::storage_usage();

        let pool_id = self.nonce;
        let new_pool = Pool::new(
            env::predecessor_account_id(),
            pool_id,
            outcomes,
            swap_fee.into()
        );
        self.nonce += 1;
        self.pools.insert(&pool_id, &new_pool);
        self.refund_storage(initial_storage, env::predecessor_account_id());

        pool_id.into()
    }
    
    #[payable]
    pub fn seed_pool(
        &mut self, 
        pool_id: U64,
        total_in: U128, 
        denorm_weights: Vec<U128>
    ) {
        let initial_storage = env::storage_usage();

        let weights_u128: Vec<u128> = denorm_weights
            .iter()
            .map(|weight| { u128::from(*weight) })
            .collect();

        let mut pool = self.pools.get(&pool_id.into()).expect("ERR_NO_POOL");
        pool.seed_pool(
            &env::predecessor_account_id(), 
            total_in.into(), 
            &weights_u128
        );
        
        self.pools.insert(&pool_id.into(), &pool);
        self.refund_storage(initial_storage, env::predecessor_account_id());
    }

    #[payable]
    pub fn exit_pool(
        &mut self, 
        pool_id: U64,
        total_in: U128, 
    ) -> PromiseOrValue<bool> {
        let initial_storage = env::storage_usage();

        let mut pool = self.pools.get(&pool_id.into()).expect("ERR_NO_POOL");
        let fees_earned = pool.exit_pool(
            &env::predecessor_account_id(), 
            total_in.into()
        );
        
        self.pools.insert(&pool_id.into(), &pool);

        self.refund_storage(initial_storage, env::predecessor_account_id());

        if fees_earned > 0 {
            return PromiseOrValue::Promise(
                collateral_token::transfer(
                    env::predecessor_account_id(), 
                    fees_earned.into(),
                    &self.collateral_token,
                    0,
                    GAS_BASE_COMPUTE
                )
            )
        } else {
            return PromiseOrValue::Value(true)
        }
    }

    pub fn get_pool_balances(
        &self,
        pool_id: U64
    ) -> Vec<U128>{
        let pool = self.pools.get(&pool_id.into()).expect("ERR_NO_POOL");
        pool.get_pool_balances().iter().map(|b| { U128(*b) }).collect()
    }

    pub fn get_spot_price(
        &self, 
        pool_id: U64, 
        outcome: u16
    ) -> U128 {
        let pool = self.pools.get(&pool_id.into()).expect("ERR_NO_POOL");
        pool.get_spot_price(outcome).into()
    }
    
    pub fn get_spot_price_sans_fee(
        &self, 
        pool_id: U64, 
        outcome: u16
    ) -> U128 {        
        let pool = self.pools.get(&pool_id.into()).expect("ERR_NO_POOL");
        pool.get_spot_price_sans_fee(outcome).into()
    }
    
    pub fn calc_buy_amount(
        &self, 
        pool_id: U64, 
        collateral_in: U128,
        outcome_target: u16
    ) -> U128 {
        let pool = self.pools.get(&pool_id.into()).expect("ERR_NO_POOL");
        U128(pool.calc_buy_amount(collateral_in.into(), outcome_target))
    }

    pub fn calc_sell_collateral_out(
        &self, 
        pool_id: U64, 
        collateral_out: U128, 
        outcome_target: u16
    ) -> U128 {
        let pool = self.pools.get(&pool_id.into()).expect("ERR_NO_POOL");
        U128(pool.calc_sell_tokens_in(collateral_out.into(), outcome_target))
    }

    #[payable]
    pub fn sell(
        &mut self, 
        pool_id: U64, 
        collateral_out: U128, 
        outcome_target: u16,
        max_shares_in: U128
    ) -> Promise {
        let initial_storage = env::storage_usage();

        let mut pool = self.pools.get(&pool_id.into()).expect("ERR_NO_POOL");
        pool.sell(
            collateral_out.into(), 
            outcome_target, 
            max_shares_in.into()
        );
        self.pools.insert(&pool_id.into(), &pool);
        self.refund_storage(initial_storage, env::predecessor_account_id());

        collateral_token::transfer(
            env::predecessor_account_id(), 
            collateral_out.into(),
            &self.collateral_token,
            0,
            GAS_BASE_COMPUTE
        )
    }

    #[payable]
    pub fn on_receive_with_vault(
        &mut self,
        sender_id: AccountId,
        vault_id: u64,
        amount: U128,
        payload: String,
    ) -> Promise {
        let initial_storage = env::storage_usage();

        assert_eq!(env::predecessor_account_id(), self.collateral_token, "ERR_INVALID_SENDER");
        // Check sender is payment token address
        let parsed_payload: payload_structs::InitStruct = serde_json::from_str(payload.as_str()).expect("ERR_INCORRECT_JSON");

        let prom: Promise;
        match parsed_payload.function.as_str() {
            "finalize" => prom = self.finalize_pool(&sender_id, vault_id, amount.into(), parsed_payload.args),
            "join_pool" => prom = self.join_pool(&sender_id, vault_id, amount.into(), parsed_payload.args),
            "buy" => prom = self.buy(&sender_id, vault_id, amount.into(), parsed_payload.args),
            _ => panic!("ERR_INVALID_TYPE")
        };

        self.refund_storage(initial_storage, sender_id);
        prom
    }
}

impl PoolFactory {

    fn finalize_pool(
        &mut self,
        sender: &AccountId,
        vault_id: u64,
        total_in: u128, 
        args: serde_json::Value,
    ) -> Promise {
        let parsed_args: payload_structs::LPPool = payload_structs::from_args(args);
        let mut pool = self.pools.get(&parsed_args.pool_id.into()).expect("ERR_NO_POOL");
        let withdraw_amount = pool.finalize(&sender, total_in);
        self.pools.insert(&parsed_args.pool_id.into(), &pool);

        collateral_token::withdraw_from_vault(
            vault_id, 
            env::current_account_id(), 
            total_in.into(),
            &self.collateral_token,
            0,
            GAS_BASE_COMPUTE
        )
    }

    fn join_pool(
        &mut self, 
        sender: &AccountId,
        vault_id: u64,
        total_in: u128, 
        args: serde_json::Value,
    ) -> Promise {
        let parsed_args: payload_structs::LPPool = payload_structs::from_args(args);
        let mut pool = self.pools.get(&parsed_args.pool_id.into()).expect("ERR_NO_POOL");
        pool.join_pool(
            &sender, 
            total_in
        );
        self.pools.insert(&parsed_args.pool_id.into(), &pool);

        collateral_token::withdraw_from_vault(
            vault_id, 
            env::current_account_id(), 
            total_in.into(),
            &self.collateral_token,
            0,
            GAS_BASE_COMPUTE
        )
    }

    fn buy(
        &mut self, 
        sender: &AccountId,
        vault_id: u64,
        collateral_in: u128, 
        args: serde_json::Value,
    ) -> Promise {
        let parsed_args: payload_structs::Buy = payload_structs::from_args(args);
        let mut pool = self.pools.get(&parsed_args.pool_id.into()).expect("ERR_NO_POOL");
        pool.buy(
            &sender,
            collateral_in, 
            parsed_args.outcome_target, 
            parsed_args.min_shares_out.into()
        );
        self.pools.insert(&parsed_args.pool_id.into(), &pool);

        collateral_token::withdraw_from_vault(
            vault_id, 
            env::current_account_id(), 
            collateral_in.into(),
            &self.collateral_token,
            0,
            GAS_BASE_COMPUTE
        )
    }

    fn refund_storage(&self, initial_storage: StorageUsage, sender_id: AccountId) {
        let current_storage = env::storage_usage();
        let attached_deposit = env::attached_deposit();
        let refund_amount = if current_storage > initial_storage {
            let required_deposit =
                Balance::from(current_storage - initial_storage) * STORAGE_PRICE_PER_BYTE;
            assert!(
                required_deposit <= attached_deposit,
                "The required attached deposit is {}, but the given attached deposit is is {}",
                required_deposit,
                attached_deposit,
            );
            attached_deposit - required_deposit
        } else {
            attached_deposit
                + Balance::from(initial_storage - current_storage) * STORAGE_PRICE_PER_BYTE
        };
        if refund_amount > 0 {
            env::log(format!("Refunding {} tokens for storage", refund_amount).as_bytes());
            Promise::new(sender_id).transfer(refund_amount);
        }
    }
}