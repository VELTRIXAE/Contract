//! Deploy actual router release WASM atomically and attempt post-deploy takeover.
use crate::error::CombinedRouterError;
use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, BytesN, Env};

const ROUTER_WASM: &[u8] =
    include_bytes!("../../../../target/wasm32v1-none/release/veltrixae_router.wasm");

#[contract]
struct DeploymentHarness;

#[contractimpl]
impl DeploymentHarness {
    pub fn deploy(e: Env, code: BytesN<32>, factory: Address) -> Address {
        e.deployer()
            .with_current_contract(BytesN::from_array(&e, &[2; 32]))
            .deploy_v2(code, (factory,))
    }

    pub fn deploy_empty(e: Env, code: BytesN<32>) -> Address {
        e.deployer()
            .with_current_contract(BytesN::from_array(&e, &[2; 32]))
            .deploy_v2(code, ())
    }
}

#[test]
fn router_deployment_atomically_initializes_and_rejects_takeover() {
    let e = Env::default();
    e.budget().reset_unlimited();
    e.set_auths(&[]);
    let code = e.deployer().upload_contract_wasm(ROUTER_WASM);
    let factory = Address::generate(&e);
    let attacker = Address::generate(&e);
    let harness = DeploymentHarnessClient::new(&e, &e.register(DeploymentHarness, ()));
    let address = harness.deploy(&code, &factory);
    let client = crate::SoroswapRouterClient::new(&e, &address);
    assert_eq!(client.get_factory(), factory);
    assert_eq!(
        client.try_initialize(&attacker),
        Err(Ok(CombinedRouterError::RouterInitializeAlreadyInitialized))
    );
    assert_eq!(client.get_factory(), factory);
}

#[test]
fn router_deployment_without_constructor_arguments_rolls_back() {
    let e = Env::default();
    e.budget().reset_unlimited();
    e.set_auths(&[]);
    let code = e.deployer().upload_contract_wasm(ROUTER_WASM);
    let factory = Address::generate(&e);
    let harness = DeploymentHarnessClient::new(&e, &e.register(DeploymentHarness, ()));
    assert!(harness.try_deploy_empty(&code).is_err());
    let address = harness.deploy(&code, &factory);
    assert_eq!(
        crate::SoroswapRouterClient::new(&e, &address).get_factory(),
        factory
    );
}
