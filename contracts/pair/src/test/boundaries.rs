//! VELTRIXAE boundary, rounding and invariant tests for the pair contract.
//! Expected values are derived by hand from the formulas in lib.rs.
use crate::test::deposit::add_liquidity;
use crate::test::pair::SoroswapPairError;
use crate::test::SoroswapPairTest;

fn init(test: &SoroswapPairTest) {
    test.contract.initialize(
        &test.factory.address,
        &test.token_0.address,
        &test.token_1.address,
    );
}

fn send(test: &SoroswapPairTest, amount_0: i128, amount_1: i128) {
    if amount_0 > 0 {
        test.token_0
            .transfer(&test.user, &test.contract.address, &amount_0);
    }
    if amount_1 > 0 {
        test.token_1
            .transfer(&test.user, &test.contract.address, &amount_1);
    }
}

// ---------- initial liquidity ----------

#[test]
fn first_deposit_at_minimum_liquidity_is_rejected() {
    let test = SoroswapPairTest::setup();
    init(&test);
    // sqrt(1000 * 1000) = 1000 = MINIMUM_LIQUIDITY -> nothing left for the user.
    send(&test, 1000, 1000);
    let res = test.contract.try_deposit(&test.user);
    assert_eq!(
        res,
        Err(Ok(SoroswapPairError::DepositInsufficientFirstLiquidity))
    );
}

#[test]
fn first_deposit_just_above_minimum_liquidity() {
    let test = SoroswapPairTest::setup();
    init(&test);
    // sqrt(1001 * 1001) = 1001 -> user receives 1, 1000 are locked in the pair.
    let minted = add_liquidity(&test, &1001, &1001);
    assert_eq!(minted, 1);
    assert_eq!(test.contract.balance(&test.user), 1);
    assert_eq!(test.contract.balance(&test.contract.address), 1000);
    assert_eq!(test.contract.total_supply(), 1001);
    assert_eq!(test.contract.get_reserves(), (1001, 1001));
}

#[test]
fn first_deposit_uses_geometric_mean() {
    let test = SoroswapPairTest::setup();
    init(&test);
    // sqrt(1_000_000 * 4_000_000) = 2_000_000; minus 1000 locked.
    let minted = add_liquidity(&test, &1_000_000, &4_000_000);
    assert_eq!(minted, 1_999_000);
    assert_eq!(test.contract.total_supply(), 2_000_000);
}

#[test]
fn deposit_product_overflow_reverts_without_state_change() {
    let test = SoroswapPairTest::setup();
    init(&test);
    // 2^64 * 2^64 = 2^128 > i128::MAX: checked_mul fails and the call aborts.
    let big: i128 = 1 << 64;
    send(&test, big, big);
    let res = test.contract.try_deposit(&test.user);
    assert!(res.is_err());
    assert_eq!(test.contract.get_reserves(), (0, 0));
    assert_eq!(test.contract.total_supply(), 0);
}

// ---------- additional liquidity ----------

#[test]
fn additional_liquidity_rounds_down() {
    let test = SoroswapPairTest::setup();
    init(&test);
    add_liquidity(&test, &1_000_000, &4_000_000); // total supply 2_000_000
                                                  // shares_0 = 1 * 2e6 / 1e6 = 2, shares_1 = 3 * 2e6 / 4e6 = 1 (1.5 floored)
    let minted = add_liquidity(&test, &1, &3);
    assert_eq!(minted, 1);
}

#[test]
fn additional_liquidity_that_rounds_to_zero_is_rejected() {
    let test = SoroswapPairTest::setup();
    init(&test);
    add_liquidity(&test, &1_000_000, &4_000_000);
    // shares_1 = 1 * 2e6 / 4e6 = 0 -> min(2, 0) = 0
    send(&test, 1, 1);
    let res = test.contract.try_deposit(&test.user);
    assert_eq!(
        res,
        Err(Ok(SoroswapPairError::DepositInsufficientLiquidityMinted))
    );
}

#[test]
fn imbalanced_deposit_mints_by_the_smaller_side() {
    let test = SoroswapPairTest::setup();
    init(&test);
    add_liquidity(&test, &1_000_000, &4_000_000);
    // shares_0 = 1000 * 2e6 / 1e6 = 2000, shares_1 = 5000 * 2e6 / 4e6 = 2500
    let minted = add_liquidity(&test, &1000, &5000);
    assert_eq!(minted, 2000);
    // The excess token_1 is absorbed into the reserves.
    assert_eq!(test.contract.get_reserves(), (1_001_000, 4_005_000));
}

// ---------- withdrawals ----------

#[test]
fn minimum_liquidity_stays_locked_after_full_withdrawal() {
    let test = SoroswapPairTest::setup();
    init(&test);
    let minted = add_liquidity(&test, &10_000, &10_000);
    assert_eq!(minted, 9_000);

    test.contract
        .transfer(&test.user, &test.contract.address, &minted);
    let (out_0, out_1) = test.contract.withdraw(&test.user);
    // 10_000 * 9_000 / 10_000
    assert_eq!((out_0, out_1), (9_000, 9_000));
    assert_eq!(test.contract.get_reserves(), (1_000, 1_000));
    assert_eq!(test.contract.total_supply(), 1_000);

    // The locked 1000 shares can never be withdrawn.
    let res = test.contract.try_withdraw(&test.user);
    assert_eq!(
        res,
        Err(Ok(SoroswapPairError::WithdrawInsufficientSentShares))
    );
}

