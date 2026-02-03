//! Consensus hardfork types for the OP Stack.
//!
//! This crate provides network upgrade transaction builders for various
//! OP Stack hardforks including Ecotone, Fjord, Isthmus, Jovian, and Interop.

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod traits;
pub use traits::Hardfork;

mod forks;
pub use forks::Hardforks;

mod fjord;
pub use fjord::Fjord;

mod ecotone;
pub use ecotone::Ecotone;

mod isthmus;
pub use isthmus::Isthmus;

mod interop;
pub use interop::Interop;

mod jovian;
pub use jovian::Jovian;

mod utils;
pub(crate) use utils::upgrade_to_calldata;

#[cfg(test)]
mod test_utils;
