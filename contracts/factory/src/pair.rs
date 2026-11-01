// Import necessary types from the Soroban SDK
#![allow(unused)]
use soroban_sdk::{contractclient, contracttype, contracterror, xdr::ToXdr, Address, Bytes, BytesN, Env};

/// Client for the single pair method the factory calls.
///
/// Upstream generated this client with `contractimport!` from the compiled
/// pair WASM, which forced the pair to be built before the factory. The ABI is
/// identical: `initialize(factory, token_0, token_1)`; a pair error aborts the
/// whole `create_pair` invocation exactly as before.
#[contractclient(name = "Client")]
pub trait PairInitialize {
    fn initialize(e: Env, factory: Address, token_0: Address, token_1: Address);
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PairError {
    /// SoroswapFactory: token_a and token_b have identical addresses
    CreatePairIdenticalTokens = 901,
}


#[contracttype]
#[derive(Clone)]
pub struct Pair(Address, Address);
impl Pair {
    pub fn new(a: Address, b: Address) -> Result<Self, PairError> {
        if a == b {
            return Err(PairError::CreatePairIdenticalTokens);
        }
        if a < b {
            Ok(Pair(a, b))
        } else {
            Ok(Pair(b, a))
        }
    }

    pub fn salt(&self, e: &Env) -> BytesN<32> {
        let mut salt = Bytes::new(e);

        // Append the bytes of token_0 and token_1 to the salt
        salt.append(&self.0.clone().to_xdr(e)); // can be simplified to salt.append(&self.clone().to_xdr(e)); but changes the hash
        salt.append(&self.1.clone().to_xdr(e));

        // Hash the salt using SHA256 to generate a new BytesN<32> value
        e.crypto().sha256(&salt).into()
    }

    pub fn token_0(&self) -> &Address {
        &self.0
    }

    pub fn token_1(&self) -> &Address {
        &self.1
    }
}

// Define a function to create a new contract instance
pub fn create_contract(
    /*
        Overall, this function is designed to create a new contract
        instance on the blockchain with the given pair_wasm_hash
        value and a unique salt value generated from the token_a and
        token_b values. The salt value is used to ensure that each
        contract instance is unique and can be identified by its hash value.

        The deployer() method of the Env instance is used to actually
        create and deploy the new contract instance. The function returns
        the hash value of the newly created contract instance as a
        BytesN<32> value.
    */
    e: &Env,                    // Pass in the current environment as an argument
    pair_wasm_hash: BytesN<32>, // Pass in the hash of the token contract's WASM file
    token_pair: &Pair,
) -> Address {
    // Return the hash of the newly created contract as a Address value

    // Use the deployer() method of the current environment to create a new contract instance
    e.deployer()
        .with_current_contract(token_pair.salt(&e)) // Use the salt as a unique identifier for the new contract instance
        .deploy_v2(pair_wasm_hash, ()) // Deploy the new contract instance (no constructor arguments)
}
