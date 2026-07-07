#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/alloy-rs/core/main/assets/alloy.jpg",
    html_favicon_url = "https://raw.githubusercontent.com/alloy-rs/core/main/assets/favicon.ico"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

/// `arbdebug_*` namespace types.
pub mod arbdebug;
/// `arbtrace_*` namespace types.
pub mod arbtrace;
/// Transaction receipt response types.
pub mod receipt;
/// Transaction request payload types.
pub mod request;
/// Timeboost / auctioneer namespace types.
pub mod timeboost;
/// Transaction response payload types.
pub mod transaction;

/// `reth-rpc-traits` compatibility impls (gated behind `reth-compat` feature).
#[cfg(feature = "reth-compat")]
pub mod reth_compat;

pub use arbdebug::{PricingModelHistory, TimeoutQueue, TimeoutQueueHistory};
pub use arbtrace::TraceFilter;
pub use receipt::ArbTransactionReceipt;
pub use request::ArbTransactionRequest;
pub use timeboost::JsonExpressLaneSubmission;
pub use transaction::ArbTransaction;

use alloc::string::String;
use alloy_primitives::Bytes;
use serde::{Deserialize, Serialize};

/// Returned by `arb_maintenanceStatus`.
/// Nitro reference: `nitro/execution/interface.go` -> `MaintenanceStatus`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArbMaintenanceStatus {
    /// Whether the publisher process is currently running.
    pub is_running: bool,
}

/// Returned by `arb_getMinRequiredNitroVersion`.
/// Nitro reference: `nitro/arbnode/nitro-version-alerter/server.go` -> `MinRequiredNitroVersionResult`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArbMinRequiredNitroVersion {
    /// Minimum Nitro node semantic version required by the chain.
    pub node_version: String,
    /// Release date associated with `node_version`.
    pub node_version_date: String,
    /// Deadline by which the minimum version is required.
    pub upgrade_deadline: String,
}

/// Returned by `arb_getRawBlockMetadata`.
/// Nitro reference: `nitro/execution/gethexec/api.go` -> `NumberAndBlockMetadata`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArbRawBlockMetadata {
    /// L2 block number associated with the metadata blob.
    #[serde(with = "alloy_serde::quantity")]
    pub block_number: u64,
    /// Raw binary metadata payload returned by Nitro.
    pub raw_metadata: Bytes,
}
