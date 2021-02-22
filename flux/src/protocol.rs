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
        U64,
        ValidAccountId
    },
    serde_json,
    AccountId,
    env,
    collections::{
        Vector,
        UnorderedMap
    },
};

use crate::helper::*;

use crate::pool::Pool;
use crate::logger;
use crate::pool_factory;
use crate::msg_structs;

const GAS_BASE_COMPUTE: Gas = 5_000_000_000_000;
const STORAGE_PRICE_PER_BYTE: Balance = 100_000_000_000_000_000_000;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Market {
    pub end_time: u64,
    pub pool: Pool,
    pub payout_numerator: Option<Vec<U128>>,
    pub finalized: bool,
}

#[ext_contract]
pub trait CollateralToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Protocol {
    gov: AccountId, // The gov of all markets
    markets: Vector<Market>,
    token_whitelist: UnorderedMap<AccountId, u32>, // Map a token's account id to number of decimals it's denominated in TODO: change to iterable or hashmap
    paused: bool
}

#[near_bindgen]
impl Protocol {
    /**
     * @notice Initialize the contract by setting the
     * @param gov is the account_id of the account with governance privilages
     * @param token_whitelist is a list of tokens that can be used ås collateral
     */
    #[init]
    pub fn init(
        gov: ValidAccountId, 
        tokens: Vec<ValidAccountId>, 
        decimals: Vec<u32>
    ) -> Self {
        assert!(!env::state_exists(), "ERR_CONTRACT_IS_INITIALIZED");
        assert_eq!(tokens.len(), decimals.len(), "ERR_INVALID_INIT_VEC_LENGTHS");
        let mut token_whitelist: UnorderedMap<AccountId, u32> = UnorderedMap::new(b"wl".to_vec());

        for (i, id) in tokens.into_iter().enumerate() {
            let decimal = decimals[i];
            let account_id: AccountId = id.into();
            token_whitelist.insert(&account_id, &decimal);
        };

        logger::log_whitelist(&token_whitelist);

        Self {
            gov: gov.into(),
            markets: Vector::new(b"m".to_vec()),
            token_whitelist, 
            paused: false
        }
    }

    pub fn gov(&self) -> AccountId {
        self.gov.to_string()
    }

    pub fn get_pool_swap_fee(&self, market_id: U64) -> U128 {
        let market = self.get_market_expect(market_id);
        U128(market.pool.get_swap_fee())
    }

    pub fn get_token_whitelist(&self) -> Vec<(AccountId, u32)> {
        self.token_whitelist.to_vec()
    }

    pub fn get_pool_balances(
        &self,
        market_id: U64
    ) -> Vec<U128>{
        let market = self.get_market_expect(market_id);
        market.pool.get_pool_balances().into_iter().map(|b| b.into()).collect()
    }

    pub fn get_pool_token_balance(&self, market_id: U64, owner_id: &AccountId) -> U128 {
        let market = self.get_market_expect(market_id);
        U128(market.pool.get_pool_token_balance(owner_id))
    }

    pub fn get_spot_price_sans_fee(
        &self,
        market_id: U64,
        outcome: u16
    ) -> U128 {
        let market = self.get_market_expect(market_id);
        market.pool.get_spot_price_sans_fee(outcome).into()
    }

    pub fn calc_buy_amount(
        &self,
        market_id: U64,
        collateral_in: U128,
        outcome_target: u16
    ) -> U128 {
        let market = self.get_market_expect(market_id);
        U128(market.pool.calc_buy_amount(collateral_in.into(), outcome_target))
    }

    pub fn calc_sell_collateral_out(
        &self,
        market_id: U64,
        collateral_out: U128,
        outcome_target: u16
    ) -> U128 {
        let market = self.get_market_expect(market_id);
        U128(market.pool.calc_sell_collateral_out(collateral_out.into(), outcome_target))
    }

    pub fn get_share_balance(&self, account_id: &AccountId, market_id: U64, outcome: u16) -> U128 {
        let market = self.get_market_expect(market_id);
        U128(market.pool.get_share_balance(account_id, outcome))
    }

