use alloc::vec::Vec;

use alloy_consensus::Transaction;
use alloy_consensus::Typed2718;
use alloy_eips::Decodable2718;
use alloy_eips::Encodable2718;
use alloy_eips::eip2718::Eip2718Error;
use alloy_eips::eip2718::Eip2718Result;
use alloy_eips::eip2930::AccessList;
use alloy_eips::eip7702::SignedAuthorization;
use alloy_primitives::B256;
use alloy_primitives::Bytes;
use alloy_primitives::ChainId;
use alloy_primitives::FixedBytes;
use alloy_primitives::Selector;
use alloy_primitives::TxHash;
use alloy_primitives::TxKind;
use alloy_primitives::{Address, Sealable, U256, keccak256};
use alloy_rlp::Decodable;
use alloy_rlp::Encodable;
use alloy_rlp::Header;
use bytes::BufMut;
use serde::{Deserialize, Serialize};

use crate::transactions::ArbTxType;

/// Arbitrum L1 ETH deposit transaction (`type = 0x64`).
#[derive(PartialEq, Debug, Clone, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxDeposit {
    /// Arbitrum chain identifier.
    #[serde(alias = "chain_id")]
    pub chain_id: U256,
    /// L1 request identifier for this deposit.
    #[serde(alias = "request_id")]
    pub request_id: FixedBytes<32>,
    /// Depositor address.
    pub from: Address,
    /// Recipient address on L2.
    pub to: Address,
    /// Deposited ETH value.
    pub value: U256,
}

impl TxDeposit {
    /// Returns the sender/depositor address.
    pub const fn from(&self) -> Address {
        self.from
    }

    /// Computes the EIP-2718 transaction hash.
    pub fn tx_hash(&self) -> TxHash {
        let mut buf = Vec::with_capacity(self.encode_2718_len());
        self.encode_2718(&mut buf);
        keccak256(&buf)
    }
    /// Decodes sequencer feed fields for an ETH deposit payload.
    pub fn decode_fields_sequencer(
        buf: &mut &[u8],
        chain_id: U256,
        request_id: FixedBytes<32>,
        from: Address,
    ) -> Result<Self, alloy_rlp::Error> {
        // Nitro sequencer EthDeposit payload layout is fixed-width bytes:
        //   to (20 bytes) || value (32 bytes)
        // It is not RLP-encoded.
        if buf.len() < 52 {
            return Err(alloy_rlp::Error::InputTooShort);
        }

        let to = Address::from_slice(&buf[..20]);
        let value = U256::from_be_slice(&buf[20..52]);
        *buf = &buf[52..];

        // Keep decoding strict to catch malformed payloads early.
        if !buf.is_empty() {
            return Err(alloy_rlp::Error::UnexpectedLength);
        }

        Ok(Self {
            chain_id,
            request_id,
            from,
            to,
            value,
        })
    }
    /// Encodes the inner RLP fields (without list header or type byte).
    pub fn rlp_encode_fields(&self, out: &mut dyn BufMut) {
        self.chain_id.encode(out);
        self.request_id.encode(out);
        self.from.encode(out);
        self.to.encode(out);
        self.value.encode(out);
    }
    /// Returns the encoded RLP payload length for the inner fields.
    pub fn rlp_encoded_fields_length(&self) -> usize {
        self.chain_id.length()
            + self.request_id.length()
            + self.from.length()
            + self.to.length()
            + self.value.length()
    }
    /// Returns the RLP list header for the inner payload.
    pub fn rlp_header(&self) -> Header {
        Header {
            list: true,
            payload_length: self.rlp_encoded_fields_length(),
        }
    }
    /// Encodes the transaction in RLP list form (without type byte).
    pub fn rlp_encode(&self, out: &mut dyn BufMut) {
        self.rlp_header().encode(out);
        self.rlp_encode_fields(out);
    }
    /// Decodes the transaction from its RLP list form (without type byte).
    pub fn rlp_decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let header = Header::decode(buf)?;
        if !header.list {
            return Err(alloy_rlp::Error::Custom("Expected list header"));
        }
        Self::rlp_decode_fields(buf)
    }
    fn rlp_encoded_length(&self) -> usize {
        self.rlp_header().length_with_payload()
    }

    /// Decodes inner RLP fields for a deposit transaction.
    pub fn rlp_decode_fields(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let chain_id: U256 = Decodable::decode(buf)?;
        let request_id: FixedBytes<32> = Decodable::decode(buf)?;
        let from: Address = Decodable::decode(buf)?;
        let to: Address = Decodable::decode(buf)?;
        let value: U256 = Decodable::decode(buf)?;
        Ok(Self {
            chain_id,
            request_id,
            from,
            to,
            value,
        })
    }
}
impl Typed2718 for TxDeposit {
    #[doc = " Returns the EIP-2718 type flag."]
    fn ty(&self) -> u8 {
        ArbTxType::Deposit as u8
    }
}
impl Decodable for TxDeposit {
    fn decode(data: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Self::rlp_decode(data)
    }
}
impl Decodable2718 for TxDeposit {
    fn typed_decode(ty: u8, buf: &mut &[u8]) -> Eip2718Result<Self> {
        if ty != ArbTxType::Deposit as u8 {
            return Err(Eip2718Error::UnexpectedType(ty));
        }
        let tx = Self::rlp_decode(buf)?;
        Ok(tx)
    }

