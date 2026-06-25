#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/alloy-rs/core/main/assets/alloy.jpg",
    html_favicon_url = "https://raw.githubusercontent.com/alloy-rs/core/main/assets/favicon.ico"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

/// Arbitrum header `extraData` decoding types.
pub mod header;
/// Arbitrum receipt body and envelope types.
pub mod receipt;
/// reth `NodePrimitives` integration (requires `reth` feature).
#[cfg(feature = "reth")]
pub mod reth;
/// Arbitrum transaction body, envelope, and helpers.
pub mod transactions;

pub use header::{ArbHeaderDecodeError, ArbHeaderInfo};
pub use receipt::{ArbReceipt, ArbReceiptEnvelope};
pub use transactions::typed::ArbitrumTypedTransaction as ArbTypedTransaction;
pub use transactions::{ArbTxEnvelope, ArbTxType};
