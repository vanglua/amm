#![allow(clippy::needless_pass_by_value)]

use near_sdk::{
    AccountId,
    Balance,
    collections::{
		LookupMap,
	},
    env,
    borsh::{
        self,
        BorshDeserialize,
        BorshSerialize,
    },
};

use crate::logger;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct MintableToken {
    pub accounts: LookupMap<AccountId, Balance>,
    pub total_supply: Balance,
    pub pool_id: u64,
    pub outcome_id: u16,
}

impl MintableToken {

    pub fn new(pool_id: u64, outcome_id: u16, initial_supply: u128) -> Self {
        let mut accounts: LookupMap<AccountId, Balance> = LookupMap::new(format!("bt:{}:{}", pool_id, outcome_id).as_bytes().to_vec()); 
        accounts.insert(&env::current_account_id(), &initial_supply);

        Self {
            total_supply: initial_supply,
            accounts,
            pool_id,
            outcome_id
        }
    }

    pub fn mint(&mut self, account_id: &AccountId, amount: u128) {
        self.total_supply += amount;
        let account_balance = self.accounts.get(account_id).unwrap_or(0);
        let new_balance = account_balance + amount;
        self.accounts.insert(account_id, &new_balance);

        logger::log_user_balance(&self, account_id, new_balance);
        logger::log_token_status(&self);
    }

    pub fn burn(&mut self, account_id: &AccountId, amount: u128) {
        let mut balance = self.accounts.get(&account_id).unwrap_or(0);

        assert!(balance >= amount, "ERR_INSUFFICIENT_BALANCE");

        balance -= amount;
        self.accounts.insert(account_id, &balance);
        self.total_supply -= amount;

        logger::log_user_balance(&self, &account_id, balance);
        logger::log_token_status(&self);
    }

    pub fn deposit(&mut self, receiver_id: &AccountId, amount: u128) {
        assert!(amount > 0, "Cannot deposit 0 or lower");

        let receiver_balance = self.accounts.get(&receiver_id).unwrap_or(0);
        let new_balance = receiver_balance + amount;

        self.accounts.insert(&receiver_id, &new_balance);
        logger::log_user_balance(&self, &receiver_id, new_balance);
    }

    pub fn withdraw(&mut self, sender_id: &AccountId, amount: u128) {
        let sender_balance = self.accounts.get(&sender_id).unwrap_or(0);

        assert!(amount > 0, "Cannot withdraw 0 or lower");
        assert!(sender_balance >= amount, "Not enough balance");

        let new_balance = sender_balance - amount;
        self.accounts.insert(&sender_id, &new_balance);
        logger::log_user_balance(&self, &sender_id, new_balance);
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct MintableFungibleToken {
    pub token: MintableToken,
}

impl Default for MintableFungibleToken {
    fn default() -> Self {
        panic!("Contract should be initialized before usage")
    }
}

impl MintableFungibleToken {
    // TODO: rm seed_nonce
    pub fn new(pool_id: u64, outcome_id: u16, initial_supply: u128,) -> Self {
        Self {
            token: MintableToken::new(pool_id, outcome_id, initial_supply),
        }
    }

    pub fn get_balance(&self, account_id: &AccountId) -> u128 {
        self.token.accounts.get(account_id).unwrap_or(0)
    }

    pub fn total_supply(&self) -> u128 {
        self.token.total_supply
    }

    pub fn mint(&mut self, account_id: &AccountId, amount: u128) {
        self.token.mint(account_id, amount);
    }

    pub fn burn(&mut self, account_id: &AccountId, amount: u128) {
        self.token.burn(account_id, amount);
    }

    pub fn remove_account(&mut self, account_id: &AccountId) -> Option<u128> {
        self.token.accounts.remove(account_id)
    }

    pub fn transfer(&mut self, receiver_id: &AccountId, amount: u128) {
        self.token.withdraw(&env::predecessor_account_id(), amount);
        self.token.deposit(receiver_id, amount);
    }

    pub fn safe_transfer_internal(&mut self, sender: &AccountId, receiver_id: &AccountId, amount: u128) {
        self.token.withdraw(sender, amount);
        self.token.deposit(receiver_id, amount);
    }

}