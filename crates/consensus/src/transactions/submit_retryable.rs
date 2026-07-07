use alloc::vec;
use alloc::vec::Vec;

use alloy_consensus::{Transaction, Typed2718};
use alloy_eips::{
    Decodable2718, Encodable2718,
    eip2718::{Eip2718Error, Eip2718Result},
    eip2930::AccessList,
    eip7702::SignedAuthorization,
};
use alloy_primitives::{
    Address, B256, Bytes, ChainId, FixedBytes, Sealable, TxHash, TxKind, U256, address, keccak256,
};
use alloy_rlp::{BufMut, Decodable, Encodable, Header};
use bytes::Buf;
use serde::{Deserialize, Serialize};

use crate::transactions::{
    ArbTxType,
    util::{decode, decode_rest},
};
/// <https://github.com/OffchainLabs/nitro/blob/23cae22e1f76cf3675f965d78e268fd2870d8708/arbos/parse_l2.go#L292>
#[derive(PartialEq, Debug, Clone, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitRetryableTx {
    #[serde(alias = "chain_id")]
    chain_id: U256,
    #[serde(alias = "request_id")]
    request_id: B256,
    from: Address,
    #[serde(alias = "l1Basefee")]
    l1_base_fee: U256,

    deposit_value: U256,
    #[serde(alias = "maxFeePerGas")]
    gas_fee_cap: U256,
    #[serde(alias = "gas")]
    gas_limit: U256,
    retry_to: TxKind,
    retry_value: U256,
    beneficiary: Address,
    max_submission_fee: U256,
    #[serde(alias = "refundTo", alias = "feeRefundAddr")]
    fee_refund_address: Address,
    #[serde(default)]
    retry_data_size: U256,
    retry_data: Bytes,
    /// Pre-built ABI-encoded calldata for `Transaction::input()`. Serialized as
    /// `input` (matching the RPC representation) so the tx survives a
    /// serialize/deserialize round-trip; deserialize also accepts `data`.
    #[serde(default, rename = "input", alias = "data")]
    calldata: Bytes,
}

impl SubmitRetryableTx {
    /// ArbOS precompile address that receives submit-retryable calls.
    pub const ARB_RETRYABLE_TX_ADDRESS: Address =
        address!("0x000000000000000000000000000000000000006e");

    /// Returns the L1 sender that created the retryable ticket.
    pub const fn from(&self) -> Address {
        self.from
    }

