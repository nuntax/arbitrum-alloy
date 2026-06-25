//! reth `NodePrimitives` integration for Arbitrum types.
//!
//! Gated behind the `reth` feature. Binds [`ArbPrimitives`] to reth's
//! [`NodePrimitives`](reth_primitives_traits::NodePrimitives) trait so the
//! arb-reth-evm executor can use Arbitrum block/tx/receipt types.

use alloy_consensus::InMemorySize;
use alloy_primitives::Log;

use crate::{ArbReceiptEnvelope, ArbTxEnvelope};

/// Arbitrum block type (alloy-consensus Block with Arbitrum txs; Header via extra_data/mix_hash).
pub type ArbBlock = alloy_consensus::Block<ArbTxEnvelope>;

/// Arbitrum block body type.
pub type ArbBlockBody = alloy_consensus::BlockBody<ArbTxEnvelope>;

/// Arbitrum node primitives — binds Arbitrum tx/receipt/block types to reth's trait system.
///
/// Mirrors `EthPrimitives` from `reth-ethereum-primitives`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArbPrimitives;

impl reth_primitives_traits::NodePrimitives for ArbPrimitives {
    type Block = ArbBlock;
    type BlockHeader = alloy_consensus::Header;
    type BlockBody = ArbBlockBody;
    type SignedTx = ArbTxEnvelope;
    type Receipt = ArbReceiptEnvelope<Log>;
}

// ---------------------------------------------------------------------------
// InMemorySize for ArbTxEnvelope
// ---------------------------------------------------------------------------

impl InMemorySize for ArbTxEnvelope {
    fn size(&self) -> usize {
        use alloy_eips::Encodable2718;
        // For Ethereum variants, Signed<T>::size() is already impl'd in alloy-consensus.
        match self {
            Self::Legacy(tx) => tx.size(),
            Self::Eip2930(tx) => tx.size(),
            Self::Eip1559(tx) => tx.size(),
            Self::Eip7702(tx) => tx.size(),
            // Arbitrum-specific sealed variants: 2718 encoded length as heuristic.
            Self::Deposit(tx) => tx.encode_2718_len(),
            Self::SubmitRetryable(tx) => tx.encode_2718_len(),
            Self::Unsigned(tx) => tx.encode_2718_len(),
            Self::Contract(tx) => tx.encode_2718_len(),
            Self::Retry(tx) => tx.encode_2718_len(),
            Self::Internal(tx) => tx.encode_2718_len(),
        }
    }
}

// ---------------------------------------------------------------------------
// InMemorySize for ArbReceiptEnvelope<Log>
//
// Only impl for T: alloy_rlp::Encodable + ... so that Encodable2718 is satisfied
// (the existing impl on ArbReceiptEnvelope<T> requires T: Encodable + Send + Sync).
// ---------------------------------------------------------------------------

impl InMemorySize for ArbReceiptEnvelope<Log> {
    fn size(&self) -> usize {
        use alloy_eips::Encodable2718;
        self.encode_2718_len()
    }
}

// ---------------------------------------------------------------------------
// RlpEncodableReceipt + RlpDecodableReceipt on ArbReceiptEnvelope<Log>
// (envelope-level, required by reth Receipt supertrait)
// ---------------------------------------------------------------------------

impl alloy_consensus::RlpEncodableReceipt for ArbReceiptEnvelope<Log> {
    fn rlp_encoded_length_with_bloom(&self, bloom: &alloy_primitives::Bloom) -> usize {
        self.as_receipt_with_bloom().receipt.rlp_encoded_length_with_bloom(bloom)
    }

    fn rlp_encode_with_bloom(
        &self,
        bloom: &alloy_primitives::Bloom,
        out: &mut dyn alloy_rlp::BufMut,
    ) {
        self.as_receipt_with_bloom().receipt.rlp_encode_with_bloom(bloom, out)
    }
}

impl alloy_consensus::RlpDecodableReceipt for ArbReceiptEnvelope<Log> {
    fn rlp_decode_with_bloom(
        buf: &mut &[u8],
    ) -> alloy_rlp::Result<alloy_consensus::ReceiptWithBloom<Self>> {
        use alloy_consensus::RlpDecodableReceipt;
        // Decode the inner ArbReceipt<Log> and wrap in the Legacy variant.
        // The type is recovered from context (e.g. block body iteration); we use Legacy
        // as the default fallback, matching how alloy decodes bare RLP receipts.
        let rwb =
            <crate::ArbReceipt<Log> as RlpDecodableReceipt>::rlp_decode_with_bloom(buf)?;
        Ok(alloy_consensus::ReceiptWithBloom {
            receipt: ArbReceiptEnvelope::Legacy(rwb),
            logs_bloom: Default::default(),
        })
    }
}

// ---------------------------------------------------------------------------
// Eip2718EncodableReceipt on ArbReceiptEnvelope<Log>
// ---------------------------------------------------------------------------

impl alloy_consensus::Eip2718EncodableReceipt for ArbReceiptEnvelope<Log> {
    fn eip2718_encoded_length_with_bloom(&self, bloom: &alloy_primitives::Bloom) -> usize {
        use alloy_consensus::RlpEncodableReceipt;
        use alloy_eips::Typed2718;
        let rlp_len = self.rlp_encoded_length_with_bloom(bloom);
        if self.is_legacy() {
            rlp_len
        } else {
            1 + rlp_len
        }
    }

    fn eip2718_encode_with_bloom(
        &self,
        bloom: &alloy_primitives::Bloom,
        out: &mut dyn alloy_rlp::BufMut,
    ) {
        use alloy_consensus::RlpEncodableReceipt;
        use alloy_eips::Typed2718;
        if !self.is_legacy() {
            out.put_u8(self.ty());
        }
        self.rlp_encode_with_bloom(bloom, out);
    }
}

// ---------------------------------------------------------------------------
// Static assertion test
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn _assert_node_primitives<T: reth_primitives_traits::NodePrimitives>() {}
    fn _assert_signed_tx<T: reth_primitives_traits::SignedTransaction>() {}

    #[test]
    fn arb_primitives_satisfies_reth() {
        _assert_node_primitives::<ArbPrimitives>();
        _assert_signed_tx::<ArbTxEnvelope>();
    }
}
