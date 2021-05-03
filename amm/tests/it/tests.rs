use crate::test_utils::*;
use near_sdk::json_types::{U64, U128};
use near_sdk::serde_json::json;
use near_sdk_sim::{to_yocto, call, view, STORAGE_AMOUNT};

fn compose_buy_args(market_id: U64, outcome: u16) -> String {
    json!({
        "function": "buy",
        "args": {
            "market_id": market_id,
            "outcome_target": outcome,
            "min_shares_out": U128(to_token_denom(8) / 10)
        }
    }).to_string()
}

#[test]
fn fee_valid_market_lp_fee_test() {
    // Init test setup 
    let (_master_account, amm, token, seeder, trader, gov) = crate::test_utils::init("carol".to_string());

    // Amount used for seeding
    let seed_amount = to_token_denom(1000);
    // Amount used for trading
    let buy_amt = to_token_denom(100);
    // Target prices ($0.50)
    let target_price_a = U128(to_token_denom(5) / 10);
    let target_price_b = U128(to_token_denom(5) / 10);
    // Computed weights from target odds
    let weights = calc_weights_from_price(vec![target_price_a, target_price_b]);
    // 2% swap fee
    let swap_fee = to_token_denom(2) / 100;
    // Create a market using test utils - which returns the market id
    let market_id: U64 = create_market(&seeder, &amm, 2, Some(U128(swap_fee)));

    assert_eq!(market_id, U64(0));

    // Setup stringfied json object with add_liquidity arguments
    let add_liquidity_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
            "weight_indication": Some(weights)
        }
    }).to_string();

    // Call transfer call on collateral token
    ft_transfer_call(&seeder, seed_amount, add_liquidity_args);

    // Get the seeder's LP token balance with the `view` macro
    let seeder_pool_balance: U128 = view!(amm.get_pool_token_balance(market_id, &seeder.account_id())).unwrap_json();
    
    // Call transfer call with buy args
    // Buy outcome 0
    ft_transfer_call(&trader, buy_amt, compose_buy_args(market_id, 0));
    // Buy outcome 1
    ft_transfer_call(&trader, buy_amt, compose_buy_args(market_id, 1));

    // Payout numerators -> outcome 0 is worth 0, outcome 1 is worth $1
    let payout_num = vec![U128(0), U128(to_token_denom(1))];

    // Call resolute market on the amm contract directly
    let res = call!(
        gov,
        amm.resolute_market(market_id, Some(payout_num.to_vec())),
        deposit = STORAGE_AMOUNT
    );
    
    // Assert that resolution passed
    assert!(res.is_ok(), "ERR_TX_FAILED");

    // claim balances for trader
    let res = call!(
        trader,
        amm.claim_earnings(market_id),
        deposit = STORAGE_AMOUNT
    );

    // Assert that claim was successful
    assert!(res.is_ok(), "ERR_TX_FAILED");

    // Fetch balances after claiming
    let trader_balance_after_claim: u128 = ft_balance_of(&trader, &trader.account_id()).into();
    let seeder_balance_after_claim: u128 = ft_balance_of(&seeder, &seeder.account_id()).into();


}