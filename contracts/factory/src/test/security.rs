//! VELTRIXAE security tests for the factory: administrative authorization,
//! initialization and token validation on pair creation.
extern crate std;
use crate::test::SoroswapFactoryTest;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction},
    xdr::{AccountId, PublicKey, ScAddress, Uint256},
    Address, Env, IntoVal, Symbol, TryFromVal,
};
use veltrixae_factory_interface::FactoryError;

/// A classic Stellar account (G...) address, i.e. not a contract.
fn account_address(e: &Env) -> Address {
    let sc = ScAddress::Account(AccountId(PublicKey::PublicKeyTypeEd25519(Uint256(
        [7u8; 32],
    ))));
    Address::try_from_val(e, &sc).unwrap()
}

#[test]
fn admin_functions_require_setter_signature() {
    let test = SoroswapFactoryTest::setup();
    test.contract.initialize(&test.admin, &test.pair_wasm);

    // Drop the blanket auth mock: no signatures are provided at all.
    test.env.set_auths(&[]);

    assert!(test.contract.try_set_fee_to(&test.user).is_err());
    assert!(test.contract.try_set_fee_to_setter(&test.user).is_err());
    assert!(test.contract.try_set_fees_enabled(&true).is_err());

    // State is unchanged.
    assert_eq!(test.contract.fee_to(), test.admin);
    assert_eq!(test.contract.fee_to_setter(), test.admin);
    assert_eq!(test.contract.fees_enabled(), false);
}

#[test]
fn set_fee_to_records_setter_authorization() {
    let test = SoroswapFactoryTest::setup();
    test.contract.initialize(&test.admin, &test.pair_wasm);

    test.contract.set_fee_to(&test.user);
    let auths = test.env.auths();
    assert_eq!(auths.len(), 1);
    let (signer, invocation) = &auths[0];
    assert_eq!(signer, &test.admin);
    assert_eq!(
        invocation.function,
        AuthorizedFunction::Contract((
            test.contract.address.clone(),
            Symbol::new(&test.env, "set_fee_to"),
            (test.user.clone(),).into_val(&test.env),
        ))
    );
}

#[test]
fn previous_setter_loses_rights_after_handover() {
    let test = SoroswapFactoryTest::setup();
    test.contract.initialize(&test.admin, &test.pair_wasm);
    test.contract.set_fee_to_setter(&test.user);

    test.contract.set_fees_enabled(&true);
    // The new setter, not the old admin, must have authorized the call.
    let (signer, _) = test.env.auths()[0].clone();
    assert_eq!(signer, test.user);
    assert_ne!(signer, test.admin);
}

#[test]
fn initialize_cannot_be_replayed_to_change_setter() {
    let test = SoroswapFactoryTest::setup();
    test.contract.initialize(&test.admin, &test.pair_wasm);
    let res = test.contract.try_initialize(&test.user, &test.pair_wasm);
    assert_eq!(res, Err(Ok(FactoryError::InitializeAlreadyInitialized)));
    assert_eq!(test.contract.fee_to_setter(), test.admin);
}

#[test]
fn create_pair_does_not_require_any_signature() {
    // Pair creation is permissionless by design (as in Uniswap V2 / Soroswap).
    let test = SoroswapFactoryTest::setup();
    test.contract.initialize(&test.admin, &test.pair_wasm);
    test.env.set_auths(&[]);
    test.contract
        .create_pair(&test.token_0.address, &test.token_1.address);
    assert_eq!(test.contract.all_pairs_length(), 1);
}

#[test]
fn create_pair_rejects_account_address_as_token() {
    let test = SoroswapFactoryTest::setup();
    test.contract.initialize(&test.admin, &test.pair_wasm);
    let account = account_address(&test.env);
    assert!(test
        .contract
        .try_create_pair(&test.token_0.address, &account)
        .is_err());
    assert_eq!(test.contract.all_pairs_length(), 0);
    assert_eq!(
        test.contract.pair_exists(&test.token_0.address, &account),
        false
    );
}

#[test]
fn create_pair_rejects_undeployed_contract_address() {
    let test = SoroswapFactoryTest::setup();
    test.contract.initialize(&test.admin, &test.pair_wasm);
    let not_deployed = Address::generate(&test.env);
    assert!(test
        .contract
        .try_create_pair(&test.token_0.address, &not_deployed)
        .is_err());
    assert_eq!(test.contract.all_pairs_length(), 0);
}

#[test]
fn create_pair_rejects_non_token_contract() {
    let test = SoroswapFactoryTest::setup();
    test.contract.initialize(&test.admin, &test.pair_wasm);
    // The factory itself is a contract but does not implement the token interface.
    let factory = test.contract.address.clone();
    assert!(test
        .contract
        .try_create_pair(&test.token_0.address, &factory)
        .is_err());
    assert_eq!(test.contract.all_pairs_length(), 0);
}

#[test]
fn pair_registry_is_order_independent_and_indexed() {
    let test = SoroswapFactoryTest::setup();
    test.contract.initialize(&test.admin, &test.pair_wasm);

    let p01 = test
        .contract
        .create_pair(&test.token_1.address, &test.token_0.address);
    let p23 = test
        .contract
        .create_pair(&test.token_2.address, &test.token_3.address);

    assert_eq!(
        test.contract
            .get_pair(&test.token_0.address, &test.token_1.address),
        p01
    );
    assert_eq!(
        test.contract
            .get_pair(&test.token_1.address, &test.token_0.address),
        p01
    );
    assert_eq!(test.contract.all_pairs(&0), p01);
    assert_eq!(test.contract.all_pairs(&1), p23);
    assert_eq!(test.contract.all_pairs_length(), 2);
    assert_eq!(
        test.contract.try_all_pairs(&2),
        Err(Ok(FactoryError::IndexDoesNotExist))
    );
    assert_eq!(
        test.contract
            .try_create_pair(&test.token_0.address, &test.token_1.address),
        Err(Ok(FactoryError::CreatePairAlreadyExists))
    );
}
