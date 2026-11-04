//! VELTRIXAE router security tests: authorization, deadlines, multi-hop
//! accounting and absence of stranded funds.
use soroban_sdk::{testutils::Ledger, vec, Address, Vec};

use crate::error::CombinedRouterError;
use crate::test::add_liquidity::add_liquidity;
use crate::test::{create_token_contract, SoroswapRouterTest, TokenClient};

fn path2(test: &SoroswapRouterTest) -> Vec<Address> {
    vec![
        &test.env,
        test.token_0.address.clone(),
        test.token_1.address.clone(),
    ]
}

#[test]
fn swap_requires_signature_of_to() {
    let test = SoroswapRouterTest::setup();
    test.contract.initialize(&test.factory.address);
    add_liquidity(&test, &1_000_000_000, &4_000_000_000);
    let b0 = test.token_0.balance(&test.user);

    test.env.set_auths(&[]);
    let res = test.contract.try_swap_exact_tokens_for_tokens(
        &1_000_000,
        &0,
        &path2(&test),
        &test.user,
        &1000,
    );
    assert!(res.is_err());
    let res = test.contract.try_swap_tokens_for_exact_tokens(
        &1_000,
        &i128::MAX,
        &path2(&test),
        &test.user,
        &1000,
    );
    assert!(res.is_err());
    assert_eq!(test.token_0.balance(&test.user), b0);
}

#[test]
fn liquidity_operations_require_signature_of_to() {
    let test = SoroswapRouterTest::setup();
    test.contract.initialize(&test.factory.address);
    let (_, _, liquidity) = add_liquidity(&test, &1_000_000_000, &4_000_000_000);
    let pair = test
        .contract
        .router_pair_for(&test.token_0.address, &test.token_1.address);
    let lp = TokenClient::new(&test.env, &pair);

    test.env.set_auths(&[]);
    // Nobody can burn the user's LP tokens without the user's signature.
    let res = test.contract.try_remove_liquidity(
        &test.token_0.address,
        &test.token_1.address,
        &liquidity,
        &0,
        &0,
        &test.user,
        &1000,
    );
    assert!(res.is_err());
    let res = test.contract.try_add_liquidity(
        &test.token_0.address,
        &test.token_1.address,
        &1_000,
        &4_000,
        &0,
        &0,
        &test.user,
        &1000,
    );
    assert!(res.is_err());
    assert_eq!(lp.balance(&test.user), liquidity);
}

#[test]
fn swap_authorization_is_rooted_at_the_router_call() {
    let test = SoroswapRouterTest::setup();
    test.contract.initialize(&test.factory.address);
    add_liquidity(&test, &1_000_000_000, &4_000_000_000);
    test.contract
        .swap_exact_tokens_for_tokens(&1_000_000, &0, &path2(&test), &test.user, &1000);

    let auths = test.env.auths();
    assert_eq!(auths.len(), 1);
    assert_eq!(auths[0].0, test.user);
    match &auths[0].1.function {
        soroban_sdk::testutils::AuthorizedFunction::Contract((addr, name, _)) => {
            assert_eq!(addr, &test.contract.address);
            assert_eq!(
                name,
                &soroban_sdk::Symbol::new(&test.env, "swap_exact_tokens_for_tokens")
            );
        }
        _ => panic!("unexpected root authorization"),
    }
}

#[test]
fn deadline_equal_to_ledger_timestamp_is_expired() {
    let test = SoroswapRouterTest::setup();
    test.contract.initialize(&test.factory.address);
    add_liquidity(&test, &1_000_000_000, &4_000_000_000);
    test.env.ledger().with_mut(|li| li.timestamp = 5_000);

    let res = test.contract.try_swap_exact_tokens_for_tokens(
        &1_000_000,
        &0,
        &path2(&test),
        &test.user,
        &5_000,
    );
    assert_eq!(res, Err(Ok(CombinedRouterError::RouterDeadlineExpired)));
    let res = test.contract.try_remove_liquidity(
        &test.token_0.address,
        &test.token_1.address,
        &1,
        &0,
        &0,
        &test.user,
        &4_999,
    );
    assert_eq!(res, Err(Ok(CombinedRouterError::RouterDeadlineExpired)));
    // One second later than the ledger time is still valid.
    test.contract
        .swap_exact_tokens_for_tokens(&1_000_000, &0, &path2(&test), &test.user, &5_001);
}

