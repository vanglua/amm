mod test_utils;
use test_utils::*;
use near_sdk::json_types::{U64, U128};
use near_sdk::serde_json::json;
use near_sdk_sim::{to_yocto, call, view, STORAGE_AMOUNT};

#[test]
fn test_valid_market_resolution() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "carol".to_string());

    let market_id = create_market(&alice, &amm, 2, Some(U128(0)));
    let target_price = U128(to_token_denom(5) / 10);
    let seed_amount = to_token_denom(100);
    let buy_amt = to_token_denom(1);
    let weights = calc_weights_from_price(vec![target_price, target_price]);
    
    let add_liquidity_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
            "weight_indication": weights
        }
    }).to_string();
    transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, add_liquidity_args);

    let payout_num = vec![U128(0), U128(to_token_denom(1))];
    
    let res = call!(
        carol,
        amm.resolute_market(market_id, Some(payout_num.to_vec())),
        deposit = STORAGE_AMOUNT
    );
    
    assert!(res.is_ok(), "ERR_TX_FAILED");
    
    let res = call!(
        carol,
        amm.resolute_market(market_id, Some(payout_num)),
        deposit = STORAGE_AMOUNT
    );

    assert!(!res.is_ok(), "ERR_TX_SHOULD_HAVE_FAILED");
}

#[test]
fn test_valid_market_payout() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "carol".to_string());
    transfer_unsafe(&token, &alice, bob.account_id().to_string(), to_token_denom(10000));
    let market_id = create_market(&alice, &amm, 2, Some(U128(0)));
    let target_price = U128(to_token_denom(5) / 10);
    let seed_amount = to_token_denom(100);
    let buy_amt = to_token_denom(1);
    let weights = calc_weights_from_price(vec![target_price, target_price]);

    let add_liquidity_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
            "weight_indication": weights
        }
    }).to_string();
    transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, add_liquidity_args);

    let payout_num = vec![U128(0), U128(to_token_denom(1))];
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
    transfer_with_vault(&token, &bob, "amm".to_string(), buy_amt, buy_a_args.to_string());
    transfer_with_vault(&token, &bob, "amm".to_string(), buy_amt, buy_b_args.to_string());
    transfer_with_vault(&token, &bob, "amm".to_string(), buy_amt, buy_a_args.to_string());
    transfer_with_vault(&token, &bob, "amm".to_string(), buy_amt, buy_b_args.to_string());
    let pre_claim_balance = get_balance(&token, bob.account_id());
    let outcome_balance_0: U128 = view!(amm.get_share_balance(&bob.account_id(), market_id, 0)).unwrap_json();
    let outcome_balance_1: U128 = view!(amm.get_share_balance(&bob.account_id(), market_id, 0)).unwrap_json();

    assert_eq!(pre_claim_balance, to_token_denom(10000) - buy_amt * 4, "unexpected balance");

    let res = call!(
        carol,
        amm.resolute_market(market_id, Some(payout_num.to_vec())),
        deposit = STORAGE_AMOUNT
    );
    
    assert!(res.is_ok(), "ERR_TX_FAILED");

    let res = call!(
        bob,
        amm.claim_earnings(market_id),
        deposit = STORAGE_AMOUNT
    );

    assert!(res.is_ok(), "ERR_TX_FAILED");
    
    let claimer_balance = get_balance(&token, bob.account_id());
    let expected_claimer_balance = 10_000_019_603_038_518_995_487_u128;
    assert_eq!(claimer_balance, expected_claimer_balance, "unexpected payout");
    
    let res = call!(
        bob,
        amm.claim_earnings(market_id),
        deposit = STORAGE_AMOUNT
    );

    let second_claim_balance = get_balance(&token, bob.account_id());
    let expected_second_claim_balance = 10_000_019_603_038_518_995_487_u128;
    assert_eq!(second_claim_balance, expected_second_claim_balance, "unexpected payout");
}


#[test]
fn test_invalid_market_payout() {
        // Get accounts
        let (_master_account, amm, token, lp, trader, gov) = init(to_yocto("1"), "carol".to_string());
        
        // Fund accounts
        transfer_unsafe(&token, &lp, trader.account_id(), to_token_denom(10000));

        // Get initial balances
        let lp_init_balance = get_balance(&token, lp.account_id());
        let trader_init_balance = get_balance(&token, trader.account_id());
        let fees = 0;
        
        // Calc expected balances after invalid resolution
        // Expect bob to have init_bal - fees
        let expected_lp_final_balance = lp_init_balance + fees;
        // Expect LP to have init_bal + fees
        let expected_trader_final_balance = trader_init_balance - fees;
        // Expect amm to have a balance of 0
        let expected_amm_final_balance = 0;
        
        // Seed / trade parameters
        let target_price = U128(to_token_denom(5) / 10);
        let seed_amount = to_token_denom(100);
        let buy_amt = to_token_denom(1);
        let weights = calc_weights_from_price(vec![target_price, target_price]);

        // Create market
        let market_id = create_market(&lp, &amm, 2, Some(U128(0)));
        

        let add_liquidity_args = json!({
            "function": "add_liquidity",
            "args": {
                "market_id": market_id,
                "weight_indication": weights
            }
        }).to_string();
        transfer_with_vault(&token, &lp, "amm".to_string(), seed_amount, add_liquidity_args);

        let amm_final_balance = get_balance(&token, "amm".to_string());
        assert_eq!(amm_final_balance, seed_amount);

        // Trade with trader x amount of times
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
        
        transfer_with_vault(&token, &trader, "amm".to_string(), buy_amt, buy_a_args.to_string());
        transfer_with_vault(&token, &trader, "amm".to_string(), buy_amt, buy_b_args.to_string());
        transfer_with_vault(&token, &trader, "amm".to_string(), buy_amt, buy_a_args.to_string());
        transfer_with_vault(&token, &trader, "amm".to_string(), buy_amt, buy_b_args.to_string()); 

        // Sell back for buy amount
        let sell_res = call!(
            trader,
            amm.sell(market_id, U128(buy_amt), 0, U128(buy_amt * 25 / 10)),
            deposit = STORAGE_AMOUNT
        );

        // LP exits his position
        let lp_exit_res = call!(
            lp,
            amm.exit_pool(market_id, seed_amount.into()),
            deposit = STORAGE_AMOUNT
        );
        
        // Resolute market
        let resolution_res = call!(
            gov,
            amm.resolute_market(market_id, None),
            deposit = STORAGE_AMOUNT
        );

        // Claim earnings
        let lp_claim_res = call!(
            lp,
            amm.claim_earnings(market_id),
            deposit = STORAGE_AMOUNT
        );

        let trader_claim_res = call!(
            trader,
            amm.claim_earnings(market_id),
            deposit = STORAGE_AMOUNT
        );

        // Get updated balances
        let lp_final_balance = get_balance(&token, lp.account_id());
        let trader_final_balance = get_balance(&token, trader.account_id());
        let amm_final_balance = get_balance(&token, "amm".to_string());
        
        // Assert balances
        assert_eq!(lp_final_balance, expected_lp_final_balance);
        assert_eq!(trader_final_balance, expected_trader_final_balance);
        assert_eq!(amm_final_balance, expected_amm_final_balance);
}