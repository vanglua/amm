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
        LookupMap
    },
};

use crate::pool::Pool;
use crate::logger;
use crate::pool_factory;
use crate::payload_structs;

const GAS_BASE_COMPUTE: Gas = 5_000_000_000_000;
const STORAGE_PRICE_PER_BYTE: Balance = 100_000_000_000_000_000_000;
const TOKEN_DENOM: u128 = 1_000_000_000_000_000_000; // 1e18
const MAX_FEE: u128 = TOKEN_DENOM / 20; // max fee is 5%
const MIN_FEE: u128 = TOKEN_DENOM / 10_000; // max fee is %0.01

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Market {
    pub end_time: u64,
    pub pool: Pool,
    pub payout_numerator: Option<Vec<U128>>,
    pub finalized: bool,
}

#[ext_contract]
pub trait CollateralToken {
    fn withdraw_from_vault(&mut self, vault_id: u64, receiver_id: AccountId, amount: U128);
    fn transfer(&mut self, receiver_id: AccountId, amount: U128);
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Protocol {
    gov: AccountId, // The gov of all markets
    markets: LookupMap<u64, Market>,
    id: u64, // Incrementing number that's used to define a pool's id
    token_whitelist: Vec<AccountId>
}

#[near_bindgen]
impl Protocol {

    /**
     * @notice Initialize the contract by setting the owner
     * @param owner The `account_id` that's going to have owner privileges
     */
    #[init]
    pub fn init(owner: AccountId, gov: AccountId, token_whitelist: Vec<AccountId>) -> Self {
        assert!(!env::state_exists(), "ERR_CONTRACT_IS_INITIALIZED");
        assert!(env::is_valid_account_id(owner.as_bytes()), "ERR_INVALID_ACCOUNT_ID");
        assert!(env::is_valid_account_id(gov.as_bytes()), "ERR_INVALID_ACCOUNT_ID");

        Self {
            gov,
            markets: LookupMap::new(b"markets".to_vec()),
            id: 0,
            token_whitelist
        }
    }

    pub fn gov(&self) -> AccountId {
        self.gov.to_string()
    }

    pub fn get_pool_swap_fee(&self, market_id: U64) -> U128 {
        let market = self.markets.get(&market_id.into()).expect("ERR_NO_MARKET");
        U128(market.pool.get_swap_fee())
    }

    pub fn get_pool_balances(
        &self,
        market_id: U64
    ) -> Vec<U128>{
        let market = self.markets.get(&market_id.into()).expect("ERR_NO_POOL");
        market.pool.get_pool_balances().iter().map(|b| { U128(*b) }).collect()
    }

    pub fn get_pool_token_balance(&self, market_id: U64, owner_id: &AccountId) -> U128 {
        let market = self.markets.get(&market_id.into()).expect("ERR_NO_MARKET");
        U128(market.pool.get_pool_token_balance(owner_id))
    }

    pub fn get_spot_price_sans_fee(
        &self,
        market_id: U64,
        outcome: u16
    ) -> U128 {
        let market = self.markets.get(&market_id.into()).expect("ERR_NO_MARKET");
        market.pool.get_spot_price_sans_fee(outcome).into()
    }

    pub fn calc_buy_amount(
        &self,
        market_id: U64,
        collateral_in: U128,
        outcome_target: u16
    ) -> U128 {
        let market = self.markets.get(&market_id.into()).expect("ERR_NO_MARKET");
        U128(market.pool.calc_buy_amount(collateral_in.into(), outcome_target))
    }

    pub fn calc_sell_collateral_out(
        &self,
        market_id: U64,
        collateral_out: U128,
        outcome_target: u16
    ) -> U128 {
        let market = self.markets.get(&market_id.into()).expect("ERR_NO_MARKET");
        U128(market.pool.calc_sell_collateral_out(collateral_out.into(), outcome_target))
    }

    pub fn get_share_balance(&self, account_id: &AccountId, market_id: U64, outcome: u16) -> U128 {
        let market = self.markets.get(&market_id.into()).expect("ERR_NO_MARKET");
        U128(market.pool.get_share_balance(account_id, outcome))
    }

    pub fn get_fees_withdrawable(&self, market_id: U64, account_id: &AccountId) -> U128 {
        let market = self.markets.get(&market_id.into()).expect("ERR_NO_MARKET");
        U128(market.pool.get_fees_withdrawable(account_id))
    }

