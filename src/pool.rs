use near_sdk::{
    env,
    AccountId,
    collections::{
        UnorderedMap,
        LookupMap
	},
    borsh::{
        self,
        BorshDeserialize,
        BorshSerialize,
    },
};

use crate::math;
use crate::constants;

use crate::vault_token::MintableFungibleTokenVault;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Pool {
    id: u64,
    seed_nonce: u64,
    owner: AccountId,
    outcome_tokens: UnorderedMap<u16, MintableFungibleTokenVault>,
    num_of_outcomes: u16,
    collateral: u128,
    swap_fee: u128,
    withdrawn_fees: LookupMap<AccountId, u128>,
    total_withdrawn_fees: u128,
    fee_pool_weight: u128,
    finalized: bool,
    pool_token: MintableFungibleTokenVault,
}

impl Pool {
    pub fn new(
        sender: AccountId, 
        pool_id: u64, 
        num_of_outcomes: u16, 
        swap_fee: u128
    ) -> Self {
        assert!(num_of_outcomes >= constants::MIN_BOUND_TOKENS, "ERR_MIN_OUTCOMES");
        assert!(num_of_outcomes <= constants::MAX_BOUND_TOKENS, "ERR_MAX_OUTCOMES");

        Self {
            id: pool_id,
            seed_nonce: 1,
            owner: sender,
            outcome_tokens: UnorderedMap::new(format!("pool::{}", pool_id).as_bytes().to_vec()),
            finalized: false,
            num_of_outcomes,
            swap_fee,
            withdrawn_fees: LookupMap::new(format!("pool::{}", pool_id).as_bytes().to_vec()),
            total_withdrawn_fees: 0,
            fee_pool_weight: 0,
            collateral: 0,
            pool_token: MintableFungibleTokenVault::new(pool_id, num_of_outcomes, 0, 0)
        }
    }

    pub fn get_swap_fee(&self) -> u128 {
        self.swap_fee
    }

    pub fn get_share_balance(&self, account_id: &AccountId, outcome: u16) -> u128 {
        self.outcome_tokens
            .get(&outcome)
            .expect("ERR_NO_OUTCOME")
            .get_balance(account_id)
    }

    pub fn get_pool_token_balance(&self, owner_id: &AccountId) -> u128 {
        self.pool_token.get_balance(owner_id)
    }

    pub fn get_pool_balances(&self) -> Vec<u128> {
        self.outcome_tokens.iter().map(|(_outcome, token)| {
            token.get_balance(&env::current_account_id())
        }).collect()
    }

    pub fn seed_pool(
        &mut self,
        sender: &AccountId,
        total_in: u128, 
        weight_indication: &Vec<u128>
    ) {
        assert_eq!(sender, &self.owner, "ERR_NO_OWNER");
        assert!(!self.finalized, "ERR_POOL_FINALIZED");
        assert!(weight_indication.len() as u16 == self.num_of_outcomes, "ERR_INVALID_WEIGHTS");
        
        let mut outcome_tokens_to_return: Vec<u128> = vec![];
        let max_weight = weight_indication.iter().max().unwrap();

        for (i, weight) in weight_indication.iter().enumerate() {
            let remaining = math::div_u128(math::mul_u128(total_in, *weight), *max_weight);
            outcome_tokens_to_return.insert(i, total_in - remaining);
        }

        let pool_token_supply = self.pool_token.total_supply();

        if pool_token_supply > 0 {
            self.outcome_tokens.clear();
            self.burn_internal(&env::predecessor_account_id(), self.pool_token.total_supply());
        } 

        self.mint_and_transfer_outcome_tokens(
            sender, 
            total_in, 
            &outcome_tokens_to_return
        );
        
        self.mint_internal(sender, total_in);
        self.seed_nonce += 1;
    }

