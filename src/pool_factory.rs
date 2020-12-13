use near_sdk::{
    near_bindgen,
    json_types::{
        U128, 
        U64
    },
    AccountId, 
    env,
    collections::{
        UnorderedMap
    },
    borsh::{
        BorshDeserialize,
        BorshSerialize
    }
};

use crate::pool::Pool;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct PoolFactory {
    owner: AccountId, // The owner of the contract
    nonce: u64, // Incrementing number that's used to define a pool's id
    pools: UnorderedMap<u64, Pool> // Maps pool ids to pool
}

/** 
 * @notice implement `Default` for `PoolFactory` - allowing for a default state to be set
 * @panics if the contract hasn't been initialized yet, there is no default state
 */
impl Default for PoolFactory {
    fn default() -> Self {
        panic!("ERR_CONTRACT_NOT_INITIATED")
    }
}

#[near_bindgen]
impl PoolFactory {

    /**
     * @notice Initialize the contract by setting the owner
     * @param owner The `account_id` that's going to have owner privileges
     */
    #[init]
    pub fn init(owner: AccountId) -> Self {
        assert!(!env::state_exists(), "ERR_CONTRACT_IS_INITIALIZED");
        assert!(env::is_valid_account_id(owner.as_bytes()), "ERR_INVALID_ACCOUNT_ID");
        
        Self {
            owner: owner,
            nonce: 0,
            pools: UnorderedMap::new(b"pools".to_vec())
        }
    }

    pub fn get_pool_swap_fee(&self, pool_id: U64) -> U128 {
        let pool = self.pools.get(&pool_id.into()).expect("ERR_NO_POOL");
        U128(pool.get_swap_fee())
    }

    pub fn new_pool(
        &mut self, 
        outcomes: u16, 
        swap_fee: U128
    ) -> U64 {
        let new_pool = Pool::new(env::predecessor_account_id(), self.nonce, outcomes, swap_fee.into());
        let pool_id = self.nonce;
        
        self.nonce += 1;
        
        self.pools.insert(&pool_id, &new_pool);
        pool_id.into()
    }

    pub fn bind_pool(
        &mut self, 
        pool_id: U64, 
        total_in: U128, 
        denorm_weights: Vec<U128>
    ) {
        let mut pool = self.pools.get(&pool_id.into()).expect("ERR_NO_POOL");

        pool.bind_pool(
            &env::predecessor_account_id(), 
            total_in.into(), 
            denorm_weights
        );
        
        self.pools.insert(&pool_id.into(), &pool);
    }

    // TODO: finalize

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

}