    /// Constructs a new submit-retryable transaction body.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chain_id: U256,
        request_id: B256,
        from: Address,
        l1_base_fee: U256,
        deposit_value: U256,
        gas_fee_cap: U256,
        gas_limit: U256,
        retry_to: TxKind,
        retry_value: U256,
        beneficiary: Address,
        max_submission_fee: U256,
        fee_refund_address: Address,
        retry_data: Bytes,
    ) -> Self {
        let mut this = Self {
            chain_id,
            request_id,
            from,
            l1_base_fee,
            deposit_value,
            gas_fee_cap,
            gas_limit,
            retry_to,
            retry_value,
            beneficiary,
            max_submission_fee,
            fee_refund_address,
            retry_data_size: U256::from(retry_data.len()),
            retry_data,
            calldata: Bytes::new(),
        };
        this.calldata = this.build_calldata();
        this
    }

    /// Computes the EIP-2718 transaction hash.
    pub fn tx_hash(&self) -> TxHash {
        let mut buf = Vec::with_capacity(self.encode_2718_len());
        self.encode_2718(&mut buf);
        keccak256(&buf)
    }

    fn build_calldata(&self) -> Bytes {
        let mut retry_to = Address::ZERO;
        if let TxKind::Call(addr) = self.retry_to {
            retry_to = addr;
        }

        let mut data = Vec::new();
        data.extend_from_slice(self.request_id.as_slice());
        data.extend_from_slice(&self.l1_base_fee.to_be_bytes::<32>());
        data.extend_from_slice(&self.deposit_value.to_be_bytes::<32>());
        data.extend_from_slice(&self.retry_value.to_be_bytes::<32>());
        data.extend_from_slice(&self.gas_fee_cap.to_be_bytes::<32>());
        data.extend_from_slice(&U256::from(self.gas_limit).to_be_bytes::<32>());
        data.extend_from_slice(&self.max_submission_fee.to_be_bytes::<32>());
        data.extend_from_slice(&[0u8; 12]);
        data.extend_from_slice(self.fee_refund_address.as_slice());
        data.extend_from_slice(&[0u8; 12]);
        data.extend_from_slice(self.beneficiary.as_slice());
        data.extend_from_slice(&[0u8; 12]);
        data.extend_from_slice(retry_to.as_slice());
        let offset = data.len() + 32;
        data.extend_from_slice(&U256::from(offset).to_be_bytes::<32>());
        data.extend_from_slice(&U256::from(self.retry_data.len()).to_be_bytes::<32>());
        data.extend_from_slice(self.retry_data.as_ref());
        let extra = self.retry_data.len() % 32;
        if extra > 0 {
            data.extend_from_slice(&vec![0u8; 32 - extra]);
        }

        let mut with_selector = Vec::with_capacity(4 + data.len());
        with_selector.extend_from_slice(&[0xc9, 0xf9, 0x5d, 0x32]);
        with_selector.extend_from_slice(&data);
        with_selector.into()
    }

    fn rlp_decode_fields(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let chain_id: U256 = Decodable::decode(buf)?;
        let request_id: FixedBytes<32> = Decodable::decode(buf)?;
        let from: Address = Decodable::decode(buf)?;
        let l1_base_fee: U256 = Decodable::decode(buf)?;
        let deposit_value: U256 = Decodable::decode(buf)?;
        let gas_fee_cap: U256 = Decodable::decode(buf)?;
        let gas_limit: U256 = Decodable::decode(buf)?;
        let retry_to: TxKind = Decodable::decode(buf)?;
        let retry_value: U256 = Decodable::decode(buf)?;
        let beneficiary: Address = Decodable::decode(buf)?;
        let max_submission_fee: U256 = Decodable::decode(buf)?;
        let fee_refund_address: Address = Decodable::decode(buf)?;
        let retry_data: Bytes = Decodable::decode(buf)?;
        let mut this = Self {
            chain_id,
            request_id: B256::from(request_id.0),
            from,
            l1_base_fee,
            deposit_value,
            gas_fee_cap,
            gas_limit,
            retry_to,
            retry_value,
            beneficiary,
            max_submission_fee,
            fee_refund_address,
            retry_data_size: U256::from(retry_data.len()),
            retry_data,
            calldata: Bytes::new(),
        };
        this.calldata = this.build_calldata();
        Ok(this)
    }
    fn rlp_encode_fields(&self, out: &mut dyn BufMut) {
        Encodable::encode(&self.chain_id, out);
        Encodable::encode(&self.request_id, out);
        Encodable::encode(&self.from, out);
        Encodable::encode(&self.l1_base_fee, out);
        Encodable::encode(&self.deposit_value, out);
        Encodable::encode(&self.gas_fee_cap, out);
        Encodable::encode(&self.gas_limit, out);
        Encodable::encode(&self.retry_to, out);
        Encodable::encode(&self.retry_value, out);
        Encodable::encode(&self.beneficiary, out);
        Encodable::encode(&self.max_submission_fee, out);
        Encodable::encode(&self.fee_refund_address, out);
        Encodable::encode(&self.retry_data, out);
    }

    /// Returns the RLP list header for the inner payload.
    pub fn rlp_header(&self) -> Header {
        Header {
            list: true,
            payload_length: self.rlp_encoded_fields_length(),
        }
    }

    /// Returns the length of the encoding produced by encode_for_hash.
    pub fn rlp_encoded_fields_length(&self) -> usize {
        self.chain_id.length()
            + self.request_id.length()
            + self.from.length()
            + self.l1_base_fee.length()
            + self.deposit_value.length()
            + self.gas_fee_cap.length()
            + self.gas_limit.length()
            + self.retry_to.length()
            + self.retry_value.length()
            + self.beneficiary.length()
            + self.max_submission_fee.length()
            + self.fee_refund_address.length()
            + self.retry_data.length()
    }
    /// Decodes a retryable transaction in the format used by the sequencer.
    pub fn decode_fields_sequencer(
        buf: &mut &[u8],
        chain_id: U256,
        request_id: B256,
        sender: Address,
        l1_base_fee: U256,
    ) -> alloy_rlp::Result<Self> {
        buf.advance(12);
        let retry_to_decoded: Address = decode(buf)?;
        let retry_to = if retry_to_decoded == Address::default() {
            TxKind::Create
        } else {
            TxKind::Call(retry_to_decoded)
        };
        let mut this = Self {
            retry_to,
            retry_value: decode(buf)?,
            deposit_value: decode(buf)?,
            max_submission_fee: decode(buf).inspect(|_| {
                buf.advance(12);
            })?,
            fee_refund_address: decode(buf).inspect(|_| {
                buf.advance(12);
            })?,
            beneficiary: decode(buf)?,
            gas_limit: decode(buf)?,
            gas_fee_cap: decode(buf)?,
            retry_data_size: decode(buf)?,
            retry_data: decode_rest(buf),
            chain_id,
            request_id,
            from: sender,
            l1_base_fee,
            calldata: Bytes::new(),
        };
        this.calldata = this.build_calldata();
        Ok(this)
    }
    /// Decodes the transaction from its RLP list form (without type byte).
    pub fn rlp_decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let header = Header::decode(buf)?;
        if !header.list {
            return Err(alloy_rlp::Error::UnexpectedString);
        }
        let remaining = buf.len();

        if header.payload_length > remaining {
            return Err(alloy_rlp::Error::InputTooShort);
        }

        let this = Self::rlp_decode_fields(buf)?;

        if buf.len() + header.payload_length != remaining {
            return Err(alloy_rlp::Error::UnexpectedLength);
        }

        Ok(this)
    }

    fn rlp_encoded_length(&self) -> usize {
        self.rlp_header().length_with_payload()
    }
    fn rlp_encode(&self, out: &mut dyn BufMut) {
        let header = self.rlp_header();
        Header::encode(&header, out);

        self.rlp_encode_fields(out);
    }
}

