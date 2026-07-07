use alloy_consensus::TxReceipt;
use alloy_network_primitives::ReceiptResponse;
use alloy_primitives::{Address, B256, BlockHash, TxHash};
use serde::{Deserialize, Serialize};

use alloy_rpc_types_eth::Log as RpcLog;
use arbitrum_alloy_consensus::ArbReceiptEnvelope;

/// Arbitrum transaction receipt response with Nitro extensions.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArbTransactionReceipt {
    /// Base Ethereum receipt payload with typed Arbitrum receipt envelope.
    #[serde(flatten)]
    pub inner: alloy_rpc_types_eth::TransactionReceipt<ArbReceiptEnvelope<RpcLog>>,

    /// Gas charged for L1 calldata posting
    #[serde(default, with = "alloy_serde::quantity")]
    pub gas_used_for_l1: u64,

    /// L1 block number at the time of this transaction
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "alloy_serde::quantity::opt"
    )]
    pub l1_block_number: Option<u64>,

    /// Whether the transaction was timeboosted in its block (Nitro block metadata).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeboosted: Option<bool>,
}

impl ReceiptResponse for ArbTransactionReceipt {
    fn contract_address(&self) -> Option<Address> {
        self.inner.contract_address
    }

    fn status(&self) -> bool {
        self.inner.inner.status()
    }

    fn block_hash(&self) -> Option<BlockHash> {
        self.inner.block_hash
    }

    fn block_number(&self) -> Option<u64> {
        self.inner.block_number
    }

    fn transaction_hash(&self) -> TxHash {
        self.inner.transaction_hash
    }

    fn transaction_index(&self) -> Option<u64> {
        self.inner.transaction_index
    }

    fn gas_used(&self) -> u64 {
        self.inner.gas_used
    }

    fn effective_gas_price(&self) -> u128 {
        self.inner.effective_gas_price
    }

    fn blob_gas_used(&self) -> Option<u64> {
        self.inner.blob_gas_used
    }

    fn blob_gas_price(&self) -> Option<u128> {
        self.inner.blob_gas_price
    }

    fn from(&self) -> Address {
        self.inner.from
    }

    fn to(&self) -> Option<Address> {
        self.inner.to
    }

    fn cumulative_gas_used(&self) -> u64 {
        self.inner.inner.cumulative_gas_used()
    }

    fn state_root(&self) -> Option<B256> {
        None
    }
}