    pub fn get_fees_withdrawable(&self, market_id: U64, account_id: &AccountId) -> U128 {
        let market = self.get_market_expect(market_id);
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
        self.assert_unpaused();
        let end_time: u64 = end_time.into();
        let swap_fee: u128 = swap_fee.into();
        let market_id = self.markets.len();
        let token_decimals = self.token_whitelist.get(&collateral_token_id);
        assert!(token_decimals.is_some(), "ERR_INVALID_COLLATERAL");
        assert!(outcome_tags.len() as u16 == outcomes, "ERR_INVALID_TAG_LENGTH");
        assert!(end_time > ns_to_ms(env::block_timestamp()), "ERR_INVALID_END_TIME");
        let initial_storage = env::storage_usage();

        let pool = pool_factory::new_pool(
            market_id,
            outcomes,
            collateral_token_id,
            token_decimals.unwrap(),
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
        
        self.markets.push(&market);
        self.refund_storage(initial_storage, env::predecessor_account_id());
        market_id.into()
    }

    #[payable]
    pub fn exit_pool(
        &mut self,
        market_id: U64,
        total_in: U128,
    ) -> PromiseOrValue<bool> {
        self.assert_unpaused();
        let initial_storage = env::storage_usage();

        let mut market = self.markets.get(market_id.into()).expect("ERR_NO_MARKET");
        let fees_earned = market.pool.exit_pool(
            &env::predecessor_account_id(),
            total_in.into()
        );
        
        self.markets.replace(market_id.into(), &market);

        self.refund_storage(initial_storage, env::predecessor_account_id());

        if fees_earned > 0 {
            PromiseOrValue::Promise(
                collateral_token::ft_transfer(
                    env::predecessor_account_id(), 
                    fees_earned.into(),
                    None,
                    &market.pool.collateral_token_id,
                    1,
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
        self.assert_unpaused();
        let initial_storage = env::storage_usage();
        let collateral_out: u128 = collateral_out.into();
        let mut market = self.markets.get(market_id.into()).expect("ERR_NO_MARKET");
        assert!(!market.finalized, "ERR_FINALIZED_MARKET");
        assert!(market.end_time > ns_to_ms(env::block_timestamp()), "ERR_MARKET_ENDED");
        let escrowed = market.pool.sell(
            &env::predecessor_account_id(),
            collateral_out,
            outcome_target,
            max_shares_in.into()
        );

        self.markets.replace(market_id.into(), &market);
        self.refund_storage(initial_storage, env::predecessor_account_id());

        collateral_token::ft_transfer(
            env::predecessor_account_id(), 
            U128(collateral_out - escrowed),
            None,
            &market.pool.collateral_token_id,
            1,
            GAS_BASE_COMPUTE
        )
    }

    #[payable]
    pub fn claim_earnings(
        &mut self,
        market_id: U64
    ) -> Promise { 
        self.assert_unpaused();
        let initial_storage = env::storage_usage();
        let mut market = self.markets.get(market_id.into()).expect("ERR_NO_MARKET");
        assert!(market.finalized, "ERR_NOT_FINALIZED");

        let payout = market.pool.payout(&env::predecessor_account_id(), &market.payout_numerator);
        self.markets.replace(market_id.into(), &market);

        self.refund_storage(initial_storage, env::predecessor_account_id());

        logger::log_claim_earnings(
            market_id,
            env::predecessor_account_id(),
            payout
        );

        if payout > 0 {
                collateral_token::ft_transfer(
                    env::predecessor_account_id(), 
                    payout.into(),
                    None,
                    &market.pool.collateral_token_id,
                    1,
                    GAS_BASE_COMPUTE
                )
        } else {
            panic!("ERR_NO_PAYOUT");
        }
    }


    // Callback for collateral tokens
    #[payable]
    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> U128 {
        self.assert_unpaused();
        let amount: u128 = amount.into();
        assert!(amount > 0, "ERR_ZERO_AMOUNT");

        let parsed_msg: msg_structs::InitStruct = serde_json::from_str(msg.as_str()).expect("ERR_INCORRECT_JSON");

        match parsed_msg.function.as_str() {
            "add_liquidity" => self.add_liquidity(&sender_id, amount, parsed_msg.args), 
            "buy" => self.buy(&sender_id, amount, parsed_msg.args),
            _ => panic!("ERR_UNKNOWN_FUNCTION")
        };

        0.into()
    }

    /*** Gov setters ***/
    #[payable]
    pub fn resolute_market(
        &mut self,
        market_id: U64,
        payout_numerator: Option<Vec<U128>>
    ) {
        self.assert_gov();
        // let initial_storage = env::storage_usage();
        let mut market = self.markets.get(market_id.into()).expect("ERR_NO_MARKET");
        assert!(!market.finalized, "ERR_IS_FINALIZED");
        match &payout_numerator {
            Some(v) => {
                let sum = v.iter().fold(0, |s, &n| s + u128::from(n));
                assert_eq!(sum, market.pool.collateral_denomination, "ERR_INVALID_PAYOUT_SUM");
                assert_eq!(v.len(), market.pool.outcomes as usize, "ERR_INVALID_NUMERATOR");
            },
            None => ()
        };

        market.payout_numerator = payout_numerator;
        market.finalized = true;
        self.markets.replace(market_id.into(), &market);
        // self.refund_storage(initial_storage, env::predecessor_account_id());

        logger::log_market_status(&market);
    }

    pub fn set_gov(
        &mut self,
        new_gov: ValidAccountId
    ) {
        self.assert_gov();
        self.gov = new_gov.into();
    }

    pub fn pause(&mut self) {
        self.assert_gov();
        self.paused = true;
    }

    pub fn unpause(&mut self) {
        self.assert_gov();
        self.paused = false;
    }

    pub fn set_token_whitelist(
        &mut self,
        tokens: Vec<ValidAccountId>, 
        decimals: Vec<u32>
    ) {
        self.assert_gov();
        assert_eq!(tokens.len(), decimals.len(), "ERR_INVALID_INIT_VEC_LENGTHS");
        let mut token_whitelist: UnorderedMap<AccountId, u32> = UnorderedMap::new(b"wl".to_vec());

        for (i, id) in tokens.into_iter().enumerate() {
            let decimal = decimals[i];
            let account_id: AccountId = id.into();
            token_whitelist.insert(&account_id, &decimal);
        };
        logger::log_whitelist(&token_whitelist);
        self.token_whitelist = token_whitelist
    }

    pub fn add_to_token_whitelist(
        &mut self,
        to_add: ValidAccountId,
        decimals: u32
    ) {
        self.assert_gov();
        let account_id = to_add.into();
        self.token_whitelist.insert(&account_id, &decimals);
        logger::log_whitelist(&self.token_whitelist);

    }

    #[payable]
    pub fn burn_outcome_tokens_redeem_collateral(
        &mut self,
        market_id: U64,
        to_burn: U128
    ) -> Promise {
        self.assert_unpaused();
        let initial_storage = env::storage_usage();

        let mut market = self.markets.get(market_id.into()).expect("ERR_NO_MARKET");
        assert!(!market.finalized, "ERR_MARKET_FINALIZED");

        let escrowed = market.pool.burn_outcome_tokens_redeem_collateral(
            &env::predecessor_account_id(),
            to_burn.into()
        );

        self.markets.replace(market_id.into(), &market);

        self.refund_storage(initial_storage, env::predecessor_account_id());

        let payout = u128::from(to_burn) - escrowed;
        collateral_token::ft_transfer(
            env::predecessor_account_id(),
            payout.into(),
            None,
            &market.pool.collateral_token_id,
            1,
            GAS_BASE_COMPUTE
        )
    }
}

impl Protocol {
    fn assert_gov(&self) {
        assert_eq!(env::predecessor_account_id(), self.gov, "ERR_NO_GOVERNANCE_ADDRESS");
    }

    fn get_market_expect(&self, market_id: U64) -> Market {
        self.markets.get(market_id.into()).expect("ERR_NO_MARKET")
    }

    fn assert_unpaused(&self) {
        assert!(!self.paused, "ERR_PROTCOL_PAUSED")
    }

    fn add_liquidity(
        &mut self,
        sender: &AccountId,
        total_in: u128,
        args: serde_json::Value,
    ) {
        let parsed_args: msg_structs::AddLiquidity = msg_structs::from_args(args);
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
           
        let mut market = self.markets.get(parsed_args.market_id.into()).expect("ERR_NO_MARKET");
        assert!(!market.finalized, "ERR_FINALIZED_MARKET");
        assert!(market.end_time > ns_to_ms(env::block_timestamp()), "ERR_MARKET_ENDED");
        assert_collateral_token(&market.pool.collateral_token_id);
        
        market.pool.add_liquidity(
            &sender,
            total_in,
            weights_u128
        );
        self.markets.replace(parsed_args.market_id.into(), &market);
    }

    fn buy(
        &mut self,
        sender: &AccountId,
        collateral_in: u128, 
        args: serde_json::Value,
    ) {
        let parsed_args: msg_structs::Buy = msg_structs::from_args(args);
        let mut market = self.markets.get(parsed_args.market_id.into()).expect("ERR_NO_MARKET");
        assert!(!market.finalized, "ERR_FINALIZED_MARKET");
        assert!(market.end_time > ns_to_ms(env::block_timestamp()), "ERR_MARKET_ENDED");
        assert_collateral_token(&market.pool.collateral_token_id);
        
        market.pool.buy(
            &sender,
            collateral_in,
            parsed_args.outcome_target,
            parsed_args.min_shares_out.into()
        );

        self.markets.replace(parsed_args.market_id.into(), &market);
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