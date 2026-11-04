//! Exercise the actual release WASM through deploy_v2, without mocked auth.
use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, BytesN, Env};
use veltrixae_factory_interface::FactoryError;

const FACTORY_WASM: &[u8] =
    include_bytes!("../../../../target/wasm32v1-none/release/veltrixae_factory.wasm");

#[contract]
struct DeploymentHarness;

#[contractimpl]
impl DeploymentHarness {
    pub fn deploy(e: Env, code: BytesN<32>, setter: Address, pair: BytesN<32>) -> Address {
        e.deployer()
            .with_current_contract(BytesN::from_array(&e, &[1; 32]))
            .deploy_v2(code, (setter, pair))
    }

    pub fn deploy_empty(e: Env, code: BytesN<32>) -> Address {
        e.deployer()
            .with_current_contract(BytesN::from_array(&e, &[1; 32]))
            .deploy_v2(code, ())
    }
}

#[test]
fn factory_deployment_atomically_initializes_and_rejects_takeover() {
    let e = Env::default();
    e.budget().reset_unlimited();
    e.set_auths(&[]);
    let code = e.deployer().upload_contract_wasm(FACTORY_WASM);
    let pair = e.deployer().upload_contract_wasm(crate::test::pair::WASM);
    let setter = Address::generate(&e);
    let attacker = Address::generate(&e);
    let harness = DeploymentHarnessClient::new(&e, &e.register(DeploymentHarness, ()));
    let address = harness.deploy(&code, &setter, &pair);
    let client = crate::SoroswapFactoryClient::new(&e, &address);
    assert_eq!(client.fee_to_setter(), setter);
    assert_eq!(client.fee_to(), setter);
    assert_eq!(client.all_pairs_length(), 0);
    assert!(!client.fees_enabled());
    assert_eq!(
        client.try_initialize(&attacker, &BytesN::from_array(&e, &[9; 32])),
        Err(Ok(FactoryError::InitializeAlreadyInitialized))
    );
    assert_eq!(client.fee_to_setter(), setter);
    e.as_contract(&address, || {
        assert_eq!(crate::storage::get_pair_wasm_hash(&e).unwrap(), pair);
    });
}

#[test]
fn factory_deployment_without_constructor_arguments_rolls_back() {
    let e = Env::default();
    e.budget().reset_unlimited();
    e.set_auths(&[]);
    let code = e.deployer().upload_contract_wasm(FACTORY_WASM);
    let setter = Address::generate(&e);
    let pair = BytesN::from_array(&e, &[7; 32]);
    let harness = DeploymentHarnessClient::new(&e, &e.register(DeploymentHarness, ()));
    assert!(harness.try_deploy_empty(&code).is_err());
    // Same deployer/salt must still be available after failure; no incomplete
    // instance can be claimed by calling initialize in a later transaction.
    let address = harness.deploy(&code, &setter, &pair);
    assert_eq!(
        crate::SoroswapFactoryClient::new(&e, &address).fee_to_setter(),
        setter
    );
}
