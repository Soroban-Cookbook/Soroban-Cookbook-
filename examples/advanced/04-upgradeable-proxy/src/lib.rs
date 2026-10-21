#![no_std]

#[cfg(any(feature = "impl-v1", test))]
pub mod implementation_v1;
#[cfg(any(feature = "impl-v2", test))]
pub mod implementation_v2;
#[cfg(any(feature = "proxy", test))]
pub mod proxy;

#[cfg(any(feature = "impl-v1", test))]
pub use implementation_v1::{ImplementationV1, ImplementationV1Client};
#[cfg(any(feature = "impl-v2", test))]
pub use implementation_v2::{ImplementationV2, ImplementationV2Client};
#[cfg(any(feature = "proxy", test))]
pub use proxy::{ProxyContract, ProxyContractClient};

#[cfg(test)]
mod test;
