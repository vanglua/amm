#![allow(clippy::needless_pass_by_value)]

use near_sdk::{
    json_types::{
        U128,
    },
    serde::{
        Serialize,
        Deserialize,
    },
    ext_contract,
    AccountId,
    Gas,
    Balance,
    collections::{
		LookupMap,
	},
    Promise,
    env,
    borsh::{
        self,
        BorshDeserialize,
        BorshSerialize,
    },
};

use crate::logger;

const GAS_BASE_COMPUTE: Gas = 5_000_000_000_000;
const GAS_FOR_CALLBACK: Gas = GAS_BASE_COMPUTE;
const GAS_FOR_PROMISE: Gas = 5_000_000_000_000;
const GAS_FOR_DATA_DEPENDENCY: Gas = 10_000_000_000_000;
const GAS_FOR_REMAINING_COMPUTE: Gas = 2 * GAS_FOR_PROMISE + GAS_FOR_DATA_DEPENDENCY + GAS_BASE_COMPUTE;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Vault {
    pub sender_id: AccountId,
    pub receiver_id: AccountId,
    pub balance: Balance,
}

#[ext_contract(ext_token_receiver)]
trait ExtTokenReceiver {
    fn on_receive_with_vault(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        vault_id: u64,
        payload: String,
    ) -> Promise;
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn resolve_vault(&mut self, vault_id: u64, sender_id: AccountId) -> U128;
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct MintableToken {
    pub accounts: LookupMap<AccountId, Balance>,
    pub total_supply: Balance,
    pub pool_id: u64,
    pub outcome_id: u16,
}

impl MintableToken {
    pub fn new(pool_id: u64, outcome_id: u16, seed_nonce: u64, initial_supply: u128) -> Self {
        let mut accounts: LookupMap<AccountId, Balance> = LookupMap::new(format!("balance:token:{}:{}:{}", pool_id, outcome_id, seed_nonce).as_bytes().to_vec()); 
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

        assert!(balance >= amount, "ERR_LOW_BALANCE");

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
pub struct MintableFungibleTokenVault {
    token: MintableToken,
    vaults: LookupMap<u64, Vault>,
    next_vault_id: u64,
}

impl Default for MintableFungibleTokenVault {
    fn default() -> Self {
        panic!("Contract should be initialized before usage")
    }
}

impl MintableFungibleTokenVault {
    pub fn new(pool_id: u64, outcome_id: u16, seed_nonce: u64 ,initial_supply: u128,) -> Self {
        Self {
            token: MintableToken::new(pool_id, outcome_id, seed_nonce, initial_supply),
            vaults: LookupMap::new(format!("vault:token:{}:{}:{}", pool_id, outcome_id, seed_nonce).as_bytes().to_vec()),
            next_vault_id: 0,
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

    pub fn transfer_no_vault(&mut self, receiver_id: &AccountId, amount: u128) {
        self.token.withdraw(&env::predecessor_account_id(), amount);
        self.token.deposit(receiver_id, amount);
    }

    pub fn safe_transfer_from_internal(&mut self, sender: &AccountId, receiver_id: &AccountId, amount: u128) {
        self.token.withdraw(sender, amount);
        self.token.deposit(receiver_id, amount);
    }

    pub fn transfer_with_vault(&mut self, receiver_id: &AccountId, amount: u128, payload: String) -> Promise {
        let gas_to_receiver = env::prepaid_gas().saturating_sub(GAS_FOR_REMAINING_COMPUTE + GAS_FOR_CALLBACK);
        let vault_id = self.next_vault_id;
        let sender_id = env::predecessor_account_id();

        self.token.withdraw(&sender_id, amount);
        self.next_vault_id += 1;

        let vault = Vault {
            balance: amount,
            sender_id: sender_id.to_string(),
            receiver_id: receiver_id.to_string(),
        };

        self.vaults.insert(&vault_id, &vault);

        ext_token_receiver::on_receive_with_vault(
            sender_id.to_string(), 
            U128(amount),
            vault_id, 
            payload,
            &receiver_id,
            0,
            gas_to_receiver,
        )
        .then(ext_self::resolve_vault(
            vault_id,
            sender_id,
            &env::current_account_id(),
            0,
            GAS_FOR_CALLBACK,
        ))
    }

    pub fn resolve_vault(&mut self, vault_id: u64, sender_id: &AccountId) -> u128 {
        assert_eq!(env::current_account_id(), env::predecessor_account_id(), "Private method can only be called by contract");

        let vault = self.vaults.remove(&vault_id).expect("Vault does not exist");

        if vault.balance > 0 {
            self.token.deposit(sender_id, vault.balance);
        }

        vault.balance
    }

    pub fn withdraw_from_vault(&mut self, vault_id: u64, receiver_id: &AccountId, amount: u128) {
        let mut vault = self.vaults.get(&vault_id).expect("Vault does not exist");
        assert!(env::predecessor_account_id() == vault.receiver_id, "Access of vault denied");
        
        let amount_to_withdraw: u128 = amount;
        assert!(amount_to_withdraw < vault.balance, "Not enough balance inside vault");

        vault.balance -= amount_to_withdraw;
        self.vaults.insert(&vault_id, &vault);
        self.token.deposit(receiver_id, amount);
    }
}