    pub fn join_pool(
        &mut self,
        sender: &AccountId,
        total_in: u128
    ) {
        assert!(self.finalized, "ERR_POOL_NOT_FINALIZED");
        let mut outcome_tokens_to_return: Vec<u128> = vec![];
        let pool_balances = self.get_pool_balances();
        let max_balance = pool_balances.iter().max().unwrap();
        let pool_supply = self.pool_token.total_supply();

        for (i, balance) in pool_balances.iter().enumerate() {
            let remaining = math::div_u128(math::mul_u128(total_in, *balance), *max_balance);
            outcome_tokens_to_return.insert(i, total_in - remaining);
        }

        self.mint_and_transfer_outcome_tokens(
            sender, 
            total_in, 
            &outcome_tokens_to_return
        );

        let to_mint = math::div_u128(math::mul_u128(total_in, pool_supply), *max_balance);
        self.mint_internal(sender, to_mint);
    }

    pub fn exit_pool(
        &mut self,
        sender: &AccountId,
        total_in: u128
    ) ->  u128 {
        let balances = self.get_pool_balances();
        let pool_token_supply = self.pool_token.total_supply();

        for (i, balance) in balances.iter().enumerate() {
            let outcome = i as u16;
            let send_out = math::div_u128(math::mul_u128(*balance, total_in), pool_token_supply);
            let mut token = self.outcome_tokens.get(&outcome).unwrap();
            token.safe_transfer_from_internal(&env::current_account_id(), sender, send_out);
            self.outcome_tokens.insert(&outcome, &token);
        }

        self.burn_internal(sender, total_in)
    }

    fn mint_and_transfer_outcome_tokens(
        &mut self,
        sender: &AccountId,
        total_in: u128,
        outcome_tokens_to_return: &Vec<u128>
    ) {
        for (outcome, amount) in outcome_tokens_to_return.iter().enumerate() {
            let mut outcome_token = self.outcome_tokens
            .get(&(outcome as u16))
            .unwrap_or_else(|| { MintableFungibleTokenVault::new(self.id, outcome as u16, self.seed_nonce, 0) });
            
            outcome_token.mint(& env::current_account_id(), total_in);

            if *amount > 0 { 
                outcome_token.safe_transfer_from_internal(&env::current_account_id(), sender, *amount);
            }

            self.outcome_tokens.insert(&(outcome as u16), &outcome_token);
        }
    }

    fn mint_internal(
        &mut self,
        to: &AccountId,
        amount: u128
    ) {
        self.before_pool_token_transfer(None, Some(to), amount);
        self.pool_token.mint(to, amount)
    }

    fn burn_internal(
        &mut self,
        from: &AccountId,
        amount: u128
    ) -> u128 {
        let fees = self.before_pool_token_transfer(Some(from), None, amount);
        self.pool_token.burn(from, amount);
        return fees;
    }

    fn before_pool_token_transfer(
        &mut self,
        from: Option<&AccountId>,
        to: Option<&AccountId>,
        amount: u128
    ) -> u128 {
        let mut fees = 0;
        if let Some(account_id) = from {
            fees = self.withdraw_fees(account_id);
        }

        let total_supply = self.pool_token.total_supply();
        let ineligible_fee_amount = match total_supply {
            0 => amount,
            _ => math::div_u128(math::mul_u128(self.fee_pool_weight, amount), total_supply)
        };

        // On transfer or burn
        if let Some(account_id) = from {
            let withdrawn_fees = self.withdrawn_fees.get(account_id).expect("ERR_NO_BAL");
            self.withdrawn_fees.insert(account_id, &(withdrawn_fees - ineligible_fee_amount));
            self.total_withdrawn_fees -= ineligible_fee_amount;
        } else { // On mint
            self.fee_pool_weight += ineligible_fee_amount;
        }

        // On transfer or mint
        if let Some(account_id) = to { 
            let withdrawn_fees = self.withdrawn_fees.get(account_id).unwrap_or(0);
            self.withdrawn_fees.insert(account_id, &(withdrawn_fees + ineligible_fee_amount));
            self.total_withdrawn_fees += ineligible_fee_amount;
        } else { // On burn
            self.fee_pool_weight -= ineligible_fee_amount;
        }

        return fees;
    }

    pub fn get_fees_withdrawable(&self, account_id: &AccountId) -> u128 {
        let pool_token_bal = self.pool_token.get_balance(account_id);
        let pool_token_total_supply = self.pool_token.total_supply();
        let raw_amount = math::div_u128(math::mul_u128(self.fee_pool_weight, pool_token_bal), pool_token_total_supply);
        let ineligible_fee_amount = self.withdrawn_fees.get(account_id).unwrap_or(0);
        raw_amount - ineligible_fee_amount
    }