#[test]
fn withdraw_rounds_down_in_favour_of_the_pool() {
    let test = SoroswapPairTest::setup();
    init(&test);
    add_liquidity(&test, &1_000_000, &4_000_000); // supply 2_000_000
                                                  // Burning 1 share: 1e6 * 1 / 2e6 = 0 -> rejected rather than paying 0.
    test.contract
        .transfer(&test.user, &test.contract.address, &1);
    let res = test.contract.try_withdraw(&test.user);
    assert_eq!(
        res,
        Err(Ok(SoroswapPairError::WithdrawInsufficientLiquidityBurned))
    );
}

#[test]
fn withdraw_without_sending_shares_is_rejected() {
    let test = SoroswapPairTest::setup();
    init(&test);
    add_liquidity(&test, &1_000_000, &4_000_000);
    let res = test.contract.try_withdraw(&test.user);
    assert_eq!(
        res,
        Err(Ok(SoroswapPairError::WithdrawInsufficientSentShares))
    );
}

// ---------- swaps, fees and the K invariant ----------

#[test]
fn swap_fee_rounds_up_so_dust_input_buys_nothing() {
    let test = SoroswapPairTest::setup();
    init(&test);
    add_liquidity(&test, &1_000_000, &1_000_000);
    // fee = ceil(1 * 3 / 1000) = 1, so a 1-unit input contributes nothing.
    send(&test, 1, 0);
    let res = test.contract.try_swap(&0, &1, &test.user);
    assert_eq!(res, Err(Ok(SoroswapPairError::SwapKConstantNotMet)));
}

#[test]
fn swap_output_boundary_matches_constant_product_formula() {
    let test = SoroswapPairTest::setup();
    init(&test);
    add_liquidity(&test, &1_000_000, &1_000_000);
    // fee = 3, out = 997 * 1e6 / (1e6 + 997) = 996
    send(&test, 1000, 0);
    let res = test.contract.try_swap(&0, &997, &test.user);
    assert_eq!(res, Err(Ok(SoroswapPairError::SwapKConstantNotMet)));
    test.contract.swap(&0, &996, &test.user);
    assert_eq!(test.contract.get_reserves(), (1_001_000, 999_004));
}

#[test]
fn swap_cannot_drain_a_reserve() {
    let test = SoroswapPairTest::setup();
    init(&test);
    add_liquidity(&test, &1_000_000, &1_000_000);
    send(&test, 1_000_000_000, 0);
    let res = test.contract.try_swap(&0, &1_000_000, &test.user);
    assert_eq!(res, Err(Ok(SoroswapPairError::SwapInsufficientLiquidity)));
}

#[test]
fn swap_to_a_pool_token_is_rejected() {
    let test = SoroswapPairTest::setup();
    init(&test);
    add_liquidity(&test, &1_000_000, &1_000_000);
    send(&test, 1000, 0);
    let res = test.contract.try_swap(&0, &900, &test.token_0.address);
    assert_eq!(res, Err(Ok(SoroswapPairError::SwapInvalidTo)));
}

#[test]
fn k_never_decreases_across_swaps() {
    let test = SoroswapPairTest::setup();
    init(&test);
    add_liquidity(&test, &50_000_000, &80_000_000);

    let inputs: [i128; 6] = [1_000, 2_500_000, 17, 9_999_999, 333_333, 42];
    for (i, amount_in) in inputs.iter().enumerate() {
        let (r0, r1) = test.contract.get_reserves();
        let k_before = r0 * r1;
        let zero_for_one = i % 2 == 0;
        let (r_in, r_out) = if zero_for_one { (r0, r1) } else { (r1, r0) };
        let fee = (amount_in * 3 + 999) / 1000;
        let out = (amount_in - fee) * r_out / (r_in + amount_in - fee);
        if out == 0 {
            continue;
        }
        if zero_for_one {
            send(&test, *amount_in, 0);
            test.contract.swap(&0, &out, &test.user);
        } else {
            send(&test, 0, *amount_in);
            test.contract.swap(&out, &0, &test.user);
        }
        let (n0, n1) = test.contract.get_reserves();
        assert!(n0 * n1 >= k_before, "K decreased at step {}", i);
    }
}

// ---------- reserve synchronization ----------

#[test]
fn sync_and_skim_reconcile_donations() {
    let test = SoroswapPairTest::setup();
    init(&test);
    add_liquidity(&test, &1_000_000, &1_000_000);
    send(&test, 500, 700);
    assert_eq!(test.contract.get_reserves(), (1_000_000, 1_000_000));

    let before_0 = test.token_0.balance(&test.user);
    test.contract.skim(&test.user);
    assert_eq!(test.token_0.balance(&test.user), before_0 + 500);
    assert_eq!(test.contract.get_reserves(), (1_000_000, 1_000_000));

    send(&test, 10, 20);
    test.contract.sync();
    assert_eq!(test.contract.get_reserves(), (1_000_010, 1_000_020));
}
