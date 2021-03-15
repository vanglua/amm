use std::cmp::Ordering;
use near_sdk::{
    env,
    AccountId,
    json_types::{
        U128,
        U64,
    },
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
use crate::logger;

use crate::outcome_token::MintableFungibleToken;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ResolutionEscrow {
    pub valid: u128,
    pub invalid: u128
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct LPEntries {
    entries: LookupMap<u64, u128>
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Account {
    pub entries: LookupMap<u16, u128>, // Stores outcome => spend 
    pub lp_entries: LookupMap<u16, u128>,
    pub resolution_escrow: ResolutionEscrow
}

impl Account {
    pub fn new(pool_id: u64, sender: &AccountId) -> Self {
        Account {
            entries: LookupMap::new(format!("p{}ae{}", pool_id, sender).as_bytes().to_vec()),
            lp_entries: LookupMap::new(format!("p{}lp{}", pool_id, sender).as_bytes().to_vec()),
            resolution_escrow: ResolutionEscrow {
                valid: 0,
                invalid: 0
            }
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Pool {
    pub id: u64,
    pub collateral_token_id: AccountId,
    pub collateral_denomination: u128,
    pub lp_entries: LookupMap<AccountId, LPEntries>,
    pub outcomes: u16,
    pub outcome_tokens: UnorderedMap<u16, MintableFungibleToken>,
    pub pool_token: MintableFungibleToken,
    pub swap_fee: u128,
    pub withdrawn_fees: LookupMap<AccountId, u128>,
    pub total_withdrawn_fees: u128,
    pub fee_pool_weight: u128,
    pub accounts: LookupMap<AccountId, Account>,
}

impl Pool {
    pub fn new(
        pool_id: u64,
        collateral_token_id: AccountId,
        collateral_decimals: u32,
        outcomes: u16,
        swap_fee: u128
    ) -> Self {
        assert!(outcomes >= constants::MIN_BOUND_TOKENS, "ERR_MIN_OUTCOMES");
        assert!(outcomes <= constants::MAX_BOUND_TOKENS, "ERR_MAX_OUTCOMES");
        let collateral_denomination = 10_u128.pow(collateral_decimals);
        assert!(swap_fee == 0 || (swap_fee <= collateral_denomination / 20 && swap_fee >= collateral_denomination / 10_000), "ERR_INVALID_FEE");

        Self {
            id: pool_id,
            collateral_token_id,
            collateral_denomination,
            lp_entries: LookupMap::new(format!("p{}lpe", pool_id).as_bytes().to_vec()),
            outcomes,
            outcome_tokens: UnorderedMap::new(format!("p{}ot", pool_id).as_bytes().to_vec()),
            pool_token: MintableFungibleToken::new(pool_id, outcomes, 0),
            swap_fee,
            withdrawn_fees: LookupMap::new(format!("p{}wf", pool_id).as_bytes().to_vec()),
            total_withdrawn_fees: 0,
            fee_pool_weight: 0,
            accounts: LookupMap::new(format!("p{}a", pool_id).as_bytes().to_vec()),
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

    pub fn add_liquidity(
        &mut self,
        sender: &AccountId,
        total_in: u128,
        weight_indication: Option<Vec<u128>>
    ) {
        assert!(total_in >= self.min_liquidity_amount(), "ERR_MIN_LIQUIDITY_AMOUNT");
        let mut outcome_tokens_to_return: Vec<u128> = vec![];

        let to_mint = if self.pool_token.total_supply() == 0 {
            assert!(weight_indication.is_some(), "ERR_EXPECTED_WEIGHT_INDICATION");
            let weights = weight_indication.unwrap();
            assert!(weights.len() as u16 == self.outcomes, "ERR_INVALID_WEIGHTS");
            let max_weight = weights.iter().max().unwrap();

            for (i, weight) in weights.iter().enumerate() {
                let remaining = math::complex_div_u128(self.collateral_denomination, math::complex_mul_u128(self.collateral_denomination, total_in, *weight), *max_weight);
                outcome_tokens_to_return.insert(i, total_in - remaining);
            }

            total_in
        } else {
            assert!(weight_indication.is_none(), "ERR_UNEXPECTED_WEIGHT_INDICATION");

            let pool_balances = self.get_pool_balances();
            let max_balance = pool_balances.iter().max().unwrap(); // max_balance = cheapest outcome
            let pool_supply = self.pool_token.total_supply();

            for (i, balance) in pool_balances.iter().enumerate() {
                let remaining = math::complex_div_u128(self.collateral_denomination, math::complex_mul_u128(self.collateral_denomination, total_in, *balance), *max_balance); // remaining = amt_in * balance / max_balance
                outcome_tokens_to_return.insert(i, total_in - remaining);
            }

            math::complex_div_u128(self.collateral_denomination, math::complex_mul_u128(self.collateral_denomination, total_in, pool_supply), *max_balance)
        };

        self.mint_and_transfer_outcome_tokens(
            sender,
            total_in,
            &outcome_tokens_to_return
        );

        self.mint_internal(sender, to_mint);

        logger::log_pool(&self);
        logger::log_transaction(&logger::TransactionType::AddLiquidity, &sender, total_in, to_mint, U64(self.id), None);
        logger::log_user_pool_status(&self, &env::predecessor_account_id(), total_in);
    }

    fn mint_and_transfer_outcome_tokens(
        &mut self,
        sender: &AccountId,
        total_in: u128,
        outcome_tokens_to_return: &Vec<u128>
    ) {
        let mut account = self.accounts.get(sender).unwrap_or_else(||Account::new(self.id, sender));

        for (i, amount) in outcome_tokens_to_return.iter().enumerate() {
            let outcome = i as u16;

            // Calculate the amount of money spent by the users on the total minted shares
            let spent_on_outcome = total_in / self.outcomes as u128;
            let spent_on_amount_out = if amount > &0 {
                math::complex_mul_u128(self.collateral_denomination, spent_on_outcome, math::complex_div_u128(self.collateral_denomination, *amount, total_in))
            } else {
                0
            };

            // Delta needs to be used spent on outcome shares for outcome in exit pool
            let lp_entry_amount = spent_on_outcome - spent_on_amount_out;
            let prev_lp_entries = account.lp_entries.get(&outcome).unwrap_or(0);
            account.lp_entries.insert(&outcome, &(prev_lp_entries + lp_entry_amount));

            let prev_spent = account.entries.get(&outcome).unwrap_or(0);
            let spent = prev_spent + spent_on_amount_out;
            account.entries.insert(&outcome, &spent);

            logger::log_account_outcome_spent(&self, &sender, outcome, spent);

            let mut outcome_token = self.outcome_tokens
            .get(&(outcome as u16))
            .unwrap_or_else(|| { MintableFungibleToken::new(self.id, outcome as u16, 0) });
            
            outcome_token.mint(& env::current_account_id(), total_in);

            if *amount > 0 {
                outcome_token.safe_transfer_internal(&env::current_account_id(), sender, *amount);
            }

            self.accounts.insert(sender, &account);
            self.outcome_tokens.insert(&(outcome as u16), &outcome_token);
        }

        self.accounts.insert(sender, &account);
    }

    pub fn exit_pool(
        &mut self,
        sender: &AccountId,
        total_in: u128
    ) ->  u128 {

        let balances = self.get_pool_balances();
        let pool_token_supply = self.pool_token.total_supply();
        let sender_pool_token_balance = self.pool_token.get_balance(sender);

        assert!(total_in <= sender_pool_token_balance, "ERR_INSUFFICIENT_BALANCE");

        let mut account = self.accounts.get(sender).expect("ERR_NO_ACCOUNT");
        let lp_token_exit_ratio = math::complex_div_u128(self.collateral_denomination, total_in, sender_pool_token_balance);

        for (i, balance) in balances.iter().enumerate() {
            let outcome = i as u16;
            let send_out = math::complex_mul_u128(self.collateral_denomination, math::complex_div_u128(self.collateral_denomination, total_in, pool_token_supply), *balance);

            let current_spend = account.entries.get(&outcome).unwrap_or(0);

            let account_total_spent_on_outcome = account.lp_entries.get(&outcome).unwrap_or(0);
            let relative_spent = math::complex_mul_u128(self.collateral_denomination, lp_token_exit_ratio, account_total_spent_on_outcome);
            let spent = current_spend + relative_spent;
            account.entries.insert(&outcome, &spent);
            account.lp_entries.insert(&outcome, &(account_total_spent_on_outcome - relative_spent));
            logger::log_account_outcome_spent(&self, sender, outcome, spent);


            let mut token = self.outcome_tokens.get(&outcome).unwrap();
            token.safe_transfer_internal(&env::current_account_id(), sender, send_out);
            self.outcome_tokens.insert(&outcome, &token);
        }

        self.accounts.insert(&sender, &account);
        let fees = self.burn_internal(sender, total_in);
        logger::log_exit_pool(&self, sender, total_in, fees);
        fees
    }

    pub fn burn_outcome_tokens_redeem_collateral (
        &mut self,
        sender: &AccountId,
        to_burn: u128
    ) -> u128 {
        let mut account = self.accounts.get(sender).expect("ERR_NO_BALANCES");

        let avg_price_paid = self.outcome_tokens.iter().fold(0, |sum, (outcome, mut token)| {

            // Calc avg price per outcome
            let spent_on_outcome = account.entries.get(&outcome).unwrap_or_else(|| panic!("ERR_NO_ENTRIES_{}", outcome));
            let user_balance = token.get_balance(sender);
            assert!(user_balance > 0, "ERR_NO_BALANCE_OUTCOME_{}", outcome);
            let price_paid_per_share = math::complex_div_u128(self.collateral_denomination, spent_on_outcome, user_balance);

            // subtract sold off tokens from entries
            let redeemed_spent = math::complex_mul_u128(self.collateral_denomination, math::complex_div_u128(self.collateral_denomination, to_burn, user_balance), spent_on_outcome);
            let new_entry_balance = spent_on_outcome - redeemed_spent;
            account.entries.insert(&outcome, &new_entry_balance);
            logger::log_account_outcome_spent(&self, sender, outcome, new_entry_balance);

            // Burn outcome tokens accordingly 
            token.burn(sender, to_burn);

            sum + price_paid_per_share
        });

        // If the user paid less than 1 they have the right to claim the difference if the market turns out valid
        // If the users paid more than 1 they will have the right to claim the difference if the market turns out invalid
        let in_escrow = match avg_price_paid.cmp(&self.collateral_denomination) {
            std::cmp::Ordering::Greater => {
                let delta = avg_price_paid - self.collateral_denomination;
                account.resolution_escrow.invalid += math::complex_mul_u128(self.collateral_denomination, delta, to_burn) - 1;
                0
            },
            std::cmp::Ordering::Less => {
                let delta = self.collateral_denomination - avg_price_paid;
                let to_escrow = math::complex_mul_u128(self.collateral_denomination, delta, to_burn) - 1;
                account.resolution_escrow.valid += to_escrow;
                to_escrow + 1
            }, 
            std::cmp::Ordering::Equal => 0
        };

        // Store updated account
        self.accounts.insert(sender, &account);

        in_escrow
    }

    // move to view impl

    fn get_and_clear_balances(
        &mut self,
        account_id: &AccountId
    ) -> Vec<u128> {
        self.outcome_tokens.iter().map(|(_outcome, mut token)| {
            token.remove_account(account_id).unwrap_or(0)
        }).collect()
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
        fees
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
            _ => math::simple_mul_u128(total_supply, self.fee_pool_weight, amount)
        };

        // On transfer or burn
        if let Some(account_id) = from {
            let withdrawn_fees = self.withdrawn_fees.get(account_id).expect("ERR_NO_BAL");
            self.withdrawn_fees.insert(account_id, &(withdrawn_fees - ineligible_fee_amount));

            logger::log_withdrawn_fees(&self.pool_token.token, account_id, withdrawn_fees - ineligible_fee_amount);

            self.total_withdrawn_fees -= ineligible_fee_amount;
        } else { // On mint
            self.fee_pool_weight += ineligible_fee_amount;
        }

        // On transfer or mint
        if let Some(account_id) = to {
            let withdrawn_fees = self.withdrawn_fees.get(account_id).unwrap_or(0);
            self.withdrawn_fees.insert(account_id, &(withdrawn_fees + ineligible_fee_amount));

            logger::log_withdrawn_fees(&self.pool_token.token, account_id, withdrawn_fees + ineligible_fee_amount);

            self.total_withdrawn_fees += ineligible_fee_amount;
        } else { // On burn
            self.fee_pool_weight -= ineligible_fee_amount;
        }

        logger::log_pool(self);

        fees
    }

    pub fn get_fees_withdrawable(&self, account_id: &AccountId) -> u128 {
        let pool_token_bal = self.pool_token.get_balance(account_id);
        let pool_token_total_supply = self.pool_token.total_supply();
        let raw_amount = math::complex_div_u128(self.collateral_denomination, math::complex_mul_u128(self.collateral_denomination, self.fee_pool_weight, pool_token_bal), pool_token_total_supply);
        let ineligible_fee_amount = self.withdrawn_fees.get(account_id).unwrap_or(0);
        raw_amount - ineligible_fee_amount
    }

    pub fn withdraw_fees(
        &mut self,
        account_id: &AccountId
    ) -> u128 {
        let pool_token_bal = self.pool_token.get_balance(account_id);
        let pool_token_total_supply = self.pool_token.total_supply();
        let raw_amount = math::simple_mul_u128(pool_token_total_supply, self.fee_pool_weight, pool_token_bal);
        let withdrawn_fees = self.withdrawn_fees.get(account_id).unwrap_or(0);
        let withdrawable_amount = raw_amount - withdrawn_fees;
        if withdrawable_amount > 0 {
            self.withdrawn_fees.insert(account_id, &raw_amount);
            self.total_withdrawn_fees += withdrawable_amount;
            logger::log_withdrawn_fees(&self.pool_token.token, account_id, raw_amount);
        }

        withdrawable_amount
    }

    pub fn calc_buy_amount(
        &self,
        collateral_in: u128,
        outcome_target: u16
    ) -> u128 {
        assert!(outcome_target <= self.outcomes, "ERR_INVALID_OUTCOME");

        let outcome_tokens = &self.outcome_tokens;
        let collateral_in_minus_fees = collateral_in - math::complex_mul_u128(self.collateral_denomination, collateral_in, self.swap_fee);
        let token_to_buy = outcome_tokens.get(&outcome_target).expect("ERR_NO_TOKEN");
        let token_to_buy_balance = token_to_buy.get_balance(&env::current_account_id());
        let mut new_buy_token_balance = token_to_buy_balance;

        for (outcome, token) in outcome_tokens.iter() {
            if outcome != outcome_target {
                let balance = token.get_balance(&env::current_account_id());
                let dividend = math::complex_mul_u128(self.collateral_denomination, new_buy_token_balance, balance);
                let divisor = balance + collateral_in_minus_fees;

                new_buy_token_balance = math::complex_div_u128(self.collateral_denomination, dividend, divisor);
            }
        }
        assert!(new_buy_token_balance > 0, "ERR_MATH_APPROX");

        token_to_buy_balance + collateral_in_minus_fees - new_buy_token_balance
    }

    pub fn calc_sell_collateral_out(
        &self,
        collateral_out: u128,
        outcome_target: u16
    ) -> u128 {
        assert!(outcome_target <= self.outcomes, "ERR_INVALID_OUTCOME");

        let outcome_tokens = &self.outcome_tokens;
        let collateral_out_plus_fees = math::complex_div_u128(self.collateral_denomination, collateral_out, self.collateral_denomination - self.swap_fee);
        let token_to_sell = outcome_tokens.get(&outcome_target).expect("ERR_NO_TOKEN");
        let token_to_sell_balance = token_to_sell.get_balance(&env::current_account_id());
        let mut new_sell_token_balance = token_to_sell_balance;

        for (outcome, token) in outcome_tokens.iter() {
            if outcome != outcome_target {
                let balance = token.get_balance(&env::current_account_id());
                let dividend = math::complex_mul_u128(self.collateral_denomination, new_sell_token_balance, balance);
                let divisor = balance - collateral_out_plus_fees;

                new_sell_token_balance = math::complex_div_u128(self.collateral_denomination, dividend, divisor);
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

        assert!(outcome_target < self.outcomes, "ERR_INVALID_OUTCOME");

        let shares_out = self.calc_buy_amount(amount_in, outcome_target);
        assert!(shares_out >= min_shares_out, "ERR_MIN_BUY_AMOUNT");

        let mut account = self.accounts.get(sender).unwrap_or_else(||{Account::new(self.id, sender)});

        // Transfer collateral in
        let fee = math::complex_mul_u128(self.collateral_denomination, amount_in, self.swap_fee);
        self.fee_pool_weight += fee;

        let current_spend_on_outcome = account.entries.get(&outcome_target).unwrap_or(0);
        let spent = current_spend_on_outcome + amount_in - fee;
        account.entries.insert(&outcome_target, &spent);
        logger::log_account_outcome_spent(&self, sender, outcome_target, spent);

        let tokens_to_mint = amount_in - fee;
        self.add_to_pools(tokens_to_mint);

        let mut token_out = self.outcome_tokens.get(&outcome_target).expect("ERR_NO_TARGET_OUTCOME");
        token_out.safe_transfer_internal(&env::current_account_id(), sender, shares_out);
        self.outcome_tokens.insert(&outcome_target, &token_out);
        self.accounts.insert(sender, &account);

        logger::log_buy(&self, &sender, outcome_target, amount_in, shares_out, fee);
        logger::log_pool(&self);
    }

    pub fn sell(
        &mut self,
        sender: &AccountId,
        amount_out: u128,
        outcome_target: u16,
        max_shares_in: u128
    ) -> u128 {

        assert!(outcome_target < self.outcomes, "ERR_INVALID_OUTCOME");
        let shares_in = self.calc_sell_collateral_out(amount_out, outcome_target);

        assert!(shares_in <= max_shares_in, "ERR_MAX_SELL_AMOUNT");
        let mut token_in = self.outcome_tokens.get(&outcome_target).expect("ERR_NO_TARGET_OUTCOME");

        let mut account = self.accounts.get(sender).expect("ERR_NO_BALANCE");
        let spent = account.entries.get(&outcome_target).expect("ERR_NO_ENTRIES");

        let fee = math::complex_mul_u128(self.collateral_denomination, amount_out, self.swap_fee);
        let avg_price = math::complex_div_u128(self.collateral_denomination, spent, token_in.get_balance(sender));
        let sell_price = math::complex_div_u128(self.collateral_denomination, amount_out + fee, shares_in);

        token_in.transfer(&env::current_account_id(), shares_in);
        self.outcome_tokens.insert(&outcome_target, &token_in);

        self.fee_pool_weight += fee;

        let to_escrow = match (sell_price).cmp(&avg_price) {
            Ordering::Less => {
                let price_delta = avg_price - sell_price;
                let escrow_amt = math::simple_mul_u128(self.collateral_denomination, price_delta, shares_in);
                account.resolution_escrow.invalid += escrow_amt;
                logger::log_to_invalid_escrow(self.id, &sender, account.resolution_escrow.invalid);

                let new_spent = spent - (amount_out + escrow_amt) - fee;
                logger::log_account_outcome_spent(&self, &sender, outcome_target, new_spent);
                account.entries.insert(&outcome_target, &new_spent);
                0
            },
            Ordering::Greater => {
                let price_delta = sell_price - avg_price;
                let escrow_amt = math::simple_mul_u128(self.collateral_denomination, price_delta, shares_in);
                account.resolution_escrow.valid += escrow_amt;
                logger::log_to_valid_escrow(self.id, &sender, account.resolution_escrow.valid);
                let entries_to_sub = (amount_out - escrow_amt) - fee;

                if entries_to_sub > spent {
                    account.entries.insert(&outcome_target, &0);
                    logger::log_account_outcome_spent(&self, &sender, outcome_target, 0);
                } else {
                    let new_spent = spent - entries_to_sub;
                    account.entries.insert(&outcome_target, &new_spent);
                    logger::log_account_outcome_spent(&self, &sender, outcome_target, new_spent);
                }

                escrow_amt
            },
            Ordering::Equal => {
                let new_spent = spent - amount_out - fee;

                account.entries.insert(&outcome_target, &new_spent);
                logger::log_account_outcome_spent(&self, &sender, outcome_target, new_spent);
                0
            }
        };

        let tokens_to_burn = amount_out + fee;
        self.remove_from_pools(tokens_to_burn);
        self.accounts.insert(&env::predecessor_account_id(), &account);

        logger::log_sell(&self, &env::predecessor_account_id(), outcome_target, shares_in, amount_out, fee, to_escrow);
        logger::log_pool(&self);

        to_escrow
    }

    pub fn payout(
        &mut self,
        account_id: &AccountId,
        payout_numerators: &Option<Vec<U128>>
    ) -> u128 {
        let pool_token_balance = self.get_pool_token_balance(account_id);
        let fees_earned = if pool_token_balance > 0 { 
            self.exit_pool(account_id, pool_token_balance) 
        } else {
            0
        };

        let balances = self.get_and_clear_balances(account_id);
        let account = match self.accounts.get(account_id) {
            Some(account) => account,
            None => return 0
        };

        let payout = if payout_numerators.is_some() {
            payout_numerators.as_ref().unwrap().iter().enumerate().fold(0, |sum, (outcome, num)| {
                let bal = balances[outcome];
                let payout = math::complex_mul_u128(self.collateral_denomination, bal, u128::from(*num));
                sum + payout
            }) + account.resolution_escrow.valid
        } else {
            balances.iter().enumerate().fold(0, |sum, (outcome, _bal)| {                
                let spent = account.entries.get(&(outcome as u16)).unwrap_or(0);
                sum + spent
            }) + account.resolution_escrow.invalid
        };

        self.accounts.remove(&account_id);

        payout + fees_earned
    }


    fn add_to_pools(&mut self, amount: u128) {
        for outcome in 0..self.outcomes {
            let mut token = self.outcome_tokens.get(&outcome).expect("ERR_NO_OUTCOME");
            token.mint(&env::current_account_id(), amount);
            self.outcome_tokens.insert(&outcome, &token);
        }
    }

    fn remove_from_pools(&mut self, amount: u128) {
        for outcome in 0..self.outcomes {
            let mut token = self.outcome_tokens.get(&outcome).expect("ERR_NO_OUTCOME");
            token.burn(&env::current_account_id(), amount);

            self.outcome_tokens.insert(&outcome, &token);
        }
    }

    fn min_liquidity_amount(&self) -> u128 {
        self.collateral_denomination / 1_000_000
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

        let ratio = math::complex_div_u128(self.collateral_denomination, odds_weight_for_target, odds_weight_sum);
        let scale = math::complex_div_u128(self.collateral_denomination, self.collateral_denomination, self.collateral_denomination - self.swap_fee);

        math::complex_mul_u128(self.collateral_denomination, ratio, scale)
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

        if odds_weight_sum == 0 {
            return 0
        }

        math::complex_div_u128(self.collateral_denomination, odds_weight_for_target, odds_weight_sum)
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
                    math::complex_mul_u128(self.collateral_denomination, odds_weight_for_target, balance)
                };
            }
        }
        odds_weight_for_target
    }
}