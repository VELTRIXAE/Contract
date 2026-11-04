#![cfg(test)]
extern crate std;
use crate::{SoroswapRouter, SoroswapRouterClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

// Token Contract
mod token {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32v1-none/release/veltrixae_test_token.wasm"
    );
    pub type TokenClient<'a> = Client<'a>;
}
pub use token::TokenClient;

pub fn create_token_contract<'a>(e: &Env, admin: &Address) -> TokenClient<'a> {
    TokenClient::new(&e, &e.register_stellar_asset_contract(admin.clone()))
}

// Pair Contract
mod pair {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/veltrixae_pair.wasm");
    pub type SoroswapPairClient<'a> = Client<'a>;
}
use pair::SoroswapPairClient;

fn pair_contract_wasm(e: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(file = "../../target/wasm32v1-none/release/veltrixae_pair.wasm");
    e.deployer().upload_contract_wasm(WASM)
}

// SoroswapFactory Contract
mod factory {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32v1-none/release/veltrixae_factory.wasm"
    );
    pub type SoroswapFactoryClient<'a> = Client<'a>;
}
use factory::SoroswapFactoryClient;

fn create_soroswap_factory<'a>(e: &Env, setter: &Address) -> SoroswapFactoryClient<'a> {
    let pair_hash = pair_contract_wasm(&e);
    let factory_address = &e.register(factory::WASM, (setter.clone(), pair_hash.clone()));
    let factory = SoroswapFactoryClient::new(e, factory_address);
    factory
}

// SoroswapRouter Contract
fn create_soroswap_router<'a>(e: &Env) -> SoroswapRouterClient<'a> {
    let address = e.register(SoroswapRouter {}, (Address::generate(e),));
    // Preserve inherited pre-initialization/error tests. This storage reset is
    // a native test fixture only, not a production method or deployment flow.
    e.as_contract(&address, || {
        crate::storage::reset_initialization_for_test(e)
    });
    SoroswapRouterClient::new(e, &address)
}

// SoroswapRouter TEST

pub struct SoroswapRouterTest<'a> {
    env: Env,
    contract: SoroswapRouterClient<'a>,
    token_0: TokenClient<'a>,
    token_1: TokenClient<'a>,
    factory: SoroswapFactoryClient<'a>,
    user: Address,
    admin: Address,
}

impl<'a> SoroswapRouterTest<'a> {
    fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        let contract = create_soroswap_router(&env);

        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        assert_ne!(admin, user);

        let mut token_0 = create_token_contract(&env, &admin);
        let mut token_1 = create_token_contract(&env, &admin);
        if &token_1.address < &token_0.address {
            std::mem::swap(&mut token_0, &mut token_1);
        }
        token_0.mint(&user, &10_000_000_000_000_000_000);
        token_1.mint(&user, &10_000_000_000_000_000_000);

        let factory = create_soroswap_factory(&env, &admin);
        env.budget().reset_unlimited();

        SoroswapRouterTest {
            env,
            contract,
            token_0,
            token_1,
            factory,
            user,
            admin,
        }
    }

    fn setup_deducted_reserve() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        let contract = create_soroswap_router(&env);

        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        assert_ne!(admin, user);

        let mut token_0 = create_token_contract(&env, &admin);
        let mut token_1 = create_token_contract(&env, &admin);
        if &token_1.address < &token_0.address {
            std::mem::swap(&mut token_0, &mut token_1);
        }

        let initial_user_balance = 24_995_705_032_704;

        token_0.mint(&user, &initial_user_balance);
        token_1.mint(&user, &initial_user_balance);

        let factory = create_soroswap_factory(&env, &admin);

        SoroswapRouterTest {
            env,
            contract,
            token_0,
            token_1,
            factory,
            user,
            admin,
        }
    }
}

// Test mods:

pub mod add_liquidity;
pub mod initialize;
//pub mod swap;
pub mod events;
pub mod library_functions;
pub mod remove_liquidity;
pub mod security;
pub mod swap_exact_tokens_for_tokens;
pub mod swap_tokens_for_exact_tokens;

// BUDGET TEST MOD
mod budget;

mod constructors;
