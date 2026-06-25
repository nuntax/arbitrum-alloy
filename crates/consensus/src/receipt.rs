use alloc::vec::Vec;
use alloy_consensus::{
    Eip658Value, Receipt, ReceiptWithBloom, RlpDecodableReceipt, RlpEncodableReceipt, TxReceipt,
};
use alloy_eips::{
    Typed2718,
    eip2718::{
        Decodable2718, EIP1559_TX_TYPE_ID, EIP2930_TX_TYPE_ID, EIP4844_TX_TYPE_ID,
        EIP7702_TX_TYPE_ID, Eip2718Error, Eip2718Result, Encodable2718, LEGACY_TX_TYPE_ID,
    },
};
use alloy_primitives::{Bloom, Log};
use alloy_rlp::{BufMut, Decodable, Encodable};
use core::fmt;

/// Arbitrum receipt body.
///
/// The consensus encoding matches Ethereum receipts, but Nitro tracks extra
/// fields such as `gas_used_for_l1` at the implementation/RPC layer.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArbReceipt<T = Log> {
    /// Standard Ethereum receipt payload.
    #[serde(flatten)]
    pub inner: Receipt<T>,
    /// Additional L1 gas accounted by Nitro for this transaction.
    #[serde(default, with = "alloy_serde::quantity")]
    pub gas_used_for_l1: u64,
}

impl<T> ArbReceipt<T> {
    /// Constructs a new Arbitrum receipt wrapper from an Ethereum receipt.
    pub const fn new(inner: Receipt<T>) -> Self {
        Self {
            inner,
            gas_used_for_l1: 0,
        }
    }

    /// Returns the inner Ethereum receipt.
    pub const fn as_receipt(&self) -> &Receipt<T> {
        &self.inner
    }

    /// Maps logs while preserving Arbitrum-specific receipt metadata.
    pub fn map_logs<U>(self, f: impl FnMut(T) -> U) -> ArbReceipt<U> {
        ArbReceipt {
            inner: self.inner.map_logs(f),
            gas_used_for_l1: self.gas_used_for_l1,
        }
    }
}

impl<T: Encodable> RlpEncodableReceipt for ArbReceipt<T> {
    fn rlp_encoded_length_with_bloom(&self, bloom: &Bloom) -> usize {
        self.inner
            .rlp_header_with_bloom(bloom)
            .length_with_payload()
    }

    fn rlp_encode_with_bloom(&self, bloom: &Bloom, out: &mut dyn BufMut) {
        self.inner.rlp_header_with_bloom(bloom).encode(out);
        self.inner.rlp_encode_fields_with_bloom(bloom, out);
    }
}

impl<T: Decodable> RlpDecodableReceipt for ArbReceipt<T> {
    fn rlp_decode_with_bloom(buf: &mut &[u8]) -> alloy_rlp::Result<ReceiptWithBloom<Self>> {
        let ReceiptWithBloom {
            receipt,
            logs_bloom,
        } = <Receipt<T> as RlpDecodableReceipt>::rlp_decode_with_bloom(buf)?;
        Ok(ReceiptWithBloom {
            receipt: Self::new(receipt),
            logs_bloom,
        })
    }
}

/// Arbitrum receipt envelope.
///
/// This keeps a strongly-typed enum of supported receipt type tags instead of
/// using a permissive "any-network" envelope.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ArbReceiptEnvelope<T = Log> {
    /// Legacy receipt (`type = 0x0`).
    #[serde(rename = "0x0", alias = "0x00")]
    Legacy(ReceiptWithBloom<ArbReceipt<T>>),
    /// EIP-2930 receipt (`type = 0x1`).
    #[serde(rename = "0x1", alias = "0x01")]
    Eip2930(ReceiptWithBloom<ArbReceipt<T>>),
    /// EIP-1559 receipt (`type = 0x2`).
    #[serde(rename = "0x2", alias = "0x02")]
    Eip1559(ReceiptWithBloom<ArbReceipt<T>>),
    /// EIP-4844 receipt (`type = 0x3`).
    #[serde(rename = "0x3", alias = "0x03")]
    Eip4844(ReceiptWithBloom<ArbReceipt<T>>),
    /// EIP-7702 receipt (`type = 0x4`).
    #[serde(rename = "0x4", alias = "0x04")]
    Eip7702(ReceiptWithBloom<ArbReceipt<T>>),
    /// Arbitrum deposit receipt (`type = 0x64`).
    #[serde(rename = "0x64")]
    Deposit(ReceiptWithBloom<ArbReceipt<T>>),
    /// Arbitrum unsigned L1-originated receipt (`type = 0x65`).
    #[serde(rename = "0x65")]
    Unsigned(ReceiptWithBloom<ArbReceipt<T>>),
    /// Arbitrum contract receipt (`type = 0x66`).
    #[serde(rename = "0x66")]
    Contract(ReceiptWithBloom<ArbReceipt<T>>),
    /// Arbitrum retry receipt (`type = 0x68`).
    #[serde(rename = "0x68")]
    Retry(ReceiptWithBloom<ArbReceipt<T>>),
    /// Arbitrum submit-retryable receipt (`type = 0x69`).
    #[serde(rename = "0x69")]
    SubmitRetryable(ReceiptWithBloom<ArbReceipt<T>>),
    /// Arbitrum internal system receipt (`type = 0x6a`).
    #[serde(rename = "0x6a", alias = "0x6A")]
    Internal(ReceiptWithBloom<ArbReceipt<T>>),
}

