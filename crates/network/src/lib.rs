#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/alloy-rs/core/main/assets/alloy.jpg",
    html_favicon_url = "https://raw.githubusercontent.com/alloy-rs/core/main/assets/favicon.ico"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{vec, vec::Vec};
use alloy_consensus::{Header as EthHeader, TxType, TypedTransaction};
use alloy_network::{
    BuildResult, Network, NetworkTransactionBuilder, NetworkWallet, TransactionBuilderError,
};
use alloy_provider::fillers::{
    ChainIdFiller, GasFiller, JoinFill, NonceFiller, RecommendedFillers,
};
use alloy_rpc_types_eth::Block;

use arb_alloy_consensus::{ArbReceiptEnvelope, ArbTxEnvelope, ArbTxType, ArbTypedTransaction};
use arb_alloy_rpc_types::{ArbTransaction, ArbTransactionReceipt, ArbTransactionRequest};

/// Alloy `Network` implementation for Arbitrum.
#[derive(Clone, Copy, Debug)]
pub struct Arbitrum {
    _private: (),
}

impl Arbitrum {
    /// Creates a new Arbitrum network marker type.
    pub const fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for Arbitrum {
    fn default() -> Self {
        Self::new()
    }
}

impl Network for Arbitrum {
    type TxType = ArbTxType;
    type TxEnvelope = ArbTxEnvelope;
    type UnsignedTx = ArbTypedTransaction;
    type ReceiptEnvelope = ArbReceiptEnvelope;
    type Header = EthHeader;

    type TransactionRequest = ArbTransactionRequest;
    type TransactionResponse = ArbTransaction;
    type ReceiptResponse = ArbTransactionReceipt;
    type HeaderResponse = alloy_rpc_types_eth::Header;
    type BlockResponse = Block<Self::TransactionResponse, Self::HeaderResponse>;
}

const fn arb_tx_type_from_eth(ty: TxType) -> Option<ArbTxType> {
    match ty {
        TxType::Legacy => Some(ArbTxType::Legacy),
        TxType::Eip2930 => Some(ArbTxType::Eip2930),
        TxType::Eip1559 => Some(ArbTxType::Eip1559),
        TxType::Eip7702 => Some(ArbTxType::Eip7702),
        TxType::Eip4844 => None,
    }
}

impl NetworkTransactionBuilder<Arbitrum> for ArbTransactionRequest {
    fn complete_type(&self, ty: ArbTxType) -> Result<(), Vec<&'static str>> {
        match ty {
            ArbTxType::Legacy => self.inner.complete_legacy(),
            ArbTxType::Eip2930 => self.inner.complete_2930(),
            ArbTxType::Eip1559 => self.inner.complete_1559(),
            ArbTxType::Eip7702 => self.inner.complete_7702(),
            _ => Err(vec!["unsupported_tx_type"]),
        }
    }

    fn can_submit(&self) -> bool {
        self.inner.from.is_some()
    }

    fn can_build(&self) -> bool {
        let common = self.inner.gas.is_some() && self.inner.nonce.is_some();

        let legacy = self.inner.gas_price.is_some();
        let eip2930 = legacy && self.inner.access_list.is_some();

        let eip1559 =
            self.inner.max_fee_per_gas.is_some() && self.inner.max_priority_fee_per_gas.is_some();

        let eip7702 = eip1559 && self.inner.authorization_list.is_some();

        common && (legacy || eip2930 || eip1559 || eip7702)
    }

    fn output_tx_type(&self) -> ArbTxType {
        match self.inner.preferred_type() {
            TxType::Eip4844 => ArbTxType::Eip1559,
            other => arb_tx_type_from_eth(other).unwrap_or(ArbTxType::Eip1559),
        }
    }

    fn output_tx_type_checked(&self) -> Option<ArbTxType> {
        match self.inner.buildable_type() {
            Some(TxType::Eip4844) => None,
            Some(other) => arb_tx_type_from_eth(other),
            None => None,
        }
    }

    fn prep_for_submission(&mut self) {
        self.inner.transaction_type = Some(self.output_tx_type() as u8);
        self.inner.trim_conflicting_keys();
        self.inner.populate_blob_hashes();
    }

    fn build_unsigned(self) -> BuildResult<ArbTypedTransaction, Arbitrum> {
        let pref = self.inner.preferred_type();
        if pref == TxType::Eip4844 {
            return Err(TransactionBuilderError::InvalidTransactionRequest(
                ArbTxType::Eip1559,
                vec!["eip4844_unsupported"],
            )
            .into_unbuilt(self));
        }

        if let Err((tx_type, missing)) = self.inner.missing_keys() {
            let arb_ty = arb_tx_type_from_eth(tx_type).unwrap_or(ArbTxType::Eip1559);
            return Err(
                TransactionBuilderError::InvalidTransactionRequest(arb_ty, missing)
                    .into_unbuilt(self),
            );
        }

        let typed = self
            .inner
            .build_typed_tx()
            .expect("checked by missing_keys");
        let mapped = match typed {
            TypedTransaction::Legacy(tx) => ArbTypedTransaction::Legacy(tx),
            TypedTransaction::Eip2930(tx) => ArbTypedTransaction::Eip2930(tx),
            TypedTransaction::Eip1559(tx) => ArbTypedTransaction::Eip1559(tx),
            TypedTransaction::Eip7702(tx) => ArbTypedTransaction::Eip7702(tx),
            TypedTransaction::Eip4844(_) => unreachable!("eip4844 is unsupported on Arbitrum"),
        };

        Ok(mapped)
    }