    fn fallback_decode(buf: &mut &[u8]) -> Eip2718Result<Self> {
        Ok(Self::decode(buf)?)
    }
}

impl Encodable2718 for TxDeposit {
    #[doc = " The length of the 2718 encoded envelope. This is the length of the type"]
    #[doc = " flag + the length of the inner encoding."]
    fn encode_2718_len(&self) -> usize {
        self.rlp_encoded_length() + 1
    }

    #[doc = " Encode the transaction according to [EIP-2718] rules. First a 1-byte"]
    #[doc = " type flag in the range 0x0-0x7f, then the body of the transaction."]
    #[doc = ""]
    #[doc = " [EIP-2718] inner encodings are unspecified, and produce an opaque"]
    #[doc = " bytestring."]
    #[doc = ""]
    #[doc = " [EIP-2718]: https://eips.ethereum.org/EIPS/eip-2718"]
    fn encode_2718(&self, out: &mut dyn BufMut) {
        out.put_u8(self.ty());
        self.rlp_encode(out);
    }
}

impl Transaction for TxDeposit {
    #[doc = " Get `chain_id`."]
    fn chain_id(&self) -> Option<ChainId> {
        Some(self.chain_id.to())
    }

    #[doc = " Get `nonce`."]
    fn nonce(&self) -> u64 {
        0
    }

    #[doc = " Get `gas_limit`."]
    fn gas_limit(&self) -> u64 {
        0 // Deposits do not have a gas limit in the same way as user transactions
    }

    #[doc = " Get `gas_price`."]
    fn gas_price(&self) -> Option<u128> {
        None // Deposits do not have a gas price in the same way as user transactions
    }

    #[doc = " For dynamic fee transactions returns the maximum fee per gas the caller is willing to pay."]
    #[doc = ""]
    #[doc = " For legacy fee transactions this is `gas_price`."]
    #[doc = ""]
    #[doc = " This is also commonly referred to as the \"Gas Fee Cap\"."]
    fn max_fee_per_gas(&self) -> u128 {
        0 // Deposits do not have a max fee per gas in the same way as user transactions
    }

