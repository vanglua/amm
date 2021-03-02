mod test_utils;
use test_utils::*;
use near_sdk::json_types::{U64, U128};
use near_sdk::serde_json::json;
use near_sdk_sim::{to_yocto, call, view, STORAGE_AMOUNT};

#[test]
fn multi_lp_payout_no_exit() {
        // Get accounts
        let (master_account, amm, token, lp, trader, gov) = init("carol".to_string());

        // Get initial balances
        let lp_init_balance: u128 = init_balance();
        let buy_amount = to_yocto("1") / 10;
        let fees = math::complex_mul_u128(token_denom(), buy_amount, swap_fee().into());
        
        // Calc expected balances after invalid resolution
        // Expect bob to have init_bal - fees
        let expected_lp_final_balance = lp_init_balance + buy_amount;
        let expected_amm_final_balance = 0;
        
        // Seed / trade parameters
        let target_price_0 = U128(to_token_denom(95) / 100);
        let target_price_1 = U128(to_token_denom(5) / 100);
        let seed_amount_0 = 200000000000000000000000 ;
        let seed_amount_1 = to_token_denom(1) ;
        let weights = calc_weights_from_price(vec![target_price_1, target_price_0]);
        
        // Create market
        let market_id = create_market(&lp, &amm, 2, Some(swap_fee()));

        let add_liquidity_args_0 = json!({
            "function": "add_liquidity",
            "args": {
                "market_id": market_id,
                "weight_indication": weights
            }
        }).to_string();
        ft_transfer_call(&trader, 100 * token_denom(), add_liquidity_args_0.to_string());


        let add_liquidity_args_1 = json!({
            "function": "add_liquidity",
            "args": {
                "market_id": market_id,
            }
        }).to_string();
        // Insert buys
        let buy_a_args = json!({
            "function": "buy",
            "args": {
                "market_id": market_id,
                "outcome_target": 0,
                "min_shares_out": U128(to_token_denom(8) / 10)
            }
        }).to_string();
        let buy_b_args = json!({
            "function": "buy",
            "args": {
                "market_id": market_id,
                "outcome_target": 1,
                "min_shares_out": U128(to_token_denom(8) / 10)
            }
        }).to_string();

        ft_transfer_call(&trader, buy_amount, buy_a_args.to_string());
        ft_transfer_call(&trader, buy_amount, buy_a_args.to_string());
        ft_transfer_call(&trader, buy_amount, buy_a_args.to_string());

        ft_transfer_call(&lp, seed_amount_0, add_liquidity_args_1.to_string());
        
        ft_transfer_call(&lp, buy_amount, buy_a_args.to_string());
        ft_transfer_call(&lp, seed_amount_1, add_liquidity_args_1.to_string());
        ft_transfer_call(&lp, buy_amount, buy_a_args.to_string());

        // Resolute market
        call!(
            gov,
            amm.resolute_market(market_id, Some(vec![U128(0), U128(to_token_denom(1))])),
            deposit = STORAGE_AMOUNT
        );

        // Claim earnings
        let trader_claim_res = call!(
            trader,
            amm.claim_earnings(market_id),
            deposit = STORAGE_AMOUNT
        );

        assert!(trader_claim_res.is_ok());
        // Claim earnings
        let lp_claim_res = call!(
            lp,
            amm.claim_earnings(market_id),
            deposit = STORAGE_AMOUNT
        );

        assert!(lp_claim_res.is_ok());

        // Get updated balances
        let lp_final_balance: u128 = ft_balance_of(&master_account, &lp.account_id()).into();
        let amm_final_balance: u128 = ft_balance_of(&master_account, &"amm".to_string()).into();
        
        // Assert balances
        // assert_eq!(lp_final_balance, expected_lp_final_balance);
        assert_eq!(amm_final_balance, expected_amm_final_balance);
}