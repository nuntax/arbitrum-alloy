use alloy_network::TransactionBuilder;
use alloy_network_primitives::TransactionResponse as _;
use alloy_primitives::{Address, Bytes, ChainId, TxKind, U256};
use alloy_rpc_types_eth::AccessList;
use serde::{Deserialize, Serialize};

use arb_alloy_consensus::ArbTxEnvelope;

/// Arbitrum transaction request wrapper around Ethereum request fields.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArbTransactionRequest {
    /// Underlying Ethereum transaction request payload.
    #[serde(flatten)]
    pub inner: alloy_rpc_types_eth::TransactionRequest,
}

impl From<alloy_rpc_types_eth::TransactionRequest> for ArbTransactionRequest {
    fn from(inner: alloy_rpc_types_eth::TransactionRequest) -> Self {
        Self { inner }
    }
}

impl From<ArbTxEnvelope> for ArbTransactionRequest {
    fn from(tx: ArbTxEnvelope) -> Self {
        Self {
            inner: alloy_rpc_types_eth::TransactionRequest::from_transaction(tx),
        }
    }
}

impl From<arb_alloy_consensus::ArbTypedTransaction> for ArbTransactionRequest {
    fn from(tx: arb_alloy_consensus::ArbTypedTransaction) -> Self {
        let inner = match tx {
            arb_alloy_consensus::ArbTypedTransaction::Legacy(tx) => {
                alloy_rpc_types_eth::TransactionRequest::from_transaction(tx)
            }
            arb_alloy_consensus::ArbTypedTransaction::Eip2930(tx) => {
                alloy_rpc_types_eth::TransactionRequest::from_transaction(tx)
            }
            arb_alloy_consensus::ArbTypedTransaction::Eip1559(tx) => {
                alloy_rpc_types_eth::TransactionRequest::from_transaction(tx)
            }
            arb_alloy_consensus::ArbTypedTransaction::Eip7702(tx) => {
                alloy_rpc_types_eth::TransactionRequest::from_transaction(tx)
            }
            arb_alloy_consensus::ArbTypedTransaction::Deposit(tx) => {
                alloy_rpc_types_eth::TransactionRequest::from_transaction(tx)
            }
            arb_alloy_consensus::ArbTypedTransaction::SubmitRetryable(tx) => {
                alloy_rpc_types_eth::TransactionRequest::from_transaction(tx)
            }
            arb_alloy_consensus::ArbTypedTransaction::Unsigned(tx) => {
                alloy_rpc_types_eth::TransactionRequest::from_transaction(tx)
            }
            arb_alloy_consensus::ArbTypedTransaction::Contract(tx) => {
                alloy_rpc_types_eth::TransactionRequest::from_transaction(tx)
            }
            arb_alloy_consensus::ArbTypedTransaction::Retry(tx) => {
                alloy_rpc_types_eth::TransactionRequest::from_transaction(tx)
            }
            arb_alloy_consensus::ArbTypedTransaction::Internal(tx) => {
                alloy_rpc_types_eth::TransactionRequest::from_transaction(tx)
            }
        };

        Self { inner }
    }
}

impl From<crate::ArbTransaction> for ArbTransactionRequest {
    fn from(tx: crate::ArbTransaction) -> Self {
        let from = tx.from();
        let inner = alloy_rpc_types_eth::TransactionRequest::from_transaction_with_sender(tx, from);
        Self { inner }
    }
}

impl TransactionBuilder for ArbTransactionRequest {
    fn chain_id(&self) -> Option<ChainId> {
        self.inner.chain_id
    }

    fn set_chain_id(&mut self, chain_id: ChainId) {
        self.inner.chain_id = Some(chain_id);
    }

    fn nonce(&self) -> Option<u64> {
        self.inner.nonce
    }

    fn set_nonce(&mut self, nonce: u64) {
        self.inner.nonce = Some(nonce);
    }

    fn take_nonce(&mut self) -> Option<u64> {
        self.inner.nonce.take()
    }

    fn input(&self) -> Option<&Bytes> {
        self.inner.input.input()
    }

    fn set_input<T: Into<Bytes>>(&mut self, input: T) {
        self.inner.input.input = Some(input.into());
    }

    fn set_input_kind<T: Into<Bytes>>(
        &mut self,
        input: T,
        kind: alloy_rpc_types_eth::TransactionInputKind,
    ) {
        match kind {
            alloy_rpc_types_eth::TransactionInputKind::Input => {
                self.inner.input.input = Some(input.into())
            }
            alloy_rpc_types_eth::TransactionInputKind::Data => {
                self.inner.input.data = Some(input.into())
            }
            alloy_rpc_types_eth::TransactionInputKind::Both => {
                let bytes = input.into();
                self.inner.input.input = Some(bytes.clone());
                self.inner.input.data = Some(bytes);
            }
        }
    }

    fn from(&self) -> Option<Address> {
        self.inner.from
    }

    fn set_from(&mut self, from: Address) {
        self.inner.from = Some(from);
    }

    fn kind(&self) -> Option<TxKind> {
        self.inner.to
    }

    fn clear_kind(&mut self) {
        self.inner.to = None;
    }

    fn set_kind(&mut self, kind: TxKind) {
        self.inner.to = Some(kind);
    }

    fn value(&self) -> Option<U256> {
        self.inner.value
    }

    fn set_value(&mut self, value: U256) {
        self.inner.value = Some(value)
    }

    fn gas_price(&self) -> Option<u128> {
        self.inner.gas_price
    }

    fn set_gas_price(&mut self, gas_price: u128) {
        self.inner.gas_price = Some(gas_price);
    }

    fn max_fee_per_gas(&self) -> Option<u128> {
        self.inner.max_fee_per_gas
    }

    fn set_max_fee_per_gas(&mut self, max_fee_per_gas: u128) {
        self.inner.max_fee_per_gas = Some(max_fee_per_gas);
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        self.inner.max_priority_fee_per_gas
    }

    fn set_max_priority_fee_per_gas(&mut self, max_priority_fee_per_gas: u128) {
        self.inner.max_priority_fee_per_gas = Some(max_priority_fee_per_gas);
    }

    fn gas_limit(&self) -> Option<u64> {
        self.inner.gas
    }

    fn set_gas_limit(&mut self, gas_limit: u64) {
        self.inner.gas = Some(gas_limit);
    }

    fn access_list(&self) -> Option<&AccessList> {
        self.inner.access_list.as_ref()
    }

    fn set_access_list(&mut self, access_list: AccessList) {
        self.inner.access_list = Some(access_list);
    }
}