    #[doc = " For dynamic fee transactions returns the Priority fee the caller is paying to the block"]
    #[doc = " author."]
    #[doc = ""]
    #[doc = " This will return `None` for legacy fee transactions"]
    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        None // Deposits do not have a max priority fee per gas in the same way as user transactions
    }

    #[doc = " Max fee per blob gas for EIP-4844 transaction."]
    #[doc = ""]
    #[doc = " Returns `None` for non-eip4844 transactions."]
    #[doc = ""]
    #[doc = " This is also commonly referred to as the \"Blob Gas Fee Cap\"."]
    fn max_fee_per_blob_gas(&self) -> Option<u128> {
        None // Deposits do not have a max fee per blob gas in the same way as user transactions
    }

    #[doc = " Return the max priority fee per gas if the transaction is a dynamic fee transaction, and"]
    #[doc = " otherwise return the gas price."]
    #[doc = ""]
    #[doc = " # Warning"]
    #[doc = ""]
    #[doc = " This is different than the `max_priority_fee_per_gas` method, which returns `None` for"]
    #[doc = " legacy fee transactions."]
    fn priority_fee_or_price(&self) -> u128 {
        0 // Deposits do not have a priority fee or price in the same way as user transactions
    }

    #[doc = " Returns the effective gas price for the given base fee."]
    #[doc = ""]
    #[doc = " If the transaction is a legacy fee transaction, the gas price is returned."]
    fn effective_gas_price(&self, _base_fee: Option<u64>) -> u128 {
        0
    }

    #[doc = " Returns `true` if the transaction supports dynamic fees."]
    fn is_dynamic_fee(&self) -> bool {
        false
    }

    #[doc = " Returns the transaction kind."]
    fn kind(&self) -> TxKind {
        TxKind::Call(self.to)
    }

    #[doc = " Returns true if the transaction is a contract creation."]
    #[doc = " We don\'t provide a default implementation via `kind` as it copies the 21-byte"]
    #[doc = " [`TxKind`] for this simple check. A proper implementation shouldn\'t allocate."]
    fn is_create(&self) -> bool {
        false // Deposits do not create a contract
    }

    #[doc = " Get `value`."]
    fn value(&self) -> U256 {
        self.value
    }

    #[doc = " Get `data`."]
    fn input(&self) -> &Bytes {
        static EMPTY_BYTES: Bytes = Bytes::new();
        &EMPTY_BYTES
    }

    #[doc = " Returns the EIP-2930 `access_list` for the particular transaction type. Returns `None` for"]
    #[doc = " older transaction types."]
    fn access_list(&self) -> Option<&AccessList> {
        None // Deposits do not have an access list in the same way as user transactions
    }

    #[doc = " Blob versioned hashes for eip4844 transaction. For previous transaction types this is"]
    #[doc = " `None`."]
    fn blob_versioned_hashes(&self) -> Option<&[B256]> {
        None // Deposits do not have blob versioned hashes in the same way as user transactions
    }

    #[doc = " Returns the [`SignedAuthorization`] list of the transaction."]
    #[doc = ""]
    #[doc = " Returns `None` if this transaction is not EIP-7702."]
    fn authorization_list(&self) -> Option<&[SignedAuthorization]> {
        None // Deposits do not have an authorization list in the same way as user transactions
    }

    #[doc = " Returns the effective tip for this transaction."]
    #[doc = ""]
    #[doc = " For dynamic fee transactions: `min(max_fee_per_gas - base_fee, max_priority_fee_per_gas)`."]
    #[doc = " For legacy fee transactions: `gas_price - base_fee`."]
    fn effective_tip_per_gas(&self, _base_fee: u64) -> Option<u128> {
        None // Deposits do not have an effective tip in the same way as user transactions
    }

    #[doc = " Get the transaction\'s address of the contract that will be called, or the address that will"]
    #[doc = " receive the transfer."]
    #[doc = ""]
    #[doc = " Returns `None` if this is a `CREATE` transaction."]
    fn to(&self) -> Option<Address> {
        Some(self.to)
    }

    #[doc = " Returns the first 4bytes of the calldata for a function call."]
    #[doc = ""]
    #[doc = " The selector specifies the function to be called."]
    fn function_selector(&self) -> Option<&Selector> {
        None
    }

    #[doc = " Returns the number of blobs of this transaction."]
    #[doc = ""]
    #[doc = " This is convenience function for `len(blob_versioned_hashes)`."]
    #[doc = ""]
    #[doc = " Returns `None` for non-eip4844 transactions."]
    fn blob_count(&self) -> Option<u64> {
        None
    }

    #[doc = " Returns the total gas for all blobs in this transaction."]
    #[doc = ""]
    #[doc = " Returns `None` for non-eip4844 transactions."]
    #[inline]
    fn blob_gas_used(&self) -> Option<u64> {
        None
    }

    #[doc = " Returns the number of blobs of [`SignedAuthorization`] in this transactions"]
    #[doc = ""]
    #[doc = " This is convenience function for `len(authorization_list)`."]
    #[doc = ""]
    #[doc = " Returns `None` for non-eip7702 transactions."]
    fn authorization_count(&self) -> Option<u64> {
        None
    }
}

