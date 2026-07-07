use alloy_consensus::{Transaction as TransactionTrait, transaction::Recovered};
use alloy_eips::{Typed2718, eip2930::AccessList, eip7702::SignedAuthorization};
use alloy_network_primitives::TransactionResponse;
use alloy_primitives::{Address, B256, BlockHash, Bytes, ChainId, TxKind, U256};
use serde::{Deserialize, Serialize};

use arbitrum_alloy_consensus::ArbTxEnvelope;

/// Arbitrum RPC transaction response.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    try_from = "tx_serde::TransactionSerdeHelper",
    into = "tx_serde::TransactionSerdeHelper"
)]
pub struct ArbTransaction {
    /// Base Ethereum transaction payload carrying an Arbitrum envelope.
    pub inner: alloy_rpc_types_eth::Transaction<ArbTxEnvelope>,

    /// Optional Arbitrum request identifier for L1-originated transactions.
    pub request_id: Option<B256>,
}

impl AsRef<ArbTxEnvelope> for ArbTransaction {
    fn as_ref(&self) -> &ArbTxEnvelope {
        self.inner.as_ref()
    }
}

impl TransactionTrait for ArbTransaction {
    fn chain_id(&self) -> Option<ChainId> {
        self.inner.chain_id()
    }

    fn nonce(&self) -> u64 {
        self.inner.nonce()
    }

    fn gas_limit(&self) -> u64 {
        self.inner.gas_limit()
    }

    fn gas_price(&self) -> Option<u128> {
        TransactionTrait::gas_price(&self.inner)
    }

    fn max_fee_per_gas(&self) -> u128 {
        TransactionTrait::max_fee_per_gas(&self.inner)
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        self.inner.max_priority_fee_per_gas()
    }

    fn max_fee_per_blob_gas(&self) -> Option<u128> {
        self.inner.max_fee_per_blob_gas()
    }

    fn priority_fee_or_price(&self) -> u128 {
        self.inner.priority_fee_or_price()
    }

    fn effective_gas_price(&self, base_fee: Option<u64>) -> u128 {
        self.inner.effective_gas_price(base_fee)
    }

    fn is_dynamic_fee(&self) -> bool {
        self.inner.is_dynamic_fee()
    }

    fn kind(&self) -> TxKind {
        self.inner.kind()
    }

    fn is_create(&self) -> bool {
        self.inner.is_create()
    }

    fn value(&self) -> U256 {
        self.inner.value()
    }

    fn input(&self) -> &Bytes {
        self.inner.input()
    }

    fn access_list(&self) -> Option<&AccessList> {
        self.inner.access_list()
    }

    fn blob_versioned_hashes(&self) -> Option<&[B256]> {
        self.inner.blob_versioned_hashes()
    }

    fn authorization_list(&self) -> Option<&[SignedAuthorization]> {
        self.inner.authorization_list()
    }
}

impl TransactionResponse for ArbTransaction {
    fn tx_hash(&self) -> B256 {
        self.inner.tx_hash()
    }

    fn block_hash(&self) -> Option<BlockHash> {
        self.inner.block_hash
    }

    fn block_number(&self) -> Option<u64> {
        self.inner.block_number
    }

    fn transaction_index(&self) -> Option<u64> {
        self.inner.transaction_index
    }

    fn from(&self) -> Address {
        self.inner.from()
    }
}

impl Typed2718 for ArbTransaction {
    fn ty(&self) -> u8 {
        self.inner.ty()
    }
}

mod tx_serde {
    use super::*;
    use serde::de::Error;

    #[derive(Serialize, Deserialize)]
    struct OptionalFields {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        from: Option<Address>,
        #[serde(
            default,
            rename = "gasPrice",
            skip_serializing_if = "Option::is_none",
            with = "alloy_serde::quantity::opt"
        )]
        effective_gas_price: Option<u128>,
        #[serde(
            default,
            rename = "arbRequestId",
            skip_serializing_if = "Option::is_none"
        )]
        request_id: Option<B256>,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct TransactionSerdeHelper {
        #[serde(flatten)]
        inner: ArbTxEnvelope,
        #[serde(default)]
        block_hash: Option<BlockHash>,
        #[serde(default, with = "alloy_serde::quantity::opt")]
        block_number: Option<u64>,
        #[serde(default, with = "alloy_serde::quantity::opt")]
        transaction_index: Option<u64>,
        #[serde(default, with = "alloy_serde::quantity::opt")]
        block_timestamp: Option<u64>,
        #[serde(flatten)]
        other: OptionalFields,
    }

    const fn inner_contains_from(inner: &ArbTxEnvelope) -> bool {
        matches!(
            inner,
            ArbTxEnvelope::Deposit(_)
                | ArbTxEnvelope::SubmitRetryable(_)
                | ArbTxEnvelope::Unsigned(_)
                | ArbTxEnvelope::Contract(_)
                | ArbTxEnvelope::Retry(_)
        )
    }

    impl From<ArbTransaction> for TransactionSerdeHelper {
        fn from(value: ArbTransaction) -> Self {
            let ArbTransaction {
                inner:
                    alloy_rpc_types_eth::Transaction {
                        inner,
                        block_hash,
                        block_number,
                        transaction_index,
                        effective_gas_price,
                        block_timestamp,
                    },
                request_id,
            } = value;

            let (inner, from) = inner.into_parts();
            let from = if inner_contains_from(&inner) {
                None
            } else {
                Some(from)
            };
            let effective_gas_price = effective_gas_price.filter(|_| inner.gas_price().is_none());

            Self {
                inner,
                block_hash,
                block_number,
                transaction_index,
                block_timestamp,
                other: OptionalFields {
                    from,
                    effective_gas_price,
                    request_id,
                },
            }
        }
    }

    impl TryFrom<TransactionSerdeHelper> for ArbTransaction {
        type Error = serde_json::Error;

        fn try_from(value: TransactionSerdeHelper) -> Result<Self, Self::Error> {
            let TransactionSerdeHelper {
                inner,
                block_hash,
                block_number,
                transaction_index,
                block_timestamp,
                other,
            } = value;

            let from = if let Some(from) = other.from {
                from
            } else {
                inner
                    .sender()
                    .map_err(|_| serde_json::Error::custom("missing `from` field"))?
            };

            let effective_gas_price = other.effective_gas_price.or(inner.gas_price());

            Ok(Self {
                inner: alloy_rpc_types_eth::Transaction {
                    inner: Recovered::new_unchecked(inner, from),
                    block_hash,
                    block_number,
                    transaction_index,
                    effective_gas_price,
                    block_timestamp,
                },
                request_id: other.request_id,
            })
        }
    }
}
