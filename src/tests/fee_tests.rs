use super::*;

#[test]
fn lp_fee_test() {
    let (master_account, amm, token, funder, joiner, trader) = init(to_yocto("1"), "alice".to_string());
    let joiner_trader_balances = to_token_denom(10000);
    let funder_balance = to_yocto("100") - joiner_trader_balances * 2;
    transfer_unsafe(&token, &funder, joiner.account_id().to_string(), to_token_denom(10000));
    transfer_unsafe(&token, &funder, trader.account_id().to_string(), to_token_denom(10000));
    
    let seed_amt = to_token_denom(1000);
    let buy_amt = to_token_denom(100);
    let target_price_a = U128(to_token_denom(5) / 10);
    let target_price_b = U128(to_token_denom(5) / 10);
    let weights = calc_weights_from_price(vec![target_price_a, target_price_b]);
    let swap_fee = to_token_denom(2) / 100;

    let pool_id: U64 = call!(
        funder,
        amm.new_pool(2, U128(swap_fee)),
        deposit = STORAGE_AMOUNT
    ).unwrap_json();

    assert_eq!(pool_id, U64(0));

    call!(
        funder,
        amm.seed_pool(pool_id, U128(seed_amt), weights),
        deposit = STORAGE_AMOUNT
    );

    let finalize_args = json!({
        "function": "finalize",
        "args": {
            "pool_id": pool_id
        }
    }).to_string();
    transfer_with_vault(&token, &funder, "amm".to_string(), seed_amt, finalize_args);

    let funder_pool_balance: U128 = view!(amm.get_pool_token_balance(pool_id, &funder.account_id())).unwrap_json();
    
    // $1000 in swaps at 2% fee
    let buy_a_args = json!({
        "function": "buy",
        "args": {
            "pool_id": pool_id,
            "outcome_target": 0,
            "min_shares_out": U128(to_token_denom(8) / 10)
        }
    }).to_string();
    let buy_b_args = json!({
        "function": "buy",
        "args": {
            "pool_id": pool_id,
            "outcome_target": 1,
            "min_shares_out": U128(to_token_denom(8) / 10)
        }
    }).to_string();

    transfer_with_vault(&token, &trader, "amm".to_string(), buy_amt, buy_a_args.to_string());
    transfer_with_vault(&token, &trader, "amm".to_string(), buy_amt, buy_b_args.to_string());
    transfer_with_vault(&token, &trader, "amm".to_string(), buy_amt, buy_a_args.to_string());
    transfer_with_vault(&token, &trader, "amm".to_string(), buy_amt, buy_b_args.to_string());
    transfer_with_vault(&token, &trader, "amm".to_string(), buy_amt, buy_a_args.to_string());
    transfer_with_vault(&token, &trader, "amm".to_string(), buy_amt, buy_b_args.to_string());
    transfer_with_vault(&token, &trader, "amm".to_string(), buy_amt, buy_a_args.to_string());
    transfer_with_vault(&token, &trader, "amm".to_string(), buy_amt, buy_b_args.to_string());
    transfer_with_vault(&token, &trader, "amm".to_string(), buy_amt, buy_a_args.to_string());
    transfer_with_vault(&token, &trader, "amm".to_string(), buy_amt, buy_b_args.to_string());

    // joiner
    let join_pool_args = json!({
        "function": "join_pool",
        "args": {
            "pool_id": pool_id
        }
    }).to_string();
    transfer_with_vault(&token, &joiner, "amm".to_string(), seed_amt, join_pool_args);

    let joiner_pool_balance: U128 = view!(amm.get_pool_token_balance(pool_id, &joiner.account_id())).unwrap_json();

    
    let expected_claimable_by_funder = to_token_denom(20);
    let claimable_by_funder: U128 = view!(amm.get_fees_withdrawable(pool_id, &funder.account_id())).unwrap_json();
    let claimable_by_joiner: U128 = view!(amm.get_fees_withdrawable(pool_id, &joiner.account_id())).unwrap_json();
    assert_eq!(U128(expected_claimable_by_funder), claimable_by_funder);
    assert_eq!(claimable_by_joiner, U128(0));

    let funder_exit_res = call!(
        funder,
        amm.exit_pool(pool_id, funder_pool_balance),
        deposit = STORAGE_AMOUNT
    );
    let joiner_exit_res = call!(
        joiner,
        amm.exit_pool(pool_id, joiner_pool_balance),
        deposit = STORAGE_AMOUNT
    );

    println!("res {:?}", joiner_exit_res);

    let funder_pool_token_balance_after_exit: U128 = view!(amm.get_pool_token_balance(pool_id, &funder.account_id())).unwrap_json();
    let joiner_pool_token_balance_after_exit: U128 = view!(amm.get_pool_token_balance(pool_id, &joiner.account_id())).unwrap_json();
    assert_eq!(funder_pool_token_balance_after_exit, U128(0));
    assert_eq!(joiner_pool_token_balance_after_exit, U128(0));
}
