//! `reth-rpc-traits` implementations for Arbitrum RPC types.
//!
//! These impls allow the reth [`RpcConverter`] blanket to fire for `Arbitrum` as the network
//! type, enabling the stock `EthApi<_, ArbRpcConverter<Provider>>` to compile and serve
//! `eth_*` methods for Arbitrum nodes.
//!
//! Mirroring op-alloy's `rpc-types/src/reth_compat.rs`.

use alloy_consensus::{SignableTransaction, error::ValueError};
use alloy_primitives::Address;
use alloy_rpc_types_eth::TransactionInfo;
use arb_alloy_consensus::{ArbTxEnvelope, ArbTypedTransaction};
use core::convert::Infallible;
use reth_rpc_traits::{FromConsensusTx, SignTxRequestError, SignableTxRequest, TryIntoSimTx};

use crate::{ArbTransaction, ArbTransactionRequest};

// ---------------------------------------------------------------------------
// FromConsensusTx<ArbTxEnvelope> for ArbTransaction
// ---------------------------------------------------------------------------

impl FromConsensusTx<ArbTxEnvelope> for ArbTransaction {
    type TxInfo = TransactionInfo;
    type Err = Infallible;

    fn from_consensus_tx(
        tx: ArbTxEnvelope,
        signer: Address,
        tx_info: Self::TxInfo,
    ) -> Result<Self, Self::Err> {
        use alloy_consensus::transaction::Recovered;
        let recovered = Recovered::new_unchecked(tx, signer);
        let inner = alloy_rpc_types_eth::Transaction::from_transaction(recovered, tx_info);
        Ok(Self { inner, request_id: None })
    }
}

// ---------------------------------------------------------------------------
// TryIntoSimTx<ArbTxEnvelope> for ArbTransactionRequest
// ---------------------------------------------------------------------------

impl TryIntoSimTx<ArbTxEnvelope> for ArbTransactionRequest {
    fn try_into_sim_tx(self) -> Result<ArbTxEnvelope, ValueError<Self>> {
        use alloy_primitives::Signature;

        let inner_clone = self.inner.clone();
        let typed = inner_clone.build_typed_tx().map_err(|_inner| {
            ValueError::new(self, "Required fields missing for sim tx")
        })?;

        // Create an empty signature — this tx is only used for simulation, never broadcast.
        let signature = Signature::new(Default::default(), Default::default(), false);

        let arb_typed: ArbTypedTransaction = match typed {
            alloy_consensus::TypedTransaction::Legacy(t) => ArbTypedTransaction::Legacy(t),
            alloy_consensus::TypedTransaction::Eip2930(t) => ArbTypedTransaction::Eip2930(t),
            alloy_consensus::TypedTransaction::Eip1559(t) => ArbTypedTransaction::Eip1559(t),
            alloy_consensus::TypedTransaction::Eip7702(t) => ArbTypedTransaction::Eip7702(t),
            alloy_consensus::TypedTransaction::Eip4844(_) => {
                return Err(ValueError::new(
                    ArbTransactionRequest::default(),
                    "EIP-4844 not supported on Arbitrum",
                ));
            }
        };

        Ok(arb_typed.into_signed(signature).into())
    }
}

// ---------------------------------------------------------------------------
// SignableTxRequest<ArbTxEnvelope> for ArbTransactionRequest
// ---------------------------------------------------------------------------

impl SignableTxRequest<ArbTxEnvelope> for ArbTransactionRequest {
    async fn try_build_and_sign(
        self,
        signer: impl alloy_network::TxSigner<alloy_primitives::Signature> + Send,
    ) -> Result<ArbTxEnvelope, SignTxRequestError> {
        use alloy_consensus::SignableTransaction;

        let mut tx = self
            .inner
            .build_typed_tx()
            .map_err(|_| SignTxRequestError::InvalidTransactionRequest)?;

        let signature = signer.sign_transaction(&mut tx).await?;

        let arb_typed: ArbTypedTransaction = match tx {
            alloy_consensus::TypedTransaction::Legacy(t) => ArbTypedTransaction::Legacy(t),
            alloy_consensus::TypedTransaction::Eip2930(t) => ArbTypedTransaction::Eip2930(t),
            alloy_consensus::TypedTransaction::Eip1559(t) => ArbTypedTransaction::Eip1559(t),
            alloy_consensus::TypedTransaction::Eip7702(t) => ArbTypedTransaction::Eip7702(t),
            alloy_consensus::TypedTransaction::Eip4844(_) => {
                return Err(SignTxRequestError::InvalidTransactionRequest);
            }
        };

        Ok(arb_typed.into_signed(signature).into())
    }
}

// ---------------------------------------------------------------------------
// TryIntoTxEnv for ArbTransactionRequest
//
// Delegates to the inner alloy_rpc_types_eth::TransactionRequest, which already
// implements TryIntoTxEnv<revm::context::TxEnv, ...>. ArbEvmConfig's TxEnvFor
// is revm::context::TxEnv (via ArbEvmFactory / ArbEvm).
// ---------------------------------------------------------------------------

#[cfg(all(feature = "alloy-evm", feature = "revm"))]
mod tx_env_impl {
    use super::ArbTransactionRequest;
    use alloy_evm::rpc::{EthTxEnvError, TryIntoTxEnv};

    impl<Spec, BlockEnv: alloy_evm::env::BlockEnvironment>
        TryIntoTxEnv<::revm::context::TxEnv, Spec, BlockEnv> for ArbTransactionRequest
    {
        type Err = EthTxEnvError;

        fn try_into_tx_env(
            self,
            evm_env: &alloy_evm::EvmEnv<Spec, BlockEnv>,
        ) -> Result<::revm::context::TxEnv, EthTxEnvError> {
            self.inner.try_into_tx_env(evm_env)
        }
    }
}