#[test]
fn slippage_failure_moves_no_funds() {
    let test = SoroswapRouterTest::setup();
    test.contract.initialize(&test.factory.address);
    add_liquidity(&test, &1_000_000_000, &4_000_000_000);
    let expected = test
        .contract
        .router_get_amounts_out(&1_000_000, &path2(&test));
    let min_out = expected.get(1).unwrap() + 1;
    let (b0, b1) = (
        test.token_0.balance(&test.user),
        test.token_1.balance(&test.user),
    );

    let res = test.contract.try_swap_exact_tokens_for_tokens(
        &1_000_000,
        &min_out,
        &path2(&test),
        &test.user,
        &1000,
    );
    assert_eq!(
        res,
        Err(Ok(CombinedRouterError::RouterInsufficientOutputAmount))
    );
    assert_eq!(test.token_0.balance(&test.user), b0);
    assert_eq!(test.token_1.balance(&test.user), b1);
}

#[test]
fn three_hop_swaps_match_quotes_and_leave_nothing_in_the_router() {
    let test = SoroswapRouterTest::setup();
    test.contract.initialize(&test.factory.address);
    let t2 = create_token_contract(&test.env, &test.admin);
    let t3 = create_token_contract(&test.env, &test.admin);
    t2.mint(&test.user, &10_000_000_000_000);
    t3.mint(&test.user, &10_000_000_000_000);

    let liq = |a: &Address, b: &Address, x: i128, y: i128| {
        test.contract
            .add_liquidity(a, b, &x, &y, &0, &0, &test.user, &1000);
    };
    liq(
        &test.token_0.address,
        &test.token_1.address,
        1_000_000_000,
        2_000_000_000,
    );
    liq(
        &test.token_1.address,
        &t2.address,
        3_000_000_000,
        1_500_000_000,
    );
    liq(&t2.address, &t3.address, 5_000_000_000, 7_000_000_000);

    let path: Vec<Address> = vec![
        &test.env,
        test.token_0.address.clone(),
        test.token_1.address.clone(),
        t2.address.clone(),
        t3.address.clone(),
    ];

    // Exact input.
    let quoted = test.contract.router_get_amounts_out(&10_000_000, &path);
    let before = t3.balance(&test.user);
    let amounts =
        test.contract
            .swap_exact_tokens_for_tokens(&10_000_000, &0, &path, &test.user, &1000);
    assert_eq!(amounts, quoted);
    assert_eq!(t3.balance(&test.user) - before, amounts.get(3).unwrap());

    // Exact output.
    let quoted_in = test.contract.router_get_amounts_in(&1_000_000, &path);
    let before_0 = test.token_0.balance(&test.user);
    let amounts = test.contract.swap_tokens_for_exact_tokens(
        &1_000_000,
        &i128::MAX,
        &path,
        &test.user,
        &1000,
    );
    assert_eq!(amounts, quoted_in);
    assert_eq!(
        before_0 - test.token_0.balance(&test.user),
        amounts.get(0).unwrap()
    );

    // The router never custodies tokens.
    let router = test.contract.address.clone();
    assert_eq!(test.token_0.balance(&router), 0);
    assert_eq!(test.token_1.balance(&router), 0);
    assert_eq!(t2.balance(&router), 0);
    assert_eq!(t3.balance(&router), 0);
}

#[test]
fn initialize_cannot_be_replayed() {
    let test = SoroswapRouterTest::setup();
    test.contract.initialize(&test.factory.address);
    let res = test.contract.try_initialize(&test.user);
    assert_eq!(
        res,
        Err(Ok(CombinedRouterError::RouterInitializeAlreadyInitialized))
    );
    assert_eq!(test.contract.get_factory(), test.factory.address);
}

#[test]
fn failed_withdrawal_slippage_rolls_back_burn_transfers_and_reserves() {
    let test = SoroswapRouterTest::setup();
    test.contract.initialize(&test.factory.address);
    let (_, _, liquidity) = add_liquidity(&test, &1_000_000, &4_000_000);
    let pair_address = test
        .contract
        .router_pair_for(&test.token_0.address, &test.token_1.address);
    let pair = crate::test::SoroswapPairClient::new(&test.env, &pair_address);
    let reserves = pair.get_reserves();
    let balances = (
        test.token_0.balance(&test.user),
        test.token_1.balance(&test.user),
    );
    let pool_balances = (
        test.token_0.balance(&pair_address),
        test.token_1.balance(&pair_address),
    );
    let lp = TokenClient::new(&test.env, &pair_address);
    let lp_pool = lp.balance(&pair_address);
    // Checks occur AFTER nested LP transfer, burn and both token transfers.
    assert_eq!(
        test.contract.try_remove_liquidity(
            &test.token_0.address,
            &test.token_1.address,
            &liquidity,
            &i128::MAX,
            &0,
            &test.user,
            &1000,
        ),
        Err(Ok(CombinedRouterError::RouterInsufficientAAmount))
    );
    assert_eq!(pair.get_reserves(), reserves);
    assert_eq!(lp.balance(&test.user), liquidity);
    assert_eq!(lp.balance(&pair_address), lp_pool);
    assert_eq!(
        (
            test.token_0.balance(&test.user),
            test.token_1.balance(&test.user)
        ),
        balances
    );
    assert_eq!(
        (
            test.token_0.balance(&pair_address),
            test.token_1.balance(&pair_address)
        ),
        pool_balances
    );
}