impl Sealable for TxDeposit {
    fn hash_slow(&self) -> B256 {
        self.tx_hash()
    }
}

#[cfg(test)]
mod tests {
    use super::TxDeposit;
    use alloy_network_primitives::ReceiptResponse;
    use alloy_primitives::{Address, FixedBytes, U256};
    use alloy_provider::Provider;
    use serial_test::serial;
    use test_utils::{DepositParams, TestContext};

    const ONE_MILLI_ETH: U256 = U256::from_limbs([1_000_000_000_000_000u64, 0, 0, 0]);

    #[test]
    fn sequencer_eth_deposit_decodes_fixed_width_payload() {
        let to = Address::from_slice(&[
            0x3f, 0x1e, 0xae, 0x7d, 0x46, 0xd8, 0x8f, 0x08, 0xfc, 0x2f, 0x8e, 0xd2, 0x7f, 0xcb,
            0x2a, 0xb1, 0x83, 0xeb, 0x2d, 0x0e,
        ]);
        let value = U256::from(123_000_000_000_000_000u128);

        let mut payload = Vec::with_capacity(52);
        payload.extend_from_slice(to.as_slice());
        payload.extend_from_slice(&value.to_be_bytes::<32>());

        let chain_id = U256::from(42161u64);
        let request_id = FixedBytes::repeat_byte(0x11);
        let from = Address::from_slice(&[
            0x50, 0x2f, 0xae, 0x7d, 0x46, 0xd8, 0x8f, 0x08, 0xfc, 0x2f, 0x8e, 0xd2, 0x7f, 0xcb,
            0x2a, 0xb1, 0x83, 0xeb, 0x3e, 0x1f,
        ]);

        let tx = TxDeposit::decode_fields_sequencer(&mut &payload[..], chain_id, request_id, from)
            .expect("sequencer payload should decode");

        assert_eq!(tx.to, to);
        assert_eq!(tx.value, value);
        assert_eq!(tx.from, from);
        assert_eq!(tx.chain_id, chain_id);
        assert_eq!(tx.request_id, request_id);
    }

    #[tokio::test]
    #[serial]
    async fn deposit_eth_produces_deposit_tx_on_l2() -> Result<(), Box<dyn std::error::Error>> {
        let Some(ctx) = TestContext::try_from_env().await else {
            eprintln!("ARBITRUM_RPC/ETHEREUM_RPC not set, skipping");
            return Ok(());
        };

        let since = ctx.arbitrum_provider.get_block_number().await?;
        println!("[deposit] scanning L2 from block {since}");

        println!("[deposit] submitting depositEth({ONE_MILLI_ETH} wei) on L1...");
        let l1_receipt = ctx
            .deposit_eth(DepositParams {
                value: ONE_MILLI_ETH,
            })
            .await?;
        assert!(l1_receipt.status(), "L1 depositEth tx reverted");
        println!(
            "[deposit] L1 tx confirmed: {} (block {})",
            l1_receipt.transaction_hash(),
            l1_receipt.block_number().unwrap_or_default(),
        );

        println!("[deposit] advancing L1 by 2 blocks to satisfy sequencer finalize-distance...");
        ctx.advance_l1_blocks(2).await?;

        println!("[deposit] waiting for TxDeposit (0x64) on L2...");
        let l2_hash = ctx
            .wait_for_l2_tx_type(0x64, since, std::time::Duration::from_secs(60))
            .await?;
        println!("[deposit] found L2 deposit tx: {l2_hash}");

        let receipt = ctx
            .arbitrum_provider
            .get_transaction_receipt(l2_hash)
            .await?;
        assert!(
            receipt.is_some(),
            "missing L2 receipt for deposit tx {l2_hash}"
        );
        let receipt = receipt.unwrap();
        assert!(receipt.status(), "L2 deposit tx did not succeed");
        println!(
            "[deposit] L2 receipt: block {}, status={}",
            receipt.block_number().unwrap_or_default(),
            receipt.status(),
        );

        Ok(())
    }
}
