mod test_utils;
use test_utils::*;
use near_sdk::json_types::{U64, U128};
use near_sdk::serde_json::json;
use near_sdk_sim::{to_yocto, call, view, STORAGE_AMOUNT};

#[test]
fn calc_buy_amount_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "carol".to_string());
    let seed_amount = to_token_denom(10);
    let half = to_token_denom(5) / 10;
    let weights = Some(vec![U128(half), U128(half)]);
    let market_id: U64 = create_market(&alice, &amm, 2, Some(U128(0)));

    assert_eq!(market_id, U64(0));
    let add_liquidity_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
            "weight_indication": weights
        }
    }).to_string();
    transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, add_liquidity_args);

    let buy_amt: U128 = view!(amm.calc_buy_amount(market_id, U128(to_token_denom(1)), 0)).unwrap_json();
    assert_eq!(u128::from(buy_amt), 1_909_090_909_090_909_091);
}

#[test]
fn calc_sell_collateral_out_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "carol".to_string());
    let seed_amount = to_token_denom(10);
    let half = to_token_denom(5) / 10;
    let weights = Some(vec![U128(half), U128(half)]);
    let market_id: U64 = create_market(&alice, &amm, 2, Some(U128(0)));

    assert_eq!(market_id, U64(0));

    let add_liquidity_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
            "weight_indication": weights
        }
    }).to_string();
    transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, add_liquidity_args);

    let collat_out: U128 = view!(amm.calc_sell_collateral_out(market_id, U128(to_token_denom(1)), 0)).unwrap_json();
    assert_eq!(u128::from(collat_out), 2_111_111_111_111_111_111);
}

#[test]
fn basic_buy_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "carol".to_string());
    let weight = to_token_denom(1) / 2;
    let seed_amount = to_token_denom(10);
    let buy_amount = to_token_denom(1);
    let invariant = to_token_denom(100);

    let market_id: U64 = create_market(&alice, &amm, 2, Some(U128(0)));
    assert_eq!(market_id, U64(0));

    let weights = Some(vec![U128(weight), U128(weight)]);
    let add_liquidity_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
            "weight_indication": weights
        }
    }).to_string();
    transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, add_liquidity_args);

    let buy_args = json!({
        "function": "buy",
        "args": {
            "market_id": market_id,
            "outcome_target": 0,
            "min_shares_out": U128(to_token_denom(15) / 10)
        }
    }).to_string();

    let buy_res = transfer_with_vault(&token, &alice, "amm".to_string(), buy_amount, buy_args);

    let seeder_balance = get_balance(&token, alice.account_id().to_string());
    assert_eq!(seeder_balance, to_yocto("1") - seed_amount - buy_amount);
    let amm_balance = get_balance(&token, "amm".to_string());
    assert_eq!(amm_balance, seed_amount + buy_amount);

    let pool_balances: Vec<U128> = view!(amm.get_pool_balances(market_id)).unwrap_json();

    let expected_target_pool_balance = invariant / 11;
    let expected_other_outcome_pool_balance = seed_amount + buy_amount;
    assert_eq!(pool_balances[0], U128(expected_target_pool_balance));
    assert_eq!(pool_balances[1], U128(expected_other_outcome_pool_balance));

    let expected_target_buyer_balance = seed_amount + buy_amount - expected_target_pool_balance;
    let expected_other_buyer_balance = 0;

    let target_buyer_balance: U128 = view!(amm.get_share_balance(&alice.account_id(), market_id, 0)).unwrap_json();
    let other_buyer_balance: U128 = view!(amm.get_share_balance(&alice.account_id(), market_id, 1)).unwrap_json();

    assert_eq!(expected_target_buyer_balance, u128::from(target_buyer_balance));
    assert_eq!(expected_other_buyer_balance, u128::from(other_buyer_balance));
}

