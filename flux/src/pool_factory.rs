use near_sdk::AccountId;
use crate::pool::Pool;

pub fn new_pool(
    pool_id: u64,
    outcomes: u16,
    collateral_token_id: AccountId,
    swap_fee: u128,
) -> Pool {
    Pool::new(
        pool_id,
        collateral_token_id,
        outcomes,
        swap_fee.into()
    )
}