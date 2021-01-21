use near_sdk::{
    env,
    AccountId,
	json_types::{
        U64,
        U128,
    },
    serde_json::json
};

use crate::flux_protocol::{ Market };
use crate::pool::{ Pool };
use crate::vault_token::{ MintableFungibleTokenVault };

// NEW_POOL env log
pub fn log_pool(pool: &Pool) {
	env::log(
		json!({
			"type": "pools".to_string(),
			"params": {
                "id": U64(pool.id),
                "seed_nonce": U64(pool.seed_nonce),
                "owner": pool.owner,
                "outcomes": pool.outcomes,
                "swap_fee": U128(pool.swap_fee),
                "collateral_token_id": pool.collateral_token_id,
                "total_withdrawn_fees": U128(pool.total_withdrawn_fees),
                "fee_pool_weight": U128(pool.fee_pool_weight),
                "public": pool.public,
                "block_height": U64(env::block_index()),
			}
		})
		.to_string()
		.as_bytes()
	);
}

// NEW_MARKET env log
pub fn log_market(
    id: U64,
    description: String,  
    extra_info: String,  
    outcome_tags: Vec<String>,  
    market: &Market
) {
	env::log(
		json!({
			"type": "markets".to_string(),
			"params": {
                "id": id,
                "description": description,
                "extra_info": extra_info,
                "outcome_tags": outcome_tags,
                "payout_numerator": market.payout_numerator,
                "finalized": market.finalized
			}
		})
		.to_string()
		.as_bytes()
	);
}

pub fn log_outcome_token_status(outcome: u16, pool: &Pool, outcome_token: &MintableFungibleTokenVault) {
    env::log(
		json!({
			"type": "token_statuses".to_string(),
			"params": {
                "pool_id": pool.id,
                "outcome_id": outcome,
                "total_supply": U128(outcome_token.total_supply()),
                "block_height": U64(env::block_index()),
			}
		})
		.to_string()
		.as_bytes()
	);
}

// NEW_OWNER

// LOG_JOIN
// LOG_EXIT
pub fn log_user_pool_status(pool: &Pool, account_id: &AccountId, total_in: u128) {
    env::log(
		json!({
			"type": "user_pool_statuses".to_string(),
			"params": {
                "id": format!("ups_{}_{}", account_id, pool.id),
                "pool_id": pool.id,
                "account_id": account_id,
                "total_in": U128(total_in),
                "block_height": U64(env::block_index()),
			}
		})
		.to_string()
		.as_bytes()
	);
}

enum SwapType {
    Sell,
    Buy,
}

pub fn log_exit_pool(pool: &Pool, account_id: &AccountId, pool_tokens_in: u128, fees_earned: u128) {
    env::log(
		json!({
			"type": "pool_exits".to_string(),
			"params": {
                "pool_id": pool.id,
                "account_id": account_id,
                "pool_tokens_in": U128(pool_tokens_in),
                "block_height": U64(env::block_index()),
                "fees_earned": U128(fees_earned),
			}
		})
		.to_string()
		.as_bytes()
	);
}

fn log_swap(pool: &Pool, account_id: &AccountId, outcome: u16, input: u128, output: u128, fee: u128, swap_type: &SwapType) {
    let swap_type_str = match swap_type {
        SwapType::Buy => "buy",
        SwapType::Sell => "sell",
    };
    
    env::log(
		json!({
			"type": "swaps".to_string(),
			"params": {
                "pool_id": pool.id,
                "block_height": U64(env::block_index()),
                "account_id": account_id,
                "outcome_id": outcome,
                "input": U128(input),
                "output": U128(output),
                "fee": U128(fee),
                "type": swap_type_str,
			}
		})
		.to_string()
		.as_bytes()
	);
}

pub fn log_buy(pool: &Pool, account_id: &AccountId, outcome: u16, amount_in: u128, shares_out: u128, fee: u128) {
    log_swap(pool, account_id, outcome, amount_in, shares_out, fee, &SwapType::Buy);
}

pub fn log_sell(pool: &Pool, account_id: &AccountId, outcome: u16, shares_in: u128, amount_out: u128, fee: u128, to_escrow: u128) {
    log_swap(pool, account_id, outcome, shares_in, amount_out - to_escrow, fee, &SwapType::Sell);
}

fn log_to_escrow(table: String, market_id: u64, sender: &AccountId, amount: u128){ 
    json!({
        "type": table,
        "params": {
            "market_id": U64(market_id),
            "claimer": sender,
            "payout": U128(amount),
        }
    });
}


pub fn log_to_invalid_escrow(market_id: u64, sender: &AccountId, amount: u128){ 
    log_to_escrow("invalid_escrow".to_string(), market_id, sender, amount);
}
pub fn log_to_valid_escrow(market_id: u64, sender: &AccountId, amount: u128){ 
    log_to_escrow("valid_escrow".to_string(), market_id, sender, amount);
}

pub fn log_claim_earnings(
    market_id: U64,
    claimer: AccountId,
    payout: u128
) {
    env::log(
		json!({
			"type": "claims".to_string(),
			"params": {
                "market_id": market_id,
                "claimer": claimer,
                "payout": U128(payout),
			}
		})
		.to_string()
		.as_bytes()
	);
}

pub fn log_market_resoluted(
    market_id: U64,
    payout_numerator: &Option<Vec<U128>>,
) {
    env::log(
		json!({
			"type": "claims".to_string(),
			"params": {
                "market_id": market_id,
                "payout_numerator": payout_numerator,
			}
		})
		.to_string()
		.as_bytes()
	);
}

// LOG_CALL