#[test]
fn basic_sell_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "carol".to_string());
    let weight = to_token_denom(1) / 2;
    let seed_amount = to_token_denom(10);
    let buy_amount = to_token_denom(1);
    let invariant = to_token_denom(100);

    let market_id: U64 = create_market(&alice, &amm, 2, Some(U128(0)));

    assert_eq!(market_id, U64(0));

    let weights = Some(vec![U128(weight), U128(weight)]);
    let add_liquidity_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
            "weight_indication": weights
        }
    }).to_string();
    transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, add_liquidity_args);

    let buy_args = json!({
        "function": "buy",
        "args": {
            "market_id": market_id,
            "outcome_target": 0,
            "min_shares_out": U128(to_token_denom(15) / 10)
        }
    }).to_string();

    let buy_res = transfer_with_vault(&token, &alice, "amm".to_string(), buy_amount, buy_args);


    let seeder_balance = get_balance(&token, alice.account_id().to_string());
    assert_eq!(seeder_balance, to_yocto("1") - seed_amount - buy_amount);
    let amm_balance = get_balance(&token, "amm".to_string());
    assert_eq!(amm_balance, seed_amount + buy_amount);

    let expected_target_pool_balance = invariant / 11;

    let expected_target_buyer_balance = seed_amount + buy_amount - expected_target_pool_balance;
    let expected_other_buyer_balance = 0;

    let target_buyer_balance: U128 = view!(amm.get_share_balance(&alice.account_id(), market_id, 0)).unwrap_json();
    let other_buyer_balance: U128 = view!(amm.get_share_balance(&alice.account_id(), market_id, 1)).unwrap_json();

    assert_eq!(expected_target_buyer_balance, u128::from(target_buyer_balance));
    assert_eq!(expected_other_buyer_balance, u128::from(other_buyer_balance));
    let seeder_balance = get_balance(&token, alice.account_id().to_string());

    let res = call!(
        alice,
        amm.sell(market_id, U128(to_token_denom(1)), 0, U128(expected_target_buyer_balance)),
        deposit = STORAGE_AMOUNT
    );

    assert!(res.is_ok(), "ERR_SELL_TX_FAILED");

    let pool_balances: Vec<U128> = view!(amm.get_pool_balances(market_id)).unwrap_json();
    assert_eq!(pool_balances[0], U128(seed_amount));
    assert_eq!(pool_balances[1], U128(seed_amount));

    let seeder_balance = get_balance(&token, alice.account_id().to_string());
    assert_eq!(seeder_balance, to_yocto("1") - seed_amount);
    let amm_balance = get_balance(&token, "amm".to_string());
    assert_eq!(amm_balance, seed_amount);
}

// Check price after uneven swaps
#[test]
fn complex_buy_test() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "carol".to_string());
    transfer_unsafe(&token, &alice, bob.account_id().to_string(), to_token_denom(100));

    let seed_amount = to_token_denom(10);
    let buy_amount = to_token_denom(1);
    let invariant = to_token_denom(10);

    let market_id: U64 = create_market(&alice, &amm, 3, Some(U128(0)));


    let weights = calc_weights_from_price(
        vec![
            U128(to_token_denom(60)),
            U128(to_token_denom(30)),
            U128(to_token_denom(10))
        ]
    );

    assert_eq!(market_id, U64(0));

    let add_liquidity_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
            "weight_indication": Some(weights)
        }
    }).to_string();
    transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, add_liquidity_args);


    let init_balances: Vec<U128> = view!(amm.get_pool_balances(market_id)).unwrap_json();
    let init_invariant = product_of(&init_balances);

    let buy_args = json!({
        "function": "buy",
        "args": {
            "market_id": market_id,
            "outcome_target": 0,
            "min_shares_out": U128(to_token_denom(8) / 10)
        }
    }).to_string();
    transfer_with_vault(&token, &bob, "amm".to_string(), buy_amount, buy_args);

    let post_trade_balances: Vec<U128> = view!(amm.get_pool_balances(market_id)).unwrap_json();
    let post_trade_invariant = product_of(&post_trade_balances);
    assert!(init_invariant - post_trade_invariant <  1000);

    let target_pool_balance: U128 = view!(amm.get_share_balance(&"amm".to_string(), market_id, 0)).unwrap_json();
    let target_buyer_balance: U128 = view!(amm.get_share_balance(&bob.account_id(), market_id, 0)).unwrap_json();
    let inverse_balances: Vec<U128> = vec![post_trade_balances[1], post_trade_balances[2]];
    let product_of_inverse = product_of(&inverse_balances);

    let expected_pool_target_balance = test_utils::math::div_u128(post_trade_invariant, product_of_inverse);
    let expected_buyer_target_balance = u128::from(init_balances[0]) + buy_amount - expected_pool_target_balance;

    assert_eq!(U128(expected_buyer_target_balance), target_buyer_balance);
    assert_eq!(U128(expected_pool_target_balance), target_pool_balance);
}