    pub fn withdraw_fees(
        &mut self,
        account_id: &AccountId
    ) -> u128 {
        let pool_token_bal = self.pool_token.get_balance(account_id);
        let pool_token_total_supply = self.pool_token.total_supply();
        let raw_amount = math::div_u128(math::mul_u128(self.fee_pool_weight, pool_token_bal), pool_token_total_supply);
        let withdrawn_fees = self.withdrawn_fees.get(account_id).unwrap_or(0);
        let withdrawable_amount = raw_amount - withdrawn_fees;
        if withdrawable_amount > 0 {
            self.withdrawn_fees.insert(account_id, &raw_amount);
            self.total_withdrawn_fees += withdrawable_amount;
        }
        return withdrawable_amount;
    }  

    pub fn finalize(
        &mut self,
        sender: &AccountId,
        amount_in: u128
    ) -> u128{
        assert_eq!(sender, &self.owner, "ERR_NO_OWNER");
        assert_eq!(self.outcome_tokens.len() as u16, self.num_of_outcomes, "ERR_NOT_BINDED");
        assert!(amount_in >= self.pool_token.total_supply(), "ERR_INSUFFICIENT_COLLATERAL");
        self.finalized = true;
        self.pool_token.total_supply()
    }

    pub fn calc_buy_amount(
        &self, 
        collateral_in: u128, 
        outcome_target: u16
    ) -> u128 {
        assert!(outcome_target <= self.num_of_outcomes, "ERR_INVALID_OUTCOME");
        
        let outcome_tokens = &self.outcome_tokens;
        let collateral_in_minus_fees = collateral_in - math::mul_u128(collateral_in, self.swap_fee);
        let token_to_buy = outcome_tokens.get(&outcome_target).expect("ERR_NO_TOKEN");
        let token_to_buy_balance = token_to_buy.get_balance(&env::current_account_id());
        let mut new_buy_token_balance = token_to_buy_balance;

        for (outcome, token) in outcome_tokens.iter() {
            if outcome != outcome_target {
                let balance = token.get_balance(&env::current_account_id());
                let dividend = math::mul_u128(new_buy_token_balance, balance);
                let divisor = balance + collateral_in_minus_fees;

                new_buy_token_balance = math::div_u128(dividend, divisor);
            }
        }
        assert!(new_buy_token_balance > 0, "ERR_MATH_APPROX");

        token_to_buy_balance + collateral_in_minus_fees - new_buy_token_balance
    }

    pub fn calc_sell_tokens_in(
        &self, 
        collateral_out: u128, 
        outcome_target: u16
    ) -> u128 {
        assert!(outcome_target <= self.num_of_outcomes, "ERR_INVALID_OUTCOME");
        
        let outcome_tokens = &self.outcome_tokens;
        let collateral_out_plus_fees = math::div_u128(collateral_out, constants::TOKEN_DENOM - self.swap_fee);
        let token_to_sell = outcome_tokens.get(&outcome_target).expect("ERR_NO_TOKEN");
        let token_to_sell_balance = token_to_sell.get_balance(&env::current_account_id());
        let mut new_sell_token_balance = token_to_sell_balance;

        for (outcome, token) in outcome_tokens.iter() {
            if outcome != outcome_target {
                let balance = token.get_balance(&env::current_account_id());
                let dividend = math::mul_u128(new_sell_token_balance, balance);
                let divisor = balance - collateral_out_plus_fees;

                new_sell_token_balance = math::div_u128(dividend, divisor);
            }
        }
        assert!(new_sell_token_balance > 0, "ERR_MATH_APPROX");

        collateral_out_plus_fees + new_sell_token_balance - token_to_sell_balance
    }