    #[payable]
    pub fn create_market(
        &mut self,
        description: String,
        extra_info: String,
        outcomes: u16,
        outcome_tags: Vec<String>,
        categories: Vec<String>,
        end_time: U64,
        collateral_token_id: AccountId,
        swap_fee: U128,
    ) -> U64 {
        let end_time: u64 = end_time.into();
        let swap_fee: u128 = swap_fee.into();
        let market_id = self.id;
        assert!(self.token_whitelist.contains(&collateral_token_id), "ERR_INVALID_COLLATERAL");
        assert!(outcome_tags.len() as u16 == outcomes, "ERR_INVALID_TAG_LENGTH");
        assert!(end_time > self.ns_to_ms(env::block_timestamp()), "ERR_INVALID_END_TIME");
        assert!(swap_fee == 0 || (swap_fee <= MAX_FEE && swap_fee >= MIN_FEE), "ERR_INVALID_FEE");
        let initial_storage = env::storage_usage();

        let pool = pool_factory::new_pool(
            market_id,
            env::predecessor_account_id(),
            outcomes,
            collateral_token_id,
            swap_fee
        );

        logger::log_pool(&pool);

        let market = Market {
            end_time,
            pool,
            payout_numerator: None,
            finalized: false
        };

        logger::log_market(&market, description, extra_info, outcome_tags, categories);
        logger::log_market_status(&market);

        self.markets.insert(&market_id, &market);
        self.refund_storage(initial_storage, env::predecessor_account_id());
        self.id += 1;
        market_id.into()
    }

    #[payable]
    pub fn exit_pool(
        &mut self,
        market_id: U64,
        total_in: U128,
    ) -> PromiseOrValue<bool> {
        let initial_storage = env::storage_usage();

        let mut market = self.markets.get(&market_id.into()).expect("ERR_NO_MARKET");
        let fees_earned = market.pool.exit_pool(
            &env::predecessor_account_id(),
            total_in.into()
        );

        self.markets.insert(&market_id.into(), &market);

        self.refund_storage(initial_storage, env::predecessor_account_id());

        if fees_earned > 0 {
            PromiseOrValue::Promise(
                collateral_token::transfer(
                    env::predecessor_account_id(),
                    fees_earned.into(),
                    &market.pool.collateral_token_id,
                    0,
                    GAS_BASE_COMPUTE
                )
            )
        } else {
            PromiseOrValue::Value(true)
        }
    }

    #[payable]
    pub fn sell(
        &mut self,
        market_id: U64,
        collateral_out: U128,
        outcome_target: u16,
        max_shares_in: U128
    ) -> Promise {
        let initial_storage = env::storage_usage();
        let collateral_out: u128 = collateral_out.into();
        let mut market = self.markets.get(&market_id.into()).expect("ERR_NO_MARKET");
        assert!(!market.finalized, "ERR_FINALIZED_MARKET");
        assert!(market.end_time > self.ns_to_ms(env::block_timestamp()), "ERR_MARKET_ENDED");
        let escrowed = market.pool.sell(
            &env::predecessor_account_id(),
            collateral_out,
            outcome_target,
            max_shares_in.into()
        );

        self.markets.insert(&market_id.into(), &market);
        self.refund_storage(initial_storage, env::predecessor_account_id());

        collateral_token::transfer(
            env::predecessor_account_id(),
            U128(collateral_out - escrowed),
            &market.pool.collateral_token_id,
            0,
            GAS_BASE_COMPUTE
        )
    }

    #[payable]
    pub fn claim_earnings(
        &mut self,
        market_id: U64
    ) -> Promise {
        let initial_storage = env::storage_usage();
        let mut market = self.markets.get(&market_id.into()).expect("ERR_NO_MARKET");
        assert!(market.finalized, "ERR_NOT_FINALIZED");

        let payout = market.pool.payout(&env::predecessor_account_id(), &market.payout_numerator);

        self.markets.insert(&market_id.into(), &market);

        self.refund_storage(initial_storage, env::predecessor_account_id());

        logger::log_claim_earnings(
            market_id,
            env::predecessor_account_id(),
            payout
        );
        if payout > 0 {
                collateral_token::transfer(
                    env::predecessor_account_id(),
                    payout.into(),
                    &market.pool.collateral_token_id,
                    0,
                    GAS_BASE_COMPUTE
                )
        } else {
            panic!("ERR_NO_PAYOUT");
        }
    }


    #[payable]
    pub fn on_receive_with_vault(
        &mut self,
        sender_id: AccountId,
        vault_id: u64,
        amount: U128,
        payload: String,
    ) -> Promise {
        let amount: u128 = amount.into();
        assert!(amount > 0, "ERR_ZERO_AMOUNT");

        let initial_storage = env::storage_usage();
        let parsed_payload: payload_structs::InitStruct = serde_json::from_str(payload.as_str()).expect("ERR_INCORRECT_JSON");

        let prom: Promise;
        match parsed_payload.function.as_str() {
            "add_liquidity" => prom = self.add_liquidity(&sender_id, vault_id, amount, parsed_payload.args),
            "buy" => prom = self.buy(&sender_id, vault_id, amount, parsed_payload.args),
            _ => panic!("ERR_UNKNOWN_FUNCTION")
        };

        self.refund_storage(initial_storage, sender_id);
        prom
    }