#[test]
fn multi_sell_test() {
    // Get accounts
    let (_master_account, amm, token, lp, trader1, trader2) = init(to_yocto("1"), "carol".to_string());


    let precision = to_token_denom(1) / 100; // 1 token_cent precision

    // Fund accounts
    transfer_unsafe(&token, &lp, trader1.account_id(), to_token_denom(10));
    transfer_unsafe(&token, &lp, trader2.account_id(), to_token_denom(10));

    // Get initial balances
    let trader1_init_balance = get_balance(&token, trader1.account_id());

    // Expect trader1 to have ....
    let expected_trader1_share_bal = 1_909_090_909_090_909_091;
    // Expect trader2 to have ....
    let expected_trader2_share_bal = 1_757_575_757_575_757_576;
    let expected_trader2_final_balance = to_token_denom(991) / 100;

    // Seed / trade parameters
    let target_price = U128(to_token_denom(5) / 10);
    let seed_amount = to_token_denom(10);
    let buy_amt = to_token_denom(1);
    let weights = calc_weights_from_price(vec![target_price, target_price]);

    // Create market
    let market_id = create_market(&lp, &amm, 2, Some(U128(0)));

    // Seed market
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

    // buy 0 from trader 1 and trader 2
    let buy_a_args = json!({
        "function": "buy",
        "args": {
            "market_id": market_id,
            "outcome_target": 0,
            "min_shares_out": U128(to_token_denom(8) / 10)
        }
    }).to_string();

    transfer_with_vault(&token, &trader1, "amm".to_string(), buy_amt, buy_a_args.to_string());
    transfer_with_vault(&token, &trader2, "amm".to_string(), buy_amt, buy_a_args.to_string());

    let trader1_share_balance: U128 = view!(amm.get_share_balance(&trader1.account_id(), market_id, 0)).unwrap_json();
    let trader2_share_balance: U128 = view!(amm.get_share_balance(&trader2.account_id(), market_id, 0)).unwrap_json();
    assert_eq!(trader1_share_balance, U128(expected_trader1_share_bal));
    assert_eq!(trader2_share_balance, U128(expected_trader2_share_bal));

    // Sell back from trader 1 and trader 2
    let sell_res_trader1 = call!(
        trader1,
        amm.sell(market_id, U128(buy_amt + buy_amt / 13), 0, U128(buy_amt * 25 / 10)),
        deposit = STORAGE_AMOUNT
    );

    let sell_res_trader2 = call!(
        trader2,
        amm.sell(market_id, U128(to_token_denom(9115) / 10000), 0, U128(buy_amt * 25 / 10)),
        deposit = STORAGE_AMOUNT
    );
    // Check balances with escrow both ways
    // Get updated balances
    let trader1_final_balance = get_balance(&token, trader1.account_id());
    let trader2_final_balance = get_balance(&token, trader2.account_id());
    // Assert balances
    assert!(trader1_init_balance - trader1_final_balance < precision);
    assert!(trader2_final_balance - expected_trader2_final_balance < precision);
}