    pub fn buy(
        &mut self,
        sender: &AccountId,
        amount_in: u128,
        outcome_target: u16,
        min_shares_out: u128
    ) {
        assert!(self.finalized, "ERR_NOT_FINALIZED");
        assert!(outcome_target < self.num_of_outcomes, "ERR_INVALID_OUTCOME");

        let shares_out = self.calc_buy_amount(amount_in, outcome_target);
        assert!(shares_out >= min_shares_out, "ERR_MIN_BUY_AMOUNT");

        // Transfer collateral in

        let fee = math::mul_u128(amount_in, self.swap_fee);
        self.fee_pool_weight += fee;

        let tokens_to_mint = amount_in - fee;
        self.add_to_pools(tokens_to_mint);

        let mut token_out = self.outcome_tokens.get(&outcome_target).expect("ERR_NO_TARGET_OUTCOME");
        token_out.safe_transfer_from_internal(&env::current_account_id(), sender, shares_out);
        self.outcome_tokens.insert(&outcome_target, &token_out);

        // Log
    }

    pub fn sell(
        &mut self,
        amount_out: u128,
        outcome_target: u16,
        max_shares_in: u128
    ) {
        assert!(self.finalized, "ERR_NOT_FINALIZED");
        assert!(outcome_target < self.num_of_outcomes, "ERR_INVALID_OUTCOME");

        let shares_in = self.calc_sell_tokens_in(amount_out, outcome_target);
        assert!(shares_in <= max_shares_in, "ERR_MAX_SELL_AMOUNT");

        let mut token_in = self.outcome_tokens.get(&outcome_target).expect("ERR_NO_TARGET_OUTCOME");
        token_in.transfer_no_vault(&env::current_account_id(), shares_in);
        self.outcome_tokens.insert(&outcome_target, &token_in);

        let fee = math::mul_u128(amount_out, self.swap_fee);
        self.fee_pool_weight += fee;

        let tokens_to_burn = amount_out + fee;
        self.remove_from_pools(tokens_to_burn);

        // Transfer collateral out

        // Log
    }

    fn add_to_pools(&mut self, amount: u128) {
        for outcome in 0..self.num_of_outcomes {
            let mut token = self.outcome_tokens.get(&outcome).expect("ERR_NO_OUTCOME");
            token.mint(&env::current_account_id(), amount);
            self.outcome_tokens.insert(&outcome, &token);
        }
    }

    fn remove_from_pools(&mut self, amount: u128) {
        for outcome in 0..self.num_of_outcomes {
            let mut token = self.outcome_tokens.get(&outcome).expect("ERR_NO_OUTCOME");
            token.burn(&env::current_account_id(), amount);
            self.outcome_tokens.insert(&outcome, &token);
        }
    }

    /**
     * Test functions
    */

    // Should be done in data layer
    pub fn get_spot_price(
        &self,
        target_outcome: u16
    ) -> u128 {

        let mut odds_weight_for_target = 0;
        let mut odds_weight_sum = 0;

        for (outcome, _) in self.outcome_tokens.iter() {
            let weight_for_outcome = self.get_odds_weight_for_outcome(outcome);
            odds_weight_sum += weight_for_outcome;

            if outcome == target_outcome {
                odds_weight_for_target = weight_for_outcome;
            }
        } 

        let ratio = math::div_u128(odds_weight_for_target, odds_weight_sum);
        let scale = math::div_u128(constants::TOKEN_DENOM, constants::TOKEN_DENOM - self.swap_fee);

        math::mul_u128(ratio, scale)
    }
    
    // Should be done in data layer
    pub fn get_spot_price_sans_fee(
        &self,
        target_outcome: u16
    ) -> u128 {
        let mut odds_weight_for_target = 0;
        let mut odds_weight_sum = 0;
        
        for (outcome, _) in self.outcome_tokens.iter() {
            let weight_for_outcome = self.get_odds_weight_for_outcome(outcome);

            odds_weight_sum += weight_for_outcome;
            
            if outcome == target_outcome {
                odds_weight_for_target = weight_for_outcome;
            }
        }

        math::div_u128(odds_weight_for_target, odds_weight_sum) 
    }

    fn get_odds_weight_for_outcome(
        &self,
        target_outcome: u16
    ) -> u128 {
        let mut odds_weight_for_target = 0;
        for (outcome, token) in self.outcome_tokens.iter() {
            if outcome != target_outcome {
                let balance = token.get_balance(&env::current_account_id());
                odds_weight_for_target = if odds_weight_for_target == 0 {
                    balance
                } else {
                    math::mul_u128(odds_weight_for_target, balance)
                };
            }
        }
        odds_weight_for_target
    }
}