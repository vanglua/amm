use near_sdk::AccountId;
use crate::pool::Pool;

pub fn new_pool(
    pool_id: u64,
    outcomes: u16,
    collateral_token_id: AccountId,
    collateral_decimals: u32,
    swap_fee: u128,
) -> Pool {
    Pool::new(
        pool_id,
        collateral_token_id,
        collateral_decimals,
        outcomes,
        swap_fee
    )
}