#[test]
fn failed_initial_deposit_rolls_back_pair_creation_and_token_transfers() {
    let test = SoroswapRouterTest::setup();
    test.contract.initialize(&test.factory.address);
    let balances = (
        test.token_0.balance(&test.user),
        test.token_1.balance(&test.user),
    );
    // Minimum liquidity exhausts the entire geometric mean after factory deployment.
    assert!(test
        .contract
        .try_add_liquidity(
            &test.token_0.address,
            &test.token_1.address,
            &1000,
            &1000,
            &0,
            &0,
            &test.user,
            &1000,
        )
        .is_err());
    assert_eq!(test.factory.all_pairs_length(), 0);
    assert!(!test
        .factory
        .pair_exists(&test.token_0.address, &test.token_1.address));
    assert_eq!(
        (
            test.token_0.balance(&test.user),
            test.token_1.balance(&test.user)
        ),
        balances
    );
    // The deterministic address must also have rolled back, so retry can deploy it.
    add_liquidity(&test, &1_000_000, &4_000_000);
    assert_eq!(test.factory.all_pairs_length(), 1);
}

#[test]
fn legacy_router_uninitialized_state_demonstrates_first_caller_risk() {
    let test = SoroswapRouterTest::setup();
    test.env.set_auths(&[]);
    // Deliberately reset legacy state is vulnerable. Constructor-created WASM
    // cannot enter this state; see the separate production deployment tests.
    test.contract.initialize(&test.user);
    assert_eq!(test.contract.get_factory(), test.user);
    assert!(test.contract.try_initialize(&test.factory.address).is_err());
}

#[test]
fn release_wasm_exact_output_swap_handles_integer_fee_rounding_counterexample() {
    let test = SoroswapRouterTest::setup();
    let wasm: &[u8] =
        include_bytes!("../../../../target/wasm32v1-none/release/veltrixae_router.wasm");
    // Use the actual release router WASM with its constructor, not the native
    // fixture that synthesizes legacy uninitialized state.
    let address = test.env.register(wasm, (test.factory.address.clone(),));
    let router = crate::SoroswapRouterClient::new(&test.env, &address);
    let (rin, rout, out, gross) = (
        72_320_497_631_014_911,
        94_716_774_993_543_284,
        28_883_596_383_956_043,
        31_825_313_508_430_335,
    );
    router.add_liquidity(
        &test.token_0.address,
        &test.token_1.address,
        &rin,
        &rout,
        &0,
        &0,
        &test.user,
        &1000,
    );
    let path = path2(&test);
    let quoted = router.router_get_amounts_in(&out, &path);
    assert_eq!(quoted.get(0), Some(gross));
    let before = (
        test.token_0.balance(&test.user),
        test.token_1.balance(&test.user),
    );
    assert_eq!(
        router.try_swap_tokens_for_exact_tokens(&out, &(gross - 1), &path, &test.user, &1000),
        Err(Ok(CombinedRouterError::RouterExcessiveInputAmount))
    );
    assert_eq!(
        (
            test.token_0.balance(&test.user),
            test.token_1.balance(&test.user)
        ),
        before
    );
    let amounts = router.swap_tokens_for_exact_tokens(&out, &gross, &path, &test.user, &1000);
    assert_eq!(amounts, quoted);
    assert_eq!(before.0 - test.token_0.balance(&test.user), gross);
    assert_eq!(test.token_1.balance(&test.user) - before.1, out);
    let pair_address = test
        .factory
        .get_pair(&test.token_0.address, &test.token_1.address);
    let pair = crate::test::SoroswapPairClient::new(&test.env, &pair_address);
    assert_eq!(pair.get_reserves(), (rin + gross, rout - out));
    assert_eq!(test.token_0.balance(&pair_address), rin + gross);
    assert_eq!(test.token_1.balance(&pair_address), rout - out);
    assert!((rin + gross) * (rout - out) >= rin * rout);
}