impl Decodable for SubmitRetryableTx {
    fn decode(data: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Self::rlp_decode(data)
    }
}

impl Decodable2718 for SubmitRetryableTx {
    fn typed_decode(ty: u8, buf: &mut &[u8]) -> Eip2718Result<Self> {
        if ty != ArbTxType::SubmitRetryable as u8 {
            return Err(Eip2718Error::UnexpectedType(ty));
        }
        let tx = Self::rlp_decode(buf)?;
        Ok(tx)
    }

    fn fallback_decode(buf: &mut &[u8]) -> Eip2718Result<Self> {
        Ok(Self::decode(buf)?)
    }
}

impl Transaction for SubmitRetryableTx {
    fn chain_id(&self) -> Option<ChainId> {
        Some(self.chain_id.to())
    }

    fn nonce(&self) -> u64 {
        0
    }

    fn gas_limit(&self) -> u64 {
        self.gas_limit.to()
    }

    fn gas_price(&self) -> Option<u128> {
        Some(self.gas_fee_cap.to())
    }

    /// This returns the gas fee cap, same as gas_price. Retryable transactions dont have 1559 style fees.
    fn max_fee_per_gas(&self) -> u128 {
        self.gas_fee_cap.to()
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        None
    }

    fn max_fee_per_blob_gas(&self) -> Option<u128> {
        None
    }

    fn priority_fee_or_price(&self) -> u128 {
        self.gas_fee_cap.to()
    }

    #[allow(unused_variables)]
    fn effective_gas_price(&self, base_fee: Option<u64>) -> u128 {
        base_fee
            .map(|v| v as u128)
            .unwrap_or_else(|| self.gas_fee_cap.to())
    }

    fn is_dynamic_fee(&self) -> bool {
        false
    }

    fn kind(&self) -> TxKind {
        TxKind::Call(Self::ARB_RETRYABLE_TX_ADDRESS)
    }

    fn is_create(&self) -> bool {
        false
    }

    fn value(&self) -> U256 {
        U256::ZERO
    }

    fn input(&self) -> &Bytes {
        &self.calldata
    }

    fn access_list(&self) -> Option<&AccessList> {
        None
    }

    fn blob_versioned_hashes(&self) -> Option<&[B256]> {
        None
    }

    fn authorization_list(&self) -> Option<&[SignedAuthorization]> {
        None
    }
}

impl Typed2718 for SubmitRetryableTx {
    fn ty(&self) -> u8 {
        ArbTxType::SubmitRetryable as u8
    }
}

impl Encodable2718 for SubmitRetryableTx {
    fn encode_2718_len(&self) -> usize {
        self.rlp_encoded_length() + 1
    }

    fn encode_2718(&self, out: &mut dyn BufMut) {
        out.put_u8(self.ty());
        self.rlp_encode(out);
    }
}

