mod test_utils;
use test_utils::*;
use near_sdk::json_types::{U64, U128};
use near_sdk::serde_json::json;
use near_sdk_sim::{to_yocto, call, view, STORAGE_AMOUNT};

#[test]
fn test_invalid_market_payout() {
        // Init and get accounts
        let (_master_account, amm, token, lp, trader1, trader2) = init(to_yocto("1"), "alice".to_string(), "carol".to_string());
        
        // Fund accounts  lp trader1 trader 1
        transfer_unsafe(&token, &lp, trader1.account_id(), to_token_denom(10000));
        transfer_unsafe(&token, &lp, trader2.account_id(), to_token_denom(10000));
        
        // Record balances before any trading happens
        let lp_init_balance = get_balance(&token, lp.account_id());
        let trader1_init_balance = get_balance(&token, trader1.account_id());
        let trader2_init_balance = get_balance(&token, trader2.account_id());
        
        // Seed / trade parameters
        let weights = vec![U128(to_token_denom(3) / 10), U128(to_token_denom(7) / 10)];
        let seed_amt = to_token_denom(10);
        let buy_amt = to_token_denom(1);

        
        // Create market
        let market_id = create_market(&lp, &amm, 2, Some(U128(0)));
        
        // Seed market w/ `seed_amt`
        call!(
            lp,
            amm.seed_pool(market_id, U128(seed_amt), weights),
            deposit = STORAGE_AMOUNT
        );
        
        // Check lp expected balance of outcome 0
        
        // Publish market
        let publish_args = json!({
            "function": "publish",
            "args": {
                "market_id": market_id
            }
        }).to_string();
        transfer_with_vault(&token, &lp, "amm".to_string(), seed_amt, publish_args);
        
        // assert expected price for 0 & 1 (70 | 30)
        let price_0: U128 = view!(amm.get_spot_price_sans_fee(market_id, 0)).unwrap_json();
        let price_1: U128 = view!(amm.get_spot_price_sans_fee(market_id, 1)).unwrap_json();
        assert_eq!(price_0, U128(to_token_denom(7) / 10));
        assert_eq!(price_1, U128(to_token_denom(3) / 10));


        // Buy `buy_amount` from trader1 buy `buy_amount` from trader2
        let buy_a_args = json!({
            "function": "buy",
            "args": {
                "market_id": market_id,
                "outcome_target": 0,
                "min_shares_out": U128(1)
            }
        }).to_string();

        // Buy some extra shares from lp accounts
        transfer_with_vault(&token, &lp, "amm".to_string(), buy_amt, buy_a_args.to_string());

        // Increase price by buying from two subsequent accounts
        transfer_with_vault(&token, &trader1, "amm".to_string(), buy_amt, buy_a_args.to_string());
        transfer_with_vault(&token, &trader2, "amm".to_string(), buy_amt, buy_a_args.to_string());

        // Sell back into pool from LP
        let sell_res = call!(
            lp,
            amm.sell(market_id, U128(25 / 10), 0, U128(buy_amt * 25 / 10)),
            deposit = STORAGE_AMOUNT
        );

        assert!(sell_res.is_ok());

        // Resolute and payout
        // Resolute market
        let resolution_res = call!(
            trader2,
            amm.resolute_market(market_id, None),
            deposit = STORAGE_AMOUNT
        );
        
        let trader1_claim_res = call!(
            trader1,
            amm.claim_earnings(market_id),
            deposit = STORAGE_AMOUNT
        );
        assert!(trader1_claim_res.is_ok());
        
        let trader2_claim_res = call!(
            trader2,
            amm.claim_earnings(market_id),
            deposit = STORAGE_AMOUNT
        );
        assert!(trader2_claim_res.is_ok());

        // Claim earnings
        let lp_claim_res = call!(
            lp,
            amm.claim_earnings(market_id),
            deposit = STORAGE_AMOUNT
        );

        assert!(lp_claim_res.is_ok());
        
        // Get updated balances
        let lp_final_balance = get_balance(&token, lp.account_id());
        let trader1_final_balance = get_balance(&token, trader1.account_id());
        let trader2_final_balance = get_balance(&token, trader2.account_id());
        let amm_final_balance = get_balance(&token, "amm".to_string());
        
        // Assert that all balances are back to where they started
        assert_eq!(lp_final_balance, lp_init_balance);
        assert_eq!(trader1_final_balance, trader1_init_balance);
        assert_eq!(trader2_final_balance, trader2_init_balance);
        assert_eq!(amm_final_balance, 0);
}