    /*** Gov setters ***/

    // TODO: validate payout num arr
    #[payable]
    pub fn resolute_market(
        &mut self,
        market_id: U64,
        payout_numerator: Option<Vec<U128>>
    ) {
        let initial_storage = env::storage_usage();
        self.assert_gov();
        let mut market = self.markets.get(&market_id.into()).expect("ERR_NO_MARKET");
        assert!(!market.finalized, "ERR_IS_FINALIZED");
        match &payout_numerator {
            Some(v) => assert!(v.len() == market.pool.outcomes as usize, "ERR_INVALID_NUMERATOR"),
            None => ()
        };

        market.payout_numerator = payout_numerator;
        market.finalized = true;
        self.markets.insert(&market_id.0, &market);
        self.refund_storage(initial_storage, env::predecessor_account_id());

        logger::log_market_status(&market);
    }

    pub fn set_gov(
        &mut self,
        new_gov: AccountId
    ) {
        self.assert_gov();
        self.gov = new_gov;
    }

    pub fn set_token_whitelist(
        &mut self,
        whitelist: Vec<AccountId>
    ) {
        self.assert_gov();
        self.token_whitelist = whitelist;
    }

    pub fn add_to_token_whitelist(
        &mut self,
        to_add: AccountId
    ) {
        self.assert_gov();
        self.token_whitelist.push(to_add);
    }

    // payable needed?
    #[payable]
    pub fn burn_outcome_tokens_redeem_collateral(
        &mut self,
        market_id: U64,
        to_burn: U128
    ) -> Promise {
        let initial_storage = env::storage_usage();

        let mut market = self.markets.get(&market_id.into()).expect("ERR_NO_MARKET");
        market.pool.burn_outcome_tokens_redeem_collateral(
            &env::predecessor_account_id(),
            to_burn.into()
        );

        self.markets.insert(&market_id.into(), &market);

        self.refund_storage(initial_storage, env::predecessor_account_id());

        collateral_token::transfer(
            env::predecessor_account_id(),
            to_burn.into(),
            &market.pool.collateral_token_id,
            0,
            GAS_BASE_COMPUTE
        )
    }
}

impl Protocol {
    fn assert_gov(&self) {
        assert_eq!(env::predecessor_account_id(), self.gov, "ERR_NO_GOVERNANCE_ADDRESS");
    }

    // TODO: make pure function
    fn assert_collateral_token(&self, collateral_token: &AccountId) {
        assert_eq!(&env::predecessor_account_id(), collateral_token, "ERR_INVALID_COLLATERAL");
    }

    // TODO: make pure function
    fn ns_to_ms(&self, ns_timestamp: u64) -> u64 {
        ns_timestamp / 1_000_000
    }

    fn add_liquidity(
        &mut self,
        sender: &AccountId,
        vault_id: u64,
        total_in: u128,
        args: serde_json::Value,
    ) -> Promise {
        let parsed_args: payload_structs::AddLiquidity = payload_structs::from_args(args);
        let weights_u128: Option<Vec<u128>> = match parsed_args.weight_indication {
            Some(weight_indication) => {
                Some(weight_indication
                    .iter()
                    .map(|weight| { u128::from(*weight) })
                    .collect()
                )
            },
            None => None
        };

        let mut market = self.markets.get(&parsed_args.market_id.into()).expect("ERR_NO_MARKET");
        assert!(!market.finalized, "ERR_FINALIZED_MARKET");
        assert!(market.end_time > self.ns_to_ms(env::block_timestamp()), "ERR_MARKET_ENDED");
        self.assert_collateral_token(&market.pool.collateral_token_id);

        market.pool.add_liquidity(
            &sender,
            total_in,
            weights_u128
        );
        self.markets.insert(&parsed_args.market_id.into(), &market);

        collateral_token::withdraw_from_vault(
            vault_id,
            env::current_account_id(),
            total_in.into(),
            &market.pool.collateral_token_id,
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
        let mut market = self.markets.get(&parsed_args.market_id.into()).expect("ERR_NO_MARKET");
        assert!(!market.finalized, "ERR_FINALIZED_MARKET");
        assert!(market.end_time > self.ns_to_ms(env::block_timestamp()), "ERR_MARKET_ENDED");
        self.assert_collateral_token(&market.pool.collateral_token_id);

        market.pool.buy(
            &sender,
            collateral_in,
            parsed_args.outcome_target,
            parsed_args.min_shares_out.into()
        );

        self.markets.insert(&parsed_args.market_id.into(), &market);

        collateral_token::withdraw_from_vault(
            vault_id,
            env::current_account_id(),
            collateral_in.into(),
            &market.pool.collateral_token_id,
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
            Promise::new(sender_id).transfer(refund_amount);
        }
    }
}