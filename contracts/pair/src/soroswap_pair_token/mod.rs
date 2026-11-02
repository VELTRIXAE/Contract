//#![no_std]

mod allowance;
mod balance;
mod contract;
mod metadata;
mod storage_types;
mod total_supply;

pub use contract::SoroswapPairToken;
#[cfg(test)]
pub use contract::SoroswapPairTokenClient;
pub use contract::{internal_burn, internal_mint};
pub use metadata::write_metadata;