#[test]
fn complex_sell_with_fee_test() {
    // Get accounts
    let (_master_account, amm, token, lp, trader1, trader2) = init(to_yocto("1"), "carol".to_string());

    let precision = to_token_denom(1) / 100; // 1 token_cent precision

    // Fund accounts
    transfer_unsafe(&token, &lp, trader1.account_id(), to_token_denom(10));
    transfer_unsafe(&token, &lp, trader2.account_id(), to_token_denom(10));

    // Get initial balances
    let trader1_init_balance = get_balance(&token, trader1.account_id());

    let expected_trader1_share_bal = 1_872_531_876_138_433_515;

    // Seed / trade parameters
    let target_price = U128(to_token_denom(5) / 10);
    let seed_amount = to_token_denom(10);
    let buy_amt = to_token_denom(1);
    let weights = calc_weights_from_price(vec![target_price, target_price]);

    // Create market
    let market_id = create_market(&lp, &amm, 2, Some(swap_fee()));

    // Seed market
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

    // buy 0 from trader 1
    let buy_a_args = json!({
        "function": "buy",
        "args": {
            "market_id": market_id,
            "outcome_target": 0,
            "min_shares_out": U128(to_token_denom(8) / 10)
        }
    }).to_string();

    transfer_with_vault(&token, &trader1, "amm".to_string(), buy_amt, buy_a_args.to_string());

    let trader1_share_balance: U128 = view!(amm.get_share_balance(&trader1.account_id(), market_id, 0)).unwrap_json();
    assert_eq!(trader1_share_balance, U128(expected_trader1_share_bal));


    // Sell back from trader 1 and trader 2
    let sell_res_trader1 = call!(
        trader1,
        amm.sell(market_id, U128(959159302164807332), 0, U128(buy_amt * 25 / 10)),
        deposit = STORAGE_AMOUNT
    );
}


#[test]
fn selling_uneven_lp_shares_binary_test() {
    // Get accounts
    let (_master_account, amm, token, lp, trader1, trader2) = init(to_yocto("1"), "carol".to_string());

    // Seed / trade parameters
    let seed_amount = to_token_denom(10);
    let amount_out_expected = to_token_denom(932) / 1000;
    let balance_after_seed = 3_333_333_333_333_333_333_u128;
    let weights = calc_weights_from_price(vec![U128(to_token_denom(55) / 100), U128(to_token_denom(45) / 100)]);

    // Create market
    let market_id = create_market(&lp, &amm, 2, Some(swap_fee()));

    // Seed market
    let add_liquidity_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
            "weight_indication": weights
        }
    }).to_string();
    transfer_with_vault(&token, &lp, "amm".to_string(), seed_amount, add_liquidity_args);

    let outcome_balance_0: U128 = view!(amm.get_share_balance(&lp.account_id(), market_id, 0)).unwrap_json();
    let outcome_balance_1: U128 = view!(amm.get_share_balance(&lp.account_id(), market_id, 1)).unwrap_json();

    let sell_res_lp = call!(
        lp,
        amm.sell(market_id, U128(amount_out_expected), 0, U128(balance_after_seed)),
        deposit = STORAGE_AMOUNT
    );


    assert!(sell_res_lp.is_ok());

}

#[test]
fn selling_uneven_lp_shares_categorical_test() {
    // Get accounts
    let (_master_account, amm, token, lp, trader1, trader2) = init(to_yocto("1"), "carol".to_string());

    // Seed / trade parameters
    let seed_amount = to_token_denom(10);
    let amount_out_expected = 838_054_961_715_504_818;
    let balance_after_seed = 3_333_333_333_333_333_333;
    let weights = vec![U128(12_000_000_000), U128(12_000_000_000), U128(18_000_000_000), U128(18_000_000_000)];

    // Create market
    let market_id = create_market(&lp, &amm, 4, Some(swap_fee()));

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

    let sell_res_lp = call!(
        lp,
        amm.sell(market_id, U128(amount_out_expected), 0, U128(balance_after_seed)),
        deposit = STORAGE_AMOUNT
    );

    assert!(sell_res_lp.is_ok());
}