impl Sealable for SubmitRetryableTx {
    fn hash_slow(&self) -> B256 {
        self.tx_hash()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_network_primitives::ReceiptResponse;
    use alloy_primitives::address;
    use alloy_primitives::hex;
    use alloy_primitives::hex::FromHex;
    use alloy_provider::Provider;
    use serial_test::serial;
    use test_utils::{RetryableParams, TestContext, dev_address};
    #[test]
    fn test_decode_submit_retryable() {
        let encoded = hex::decode(
            "000000000000000000000000abc50aee89c1b38d4ddc4ac0aee43647215ff7fc000000000000000000000000000000000000000000000000002382664887b00000000000000000000000000000000000000000000000000000239debfd13ec00000000000000000000000000000000000000000000000000000001bdcb71f400000000000000000000000000abc50aee89c1b38d4ddc4ac0aee43647215ff7fc000000000000000000000000abc50aee89c1b38d4ddc4ac0aee43647215ff7fc00000000000000000000000000000000000000000000000000000000000493e00000000000000000000000000000000000000000000000000000000005a1c5c00000000000000000000000000000000000000000000000000000000000000000",
        ).unwrap();
        let mut buf = &encoded[..];
        let from = address!("0x8789dfc2406ac2d60f174813e8a79f2b69862566");
        let l1_base_fee = U256::from(335396856);
        let request_id = B256::from(U256::from(0x20eb40).to_be_bytes::<32>());

        let tx: SubmitRetryableTx = SubmitRetryableTx::decode_fields_sequencer(
            &mut buf,
            U256::from(42161),
            request_id,
            from,
            l1_base_fee,
        )
        .unwrap();
        let hash = tx.tx_hash();
        assert_eq!(
            hash,
            TxHash::from_hex("0x19f98fc86cae7ac924a2ad789e86fca824aff065ec7366daedeb1d8e60ae96f5")
                .unwrap()
        )
    }

    #[tokio::test]
    #[serial]
    async fn submit_retryable_produces_submit_retryable_tx_on_l2()
    -> Result<(), Box<dyn std::error::Error>> {
        let Some(ctx) = TestContext::try_from_env().await else {
            eprintln!("ARBITRUM_RPC/ETHEREUM_RPC not set, skipping");
            return Ok(());
        };

        let since = ctx.arbitrum_provider.get_block_number().await?;
        println!("[retryable] scanning L2 from block {since}");

        let max_submission_cost = U256::from(100_000_000_000_000u64);
        let gas_limit = U256::from(100_000u64);
        let max_fee_per_gas = U256::from(1_000_000_000u64);
        let value = max_submission_cost + gas_limit * max_fee_per_gas;

        let dev_addr = dev_address();
        println!("[retryable] dev addr (L2 target): {dev_addr}");

        println!("[retryable] submitting createRetryableTicket on L1 (value={value} wei)...");
        let l1_receipt = ctx
            .submit_retryable(RetryableParams {
                to: dev_addr,
                l2_call_value: U256::ZERO,
                max_submission_cost,
                excess_fee_refund_address: dev_addr,
                call_value_refund_address: dev_addr,
                gas_limit,
                max_fee_per_gas,
                data: Default::default(),
                value,
            })
            .await?;
        assert!(l1_receipt.status(), "L1 createRetryableTicket tx reverted");
        println!(
            "[retryable] L1 tx confirmed: {} (block {})",
            l1_receipt.transaction_hash(),
            l1_receipt.block_number().unwrap_or_default(),
        );

        println!("[retryable] advancing L1 by 2 blocks...");
        ctx.advance_l1_blocks(2).await?;

        println!("[retryable] waiting for SubmitRetryable (0x69) on L2...");
        let l2_hash = ctx
            .wait_for_l2_tx_type(0x69, since, std::time::Duration::from_secs(60))
            .await?;
        println!("[retryable] found L2 SubmitRetryable tx: {l2_hash}");

        let receipt = ctx
            .arbitrum_provider
            .get_transaction_receipt(l2_hash)
            .await?;
        assert!(
            receipt.is_some(),
            "missing L2 receipt for SubmitRetryable tx {l2_hash}"
        );
        println!(
            "[retryable] L2 receipt: block {}",
            receipt.unwrap().block_number().unwrap_or_default(),
        );

        Ok(())
    }
}