impl<T> ArbReceiptEnvelope<T> {
    pub(crate) const fn as_receipt_with_bloom(&self) -> &ReceiptWithBloom<ArbReceipt<T>> {
        match self {
            Self::Legacy(r)
            | Self::Eip2930(r)
            | Self::Eip1559(r)
            | Self::Eip4844(r)
            | Self::Eip7702(r)
            | Self::Deposit(r)
            | Self::Unsigned(r)
            | Self::Contract(r)
            | Self::Retry(r)
            | Self::SubmitRetryable(r)
            | Self::Internal(r) => r,
        }
    }
}

impl<T> TxReceipt for ArbReceiptEnvelope<T>
where
    T: Clone + fmt::Debug + PartialEq + Eq + Send + Sync,
{
    type Log = T;

    fn status_or_post_state(&self) -> Eip658Value {
        self.as_receipt_with_bloom().receipt.inner.status
    }

    fn status(&self) -> bool {
        self.as_receipt_with_bloom()
            .receipt
            .inner
            .status
            .coerce_status()
    }

    fn bloom(&self) -> Bloom {
        self.as_receipt_with_bloom().logs_bloom
    }

    fn bloom_cheap(&self) -> Option<Bloom> {
        Some(self.bloom())
    }

    fn cumulative_gas_used(&self) -> u64 {
        self.as_receipt_with_bloom()
            .receipt
            .inner
            .cumulative_gas_used
    }

    fn logs(&self) -> &[T] {
        &self.as_receipt_with_bloom().receipt.inner.logs
    }

    fn into_logs(self) -> Vec<Self::Log>
    where
        Self::Log: Clone,
    {
        match self {
            Self::Legacy(r)
            | Self::Eip2930(r)
            | Self::Eip1559(r)
            | Self::Eip4844(r)
            | Self::Eip7702(r)
            | Self::Deposit(r)
            | Self::Unsigned(r)
            | Self::Contract(r)
            | Self::Retry(r)
            | Self::SubmitRetryable(r)
            | Self::Internal(r) => r.receipt.inner.logs,
        }
    }
}

impl<T> Encodable for ArbReceiptEnvelope<T>
where
    T: Encodable + Send + Sync,
{
    fn encode(&self, out: &mut dyn alloy_rlp::BufMut) {
        self.network_encode(out);
    }

    fn length(&self) -> usize {
        self.network_len()
    }
}

impl<T> Decodable for ArbReceiptEnvelope<T>
where
    T: Decodable + Send + Sync,
{
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Self::network_decode(buf).map_err(|_| alloy_rlp::Error::Custom("Unexpected type"))
    }
}

impl<T> Typed2718 for ArbReceiptEnvelope<T> {
    fn ty(&self) -> u8 {
        match self {
            Self::Legacy(_) => LEGACY_TX_TYPE_ID,
            Self::Eip2930(_) => EIP2930_TX_TYPE_ID,
            Self::Eip1559(_) => EIP1559_TX_TYPE_ID,
            Self::Eip4844(_) => EIP4844_TX_TYPE_ID,
            Self::Eip7702(_) => EIP7702_TX_TYPE_ID,
            Self::Deposit(_) => 0x64,
            Self::Unsigned(_) => 0x65,
            Self::Contract(_) => 0x66,
            Self::Retry(_) => 0x68,
            Self::SubmitRetryable(_) => 0x69,
            Self::Internal(_) => 0x6a,
        }
    }
}

impl<T> Encodable2718 for ArbReceiptEnvelope<T>
where
    T: Encodable + Send + Sync,
{
    fn encode_2718_len(&self) -> usize {
        self.as_receipt_with_bloom().length() + !self.is_legacy() as usize
    }

    fn encode_2718(&self, out: &mut dyn BufMut) {
        if let Some(ty) = self.type_flag() {
            out.put_u8(ty);
        }
        self.as_receipt_with_bloom().encode(out);
    }
}

impl<T> Decodable2718 for ArbReceiptEnvelope<T>
where
    T: Decodable + Send + Sync,
{
    fn typed_decode(ty: u8, buf: &mut &[u8]) -> Eip2718Result<Self> {
        let receipt = Decodable::decode(buf)?;
        match ty {
            0x00 => Ok(Self::Legacy(receipt)),
            0x01 => Ok(Self::Eip2930(receipt)),
            0x02 => Ok(Self::Eip1559(receipt)),
            0x03 => Ok(Self::Eip4844(receipt)),
            0x04 => Ok(Self::Eip7702(receipt)),
            0x64 => Ok(Self::Deposit(receipt)),
            0x65 => Ok(Self::Unsigned(receipt)),
            0x66 => Ok(Self::Contract(receipt)),
            0x68 => Ok(Self::Retry(receipt)),
            0x69 => Ok(Self::SubmitRetryable(receipt)),
            0x6a => Ok(Self::Internal(receipt)),
            _ => Err(Eip2718Error::UnexpectedType(ty)),
        }
    }

    fn fallback_decode(buf: &mut &[u8]) -> Eip2718Result<Self> {
        Ok(Self::Legacy(Decodable::decode(buf)?))
    }
}
