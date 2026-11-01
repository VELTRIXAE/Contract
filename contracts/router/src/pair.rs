use soroban_sdk::{contractclient, Address, Env};

/// Client for the pair methods the router calls. Replaces the upstream
/// `contractimport!` of the compiled pair WASM; the ABI is unchanged.
#[allow(dead_code)]
#[contractclient(name = "SoroswapPairClient")]
pub trait SoroswapPairInterface {
    fn deposit(e: Env, to: Address) -> i128;
    fn swap(e: Env, amount_0_out: i128, amount_1_out: i128, to: Address);
    fn withdraw(e: Env, to: Address) -> (i128, i128);
    fn get_reserves(e: Env) -> (i128, i128);
}