    async fn build<W: NetworkWallet<Arbitrum>>(
        self,
        wallet: &W,
    ) -> Result<<Arbitrum as Network>::TxEnvelope, TransactionBuilderError<Arbitrum>> {
        Ok(wallet.sign_request(self).await?)
    }
}

impl RecommendedFillers for Arbitrum {
    type RecommendedFillers = JoinFill<GasFiller, JoinFill<NonceFiller, ChainIdFiller>>;

    fn recommended_fillers() -> Self::RecommendedFillers {
        Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_consensus::{SignableTransaction, TxEip1559, TxEip2930, TxEip7702, TxLegacy};
    use alloy_network::{EthereumWallet, TxSigner};
    use alloy_primitives::{Address, Bytes, Signature, U256};
    use arb_alloy_consensus::transactions::internal::ArbInternalTx;
    use std::future::Future;
    use std::task::{Context, Poll, Waker};

    #[derive(Clone, Debug)]
    struct TestSigner {
        address: Address,
    }

    #[async_trait::async_trait]
    impl TxSigner<Signature> for TestSigner {
        fn address(&self) -> Address {
            self.address
        }

        async fn sign_transaction(
            &self,
            _tx: &mut dyn SignableTransaction<Signature>,
        ) -> alloy_signer::Result<Signature> {
            Ok(Signature::new(U256::from(1_u64), U256::from(1_u64), false))
        }
    }

    fn make_wallet() -> (EthereumWallet, Address) {
        let signer = TestSigner {
            address: Address::repeat_byte(0x11),
        };
        let address = signer.address;
        (EthereumWallet::new(signer), address)
    }

    fn block_on<T>(future: impl Future<Output = T>) -> T {
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        let mut future = std::pin::pin!(future);
        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(output) => return output,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    #[test]
    fn network_wallet_bridge_forwards_signer_metadata() {
        let (wallet, address) = make_wallet();

        assert_eq!(
            NetworkWallet::<Arbitrum>::default_signer_address(&wallet),
            address
        );
        assert!(NetworkWallet::<Arbitrum>::has_signer_for(&wallet, &address));

        let addresses: Vec<_> = NetworkWallet::<Arbitrum>::signer_addresses(&wallet).collect();
        assert_eq!(addresses, vec![address]);
    }

    #[test]
    fn network_wallet_bridge_signs_supported_transaction_types() {
        let (wallet, sender) = make_wallet();

        let legacy = block_on(NetworkWallet::<Arbitrum>::sign_transaction_from(
            &wallet,
            sender,
            ArbTypedTransaction::Legacy(TxLegacy::default()),
        ))
        .expect("legacy signing should succeed");
        assert!(matches!(legacy, ArbTxEnvelope::Legacy(_)));

        let eip2930 = block_on(NetworkWallet::<Arbitrum>::sign_transaction_from(
            &wallet,
            sender,
            ArbTypedTransaction::Eip2930(TxEip2930::default()),
        ))
        .expect("eip2930 signing should succeed");
        assert!(matches!(eip2930, ArbTxEnvelope::Eip2930(_)));

        let eip1559 = block_on(NetworkWallet::<Arbitrum>::sign_transaction_from(
            &wallet,
            sender,
            ArbTypedTransaction::Eip1559(TxEip1559::default()),
        ))
        .expect("eip1559 signing should succeed");
        assert!(matches!(eip1559, ArbTxEnvelope::Eip1559(_)));

        let eip7702 = block_on(NetworkWallet::<Arbitrum>::sign_transaction_from(
            &wallet,
            sender,
            ArbTypedTransaction::Eip7702(TxEip7702::default()),
        ))
        .expect("eip7702 signing should succeed");
        assert!(matches!(eip7702, ArbTxEnvelope::Eip7702(_)));
    }

    #[test]
    #[should_panic(
        expected = "Arbitrum-specific transactions cannot be converted from a signed envelope"
    )]
    fn network_wallet_bridge_rejects_custom_arbitrum_transaction_types() {
        let (wallet, sender) = make_wallet();
        // Arb-specific transaction types are L1-originated and cannot be user-signed.
        // SignableTransaction::encode_for_signing panics for these variants.
        let _ = block_on(NetworkWallet::<Arbitrum>::sign_transaction_from(
            &wallet,
            sender,
            ArbTypedTransaction::Internal(ArbInternalTx::new(42161, Bytes::new())),
        ));
    }

    #[test]
    fn recommended_fillers_are_available_for_arbitrum() {
        let _fillers = <Arbitrum as RecommendedFillers>::recommended_fillers();
        let _builder = alloy_provider::ProviderBuilder::new_with_network::<Arbitrum>();
    }
}