#[test]
fn redeem_collat_with_bought_tokens_for_higher_price() {
    let (master_account, amm, token, alice, bob, carol) = init(to_yocto("1"), "carol".to_string());

    let bob_amount = to_token_denom(10000);
    transfer_unsafe(&token, &alice, bob.account_id().to_string(), bob_amount);
    let seed_amount = to_token_denom(10);
    let buy_amount = to_token_denom(1);

    let market_id: U64 = create_market(&alice, &amm, 2, Some(U128(0)));
    assert_eq!(market_id, U64(0));

    let token_value_80_20 = 1227272727272727273;
    let target_price_a = U128(to_token_denom(80) / 100);
    let target_price_b = U128(to_token_denom(20) / 100);
    let weights = calc_weights_from_price(vec![target_price_a, target_price_b]);

    // add initial iquidity with unequal weights
    let add_liquidity_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
            "weight_indication": weights
        }
    }).to_string();
    transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, add_liquidity_args);

    //  buy outcome target 0 tokens
    let buy_args = json!({
        "function": "buy",
        "args": {
            "market_id": market_id,
            "outcome_target": 0,
            "min_shares_out": U128(0)
        }
    }).to_string();

    let buy_res = transfer_with_vault(&token, &bob, "amm".to_string(), buy_amount, buy_args);
    let expected_target_buyer_balance = token_value_80_20;
    let expected_other_buyer_balance = 0;

    let target_buyer_balance: U128 = view!(amm.get_share_balance(&bob.account_id(), market_id, 0)).unwrap_json();
    let other_buyer_balance: U128 = view!(amm.get_share_balance(&bob.account_id(), market_id, 1)).unwrap_json();

    assert_eq!(expected_target_buyer_balance, u128::from(target_buyer_balance));
    assert_eq!(expected_other_buyer_balance, u128::from(other_buyer_balance));

    // remove initial liquidity
    let liq_exit = call!(
        alice,
        amm.exit_pool(market_id, U128(seed_amount)),
        deposit = STORAGE_AMOUNT
    );
    assert!(liq_exit.is_ok());

    // add liquidity with unequal weights reversed
    let weights = calc_weights_from_price(vec![target_price_b, target_price_a]);
    let add_liquidity_args = json!({
        "function": "add_liquidity",
        "args": {
            "market_id": market_id,
            "weight_indication": weights
        }
    }).to_string();
    transfer_with_vault(&token, &alice, "amm".to_string(), seed_amount, add_liquidity_args);

    // buy outcome target 1 tokens
    let buy_args = json!({
        "function": "buy",
        "args": {
            "market_id": market_id,
            "outcome_target": 1,
            "min_shares_out": U128(0)
        }
    }).to_string();

    let buy_res = transfer_with_vault(&token, &bob, "amm".to_string(), buy_amount, buy_args);
    let expected_target_buyer_balance = token_value_80_20;
    let expected_other_buyer_balance = token_value_80_20;

    let target_buyer_balance: U128 = view!(amm.get_share_balance(&bob.account_id(), market_id, 0)).unwrap_json();
    let other_buyer_balance: U128 = view!(amm.get_share_balance(&bob.account_id(), market_id, 1)).unwrap_json();

    assert_eq!(expected_target_buyer_balance, u128::from(target_buyer_balance));
    assert_eq!(expected_other_buyer_balance, u128::from(other_buyer_balance));

    // Redeem liquidity
    let redeem_call = call!(
        bob,
        amm.burn_outcome_tokens_redeem_collateral(market_id, U128(token_value_80_20)),
        deposit = STORAGE_AMOUNT
    );
    if !redeem_call.is_ok() {
        panic!("redeem failed: {:?}", redeem_call);
    }

    // Assert collateral balance
    // bob bought 2 times, and redeemed 1.22 again (loss of 0.8 tokens)
    let expected_collateral_balance = bob_amount - (buy_amount * 2) + token_value_80_20;
    let collateral_balance = get_balance(&token, bob.account_id());
    assert_eq!(collateral_balance, expected_collateral_balance);

    // todo
    // check pool states?
}