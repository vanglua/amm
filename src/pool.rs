use near_sdk::{
    json_types::{ U128 },
    AccountId,
    collections::{
        UnorderedMap,
        Vector
	},
    borsh::{
        self,
        BorshDeserialize,
        BorshSerialize,
    },
};

use crate::math;
use crate::constants;
use crate::u256;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Pool {
    owner: AccountId,
    outcome_balances: UnorderedMap<u16, u128>,
    num_of_outcomes: u16,
    outcome_tokens_sum: u128,
    collateral: u128,
    swap_fee: u128,
    finalized: bool,
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
            owner: sender,
            outcome_balances: UnorderedMap::new(format!("pool::{}", pool_id).as_bytes().to_vec()),
            finalized: false,
            num_of_outcomes,
            outcome_tokens_sum: 0,
            swap_fee,
            collateral: 0,
        }
    }

    pub fn get_swap_fee(&self) -> u128 {
        self.swap_fee
    }

    // TODO: Test type convertion w/ into / iter 
    // TODO: Some rounding discrepancy when uneven amounts of money, change weight / payment factor
    pub fn bind_pool(
        &mut self,
        sender: &AccountId,
        total_in: u128,
        denorm_weights: Vec<U128>
    ) {
        assert_eq!(sender, &self.owner, "ERR_NO_OWNER");
        assert!(!self.finalized, "ERR_POOL_FINALIZED");
        assert!(denorm_weights.len() as u16 == self.num_of_outcomes, "ERR_INVALID_WEIGHTS");

        let mut total = 0;
        let total_tokens = denorm_weights.len() as u128 * total_in;
        let mut i = 0;
        for weight in denorm_weights {
            let weight_u128 = u128::from(weight);
            let token_balance = math::mul_u128(total_tokens, weight_u128);
            self.outcome_balances.insert(&(i as u16), &token_balance);
            total += weight_u128;
            i += 1;
        }
        
        assert_eq!(total, constants::TOKEN_DENOM, "ERR_WEIGHT_SUM_INVALID");

        self.outcome_tokens_sum = total_tokens;
        self.collateral = total_in;
    }

    pub fn finalize(
        &mut self,
        sender: &AccountId
    ) {
        assert_eq!(sender, &self.owner, "ERR_NO_OWNER");
        self.finalized = true;
        // Set owners pool tokens to default
        // Transfer self.collateral from owner to this contract
    }

    pub fn get_spot_price(
        &self,
        target_outcome: u16
    ) -> u128 {
        let zero = u256::from(0);

        let mut odds_weight_for_target = zero;
        let mut odds_weight_sum = zero;

        for (outcome, _) in self.outcome_balances.iter() {
            let weight_for_outcome = self.get_odds_weight_for_outcome(outcome);
            odds_weight_sum += weight_for_outcome;

            if outcome == target_outcome {
                odds_weight_for_target = weight_for_outcome;
            }
        } 

        // TODO Mul by 1 - fee
        let ratio = math::div_u256_to_u128(odds_weight_for_target, odds_weight_sum);
        let scale = math::div_u128(constants::TOKEN_DENOM, constants::TOKEN_DENOM - self.swap_fee);

        math::mul_u128(ratio, scale)
    }

    pub fn get_spot_price_sans_fee(
        &self,
        target_outcome: u16
    ) -> u128 {
        let zero = u256::from(0);

        let mut odds_weight_for_target = zero;
        let mut odds_weight_sum = zero;

        for (outcome, _) in self.outcome_balances.iter() {
            let weight_for_outcome = self.get_odds_weight_for_outcome(outcome);
            odds_weight_sum += weight_for_outcome;

            if outcome == target_outcome {
                odds_weight_for_target = weight_for_outcome;
            }
        } 

        math::div_u256_to_u128(odds_weight_for_target, odds_weight_sum) 
    }

    fn get_odds_weight_for_outcome(
        &self,
        target_outcome: u16
    ) -> u256 {
        let zero = u256::from(0);
        let mut odds_weight_for_target: u256 = zero;

        for (outcome, balance) in self.outcome_balances.iter() {
            if outcome != target_outcome {
                odds_weight_for_target = match odds_weight_for_target {
                    zero => u256::from(balance),
                    _ => math::mul_u256(odds_weight_for_target, u256::from(balance))
                };
            }
        }

        odds_weight_for_target
    }




}
