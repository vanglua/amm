use near_sdk::{
    serde_json,
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

use serde_json::json;

const GAS_BASE_COMPUTE: Gas = 5_000_000_000_000;
const GAS_FOR_CALLBACK: Gas = GAS_BASE_COMPUTE;
const GAS_FOR_PROMISE: Gas = 5_000_000_000_000;
const GAS_FOR_DATA_DEPENDENCY: Gas = 10_000_000_000_000;
const GAS_FOR_REMAINING_COMPUTE: Gas = 2 * GAS_FOR_PROMISE + GAS_FOR_DATA_DEPENDENCY + GAS_BASE_COMPUTE;

/// Safe identifier.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Copy)]
#[serde(crate = "near_sdk::serde")]
pub struct VaultId(pub u64);

impl VaultId {
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

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
        vault_id: VaultId,
        payload: String,
    ) -> Promise;
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn resolve_vault(&mut self, vault_id: VaultId, sender_id: AccountId) -> U128;
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Token {
    pub accounts: LookupMap<AccountId, Balance>,
    pub total_supply: Balance,
}

impl Token {
    pub fn new(pool_id: u64) -> Self {
        Self {
            total_supply: 0,
            accounts: LookupMap::new(format!("balance:token:{}", pool_id).as_bytes().to_vec()),
        }
    }

    pub fn mint(&mut self, amount: u128, account_id: &AccountId) {
        self.total_supply += amount;
        let account_balance = self.accounts.get(account_id).unwrap_or(0);
        let new_balance = account_balance + amount;
        self.accounts.insert(account_id, &new_balance);
    }

    pub fn faux_burn(&mut self, amount: u128) {
        self.total_supply -= amount;
    }

    pub fn deposit(&mut self, receiver_id: &AccountId, amount: u128) {
        assert!(amount > 0, "Cannot deposit 0 or lower");

        let receiver_balance = self.accounts.get(&receiver_id).unwrap_or(0);
        self.accounts.insert(&receiver_id, &(receiver_balance + amount));
    }

    pub fn withdraw(&mut self, sender_id: &AccountId, amount: u128) {
        let sender_balance = self.accounts.get(&sender_id).unwrap_or(0);

        assert!(amount > 0, "Cannot withdraw 0 or lower");
        assert!(sender_balance >= amount, "Not enough balance");

        self.accounts.insert(&sender_id, &(sender_balance - amount));
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleTokenVault {
    token: Token,
    vaults: LookupMap<VaultId, Vault>,
    next_vault_id: VaultId,
}

impl Default for FungibleTokenVault {
    fn default() -> Self {
        panic!("Contract should be initialized before usage")
    }
}

impl FungibleTokenVault {
    pub fn new(pool_id: u64) -> Self {
        Self {
            token: Token::new(pool_id),
            vaults: LookupMap::new(format!("vault:token:{}", pool_id).as_bytes().to_vec()),
            next_vault_id: VaultId(0),
        }
    }

    pub fn get_balance(&self, account_id: &AccountId) -> u128 {
        self.token.accounts.get(account_id).unwrap_or(0)
    }

    pub fn total_supply(&self) -> u128 {
        self.token.total_supply
    }

    pub fn mint(&mut self, amount: u128, account_id: &AccountId) {
        self.token.mint(amount, account_id);
    }

    pub fn faux_burn(&mut self, amount: u128) {
        self.token.faux_burn(amount);
    }

    pub fn transfer_unsafe(&mut self, receiver_id: &AccountId, amount: u128) {
        self.token.withdraw(&env::predecessor_account_id(), amount);
        self.token.deposit(receiver_id, amount);
    }

    pub fn transfer_with_safe(&mut self, receiver_id: &AccountId, amount: u128, payload: String) -> Promise {
        let gas_to_receiver = env::prepaid_gas().saturating_sub(GAS_FOR_REMAINING_COMPUTE + GAS_FOR_CALLBACK);
        let vault_id = self.next_vault_id;
        let sender_id = env::predecessor_account_id();

        self.token.withdraw(&sender_id, amount);
        self.next_vault_id = vault_id.next();

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

    pub fn resolve_vault(&mut self, vault_id: VaultId, sender_id: &AccountId) -> u128 {
        assert_eq!(env::current_account_id(), env::predecessor_account_id(), "Private method can only be called by contract");

        let vault = self.vaults.remove(&vault_id).expect("Vault does not exist");

        env::log(json!({
            "type": U128(vault.balance),
        }).to_string().as_bytes());

        if vault.balance > 0 {
            self.token.deposit(sender_id, vault.balance);
        }

        vault.balance
    }

    pub fn withdraw_from_vault(&mut self, vault_id: VaultId, receiver_id: &AccountId, amount: u128) {
        env::log(json!({
            "type": "Withdrawing money"
        }).to_string().as_bytes());

        let mut vault = self.vaults.get(&vault_id).expect("Vault does not exist");
        assert!(env::predecessor_account_id() == vault.receiver_id, "Access of vault denied");
        
        let amount_to_withdraw: u128 = amount;
        assert!(amount_to_withdraw < vault.balance, "Not enough balance inside vault");

        vault.balance -= amount_to_withdraw;
        self.vaults.insert(&vault_id, &vault);
        self.token.deposit(receiver_id, amount);
    }
}