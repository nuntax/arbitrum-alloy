//! Canonical addresses for all Arbitrum precompile contracts.
//!
//! Nitro reference: `nitro/precompiles/precompile.go` (precompile registration).

use alloy_primitives::{Address, address};

/// ArbSys: core L2 system operations (block number, L2→L1 messaging, etc.).
pub const ARB_SYS: Address = address!("0x0000000000000000000000000000000000000064");

/// ArbInfo: account balance and code queries.
pub const ARB_INFO: Address = address!("0x0000000000000000000000000000000000000065");

/// ArbAddressTable: address compression table for calldata optimization.
pub const ARB_ADDRESS_TABLE: Address = address!("0x0000000000000000000000000000000000000066");

/// ArbBLS: legacy BLS public key registry (Classic-era; no active methods in Nitro).
/// Nitro reference: types.ArbBLSAddress = 0x67.
pub const ARB_BLS: Address = address!("0x0000000000000000000000000000000000000067");

/// ArbFunctionTable: function table for classic Arbitrum contracts.
pub const ARB_FUNCTION_TABLE: Address = address!("0x0000000000000000000000000000000000000068");

/// ArbNativeTokenManager: mint/burn native token (active from ArbOS v41).
/// Nitro reference: types.ArbNativeTokenManagerAddress = 0x73.
pub const ARB_NATIVE_TOKEN_MANAGER: Address =
    address!("0x0000000000000000000000000000000000000073");

/// ArbFilteredTransactionsManager: filtered transaction list management.
/// Nitro reference: types.ArbFilteredTransactionsManagerAddress = 0x74.
pub const ARB_FILTERED_TRANSACTIONS_MANAGER: Address =
    address!("0x0000000000000000000000000000000000000074");

/// ArbOwnerPublic: read-only chain owner queries (callable by anyone).
pub const ARB_OWNER_PUBLIC: Address = address!("0x000000000000000000000000000000000000006b");

/// ArbGasInfo: gas pricing and L1/L2 fee information.
pub const ARB_GAS_INFO: Address = address!("0x000000000000000000000000000000000000006c");

/// ArbAggregator: batch poster management.
pub const ARB_AGGREGATOR: Address = address!("0x000000000000000000000000000000000000006d");

/// ArbRetryableTx: retryable ticket management (redeem, cancel, keepalive).
pub const ARB_RETRYABLE_TX: Address = address!("0x000000000000000000000000000000000000006e");

/// ArbStatistics: chain statistics (mostly pre-Nitro legacy).
pub const ARB_STATISTICS: Address = address!("0x000000000000000000000000000000000000006f");

/// ArbOwner: chain owner administration (only callable by chain owners).
pub const ARB_OWNER: Address = address!("0x0000000000000000000000000000000000000070");

/// ArbWasm: Stylus WASM program management.
pub const ARB_WASM: Address = address!("0x0000000000000000000000000000000000000071");

/// ArbWasmCache: Stylus WASM cache management.
pub const ARB_WASM_CACHE: Address = address!("0x0000000000000000000000000000000000000072");

/// ArbDebug: debug-only precompile (not available in production).
pub const ARB_DEBUG: Address = address!("0x00000000000000000000000000000000000000ff");

/// ArbosActs: internal ArbOS actor (only callable by ArbOS itself).
pub const ARBOS_ACTS: Address = address!("0x00000000000000000000000000000000000a4b05");

/// NodeInterface: virtual meta-contract for node-level queries.
/// Not a real on-chain contract; handled by the node software.
pub const NODE_INTERFACE: Address = address!("0x00000000000000000000000000000000000000c8");
