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

#[test]
fn legacy_factory_uninitialized_state_demonstrates_first_caller_risk() {
    let test = SoroswapFactoryTest::setup();
    test.env.set_auths(&[]);
    // Deliberately reset legacy state remains vulnerable; production deployment
    // now initializes atomically in the constructor (tested separately).
    test.contract.initialize(&test.user, &test.pair_wasm);
    assert_eq!(test.contract.fee_to_setter(), test.user);
    assert_eq!(
        test.contract.try_initialize(&test.admin, &test.pair_wasm),
        Err(Ok(FactoryError::InitializeAlreadyInitialized))
    );
}

#[test]
fn factory_activity_refreshes_instance_and_pair_hash_ttl() {
    use soroban_sdk::testutils::{
        storage::{Instance, Persistent},
        Ledger,
    };
    let test = SoroswapFactoryTest::setup();
    test.contract.initialize(&test.admin, &test.pair_wasm);
    // Initialize one pair before advancing ledgers so its shared uploaded code
    // stays live. The factory hash VALUE TTL does not renew uploaded WASM TTL.
    test.contract
        .create_pair(&test.token_0.address, &test.token_1.address);
    // Test-token initialize does not renew its instance TTL.
    // Keep unrelated token instances/code alive while exercising factory TTL.
    for token in [&test.token_2.address, &test.token_3.address] {
        test.env
            .deployer()
            .extend_ttl(token.clone(), 3 * 17280, 3 * 17280);
    }
    let initial = test.env.as_contract(&test.contract.address, || {
        (
            test.env.storage().instance().get_ttl(),
            test.env
                .storage()
                .persistent()
                .get_ttl(&crate::storage::DataKey::PairWasmHash),
        )
    });
    test.env
        .ledger()
        .with_mut(|l| l.sequence_number += 2 * 17280);
    let aged = test.env.as_contract(&test.contract.address, || {
        (
            test.env.storage().instance().get_ttl(),
            test.env
                .storage()
                .persistent()
                .get_ttl(&crate::storage::DataKey::PairWasmHash),
        )
    });
    assert!(aged.0 < initial.0 && aged.1 < initial.1);
    test.contract
        .create_pair(&test.token_2.address, &test.token_3.address);
    let refreshed = test.env.as_contract(&test.contract.address, || {
        (
            test.env.storage().instance().get_ttl(),
            test.env
                .storage()
                .persistent()
                .get_ttl(&crate::storage::DataKey::PairWasmHash),
        )
    });
    assert!(refreshed.0 > aged.0 && refreshed.1 > aged.1);
    assert_eq!(test.contract.fee_to_setter(), test.admin);
}

#[test]
fn archived_uploaded_pair_code_fails_without_registry_mutation() {
    use soroban_sdk::testutils::Ledger;
    let test = SoroswapFactoryTest::setup();
    test.contract.initialize(&test.admin, &test.pair_wasm);
    // No pair has initialized/extended the uploaded pair WASM. The fixture's
    // default upload TTL expires before the factory's 30-day instance TTL.
    test.env
        .ledger()
        .with_mut(|l| l.sequence_number += 2 * 17280);
    // Archived-footprint access is a non-recoverable host error in SDK 22,
    // so even try_create_pair panics rather than returning a contract error.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        test.contract
            .try_create_pair(&test.token_0.address, &test.token_1.address)
    }));
    assert!(result.is_err());
    assert_eq!(test.contract.all_pairs_length(), 0);
    assert!(!test
        .contract
        .pair_exists(&test.token_0.address, &test.token_1.address));
    assert_eq!(test.contract.fee_to_setter(), test.admin);
}
