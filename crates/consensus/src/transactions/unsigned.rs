use alloc::vec::Vec;

use alloy_consensus::{Transaction, Typed2718};
use alloy_eips::{
    Decodable2718, Encodable2718,
    eip2718::{Eip2718Error, Eip2718Result},
    eip2930::AccessList,
    eip7702::SignedAuthorization,
};
use alloy_primitives::{Address, B256, Bytes, ChainId, Sealable, TxHash, TxKind, U256, keccak256};
use alloy_rlp::{Decodable, Encodable, Header};
use bytes::BufMut;
use serde::{Deserialize, Serialize};

use crate::transactions::ArbTxType;

/// Arbitrum L1-originated unsigned user transaction (`type = 0x65`).
#[derive(PartialEq, Debug, Clone, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxUnsigned {
    /// Arbitrum chain identifier.
    #[serde(alias = "chain_id")]
    pub chain_id: U256,
    /// Sender address supplied by ArbOS.
    pub from: Address,
    /// Sender nonce.
    #[serde(with = "alloy_serde::quantity")]
    pub nonce: u64,
    /// Maximum fee per gas.
    #[serde(alias = "maxFeePerGas", alias = "gasPrice")]
    pub gas_fee_cap: U256,
    /// Gas limit for execution.
    #[serde(alias = "gas", with = "alloy_serde::quantity")]
    pub gas_limit: u64,
    /// Call target (or create).
    pub to: TxKind,
    /// ETH value transferred to the target.
    pub value: U256,
    /// Transaction calldata.
    pub input: Bytes,
}

impl TxUnsigned {
    /// Returns the sender address.
    pub const fn from(&self) -> Address {
        self.from
    }

    /// Computes the EIP-2718 transaction hash.
    pub fn tx_hash(&self) -> TxHash {
        let mut buf = Vec::with_capacity(self.encode_2718_len());
        self.encode_2718(&mut buf);
        keccak256(&buf)
    }

    /// Encodes the inner RLP fields (without list header or type byte).
    pub fn rlp_encode_fields(&self, out: &mut dyn BufMut) {
        self.chain_id.encode(out);
        self.from.encode(out);
        self.nonce.encode(out);
        self.gas_fee_cap.encode(out);
        self.gas_limit.encode(out);
        self.to.encode(out);
        self.value.encode(out);
        self.input.encode(out);
    }

    /// Returns the encoded RLP payload length for the inner fields.
    pub fn rlp_encoded_fields_length(&self) -> usize {
        self.chain_id.length()
            + self.from.length()
            + self.nonce.length()
            + self.gas_fee_cap.length()
            + self.gas_limit.length()
            + self.to.length()
            + self.value.length()
            + self.input.length()
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

    fn rlp_encoded_length(&self) -> usize {
        self.rlp_header().length_with_payload()
    }

    /// Decodes the transaction from its RLP list form (without type byte).
    pub fn rlp_decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let header = Header::decode(buf)?;
        if !header.list {
            return Err(alloy_rlp::Error::Custom("Expected list header"));
        }
        let chain_id: U256 = Decodable::decode(buf)?;
        let from: Address = Decodable::decode(buf)?;
        let nonce: u64 = Decodable::decode(buf)?;
        let gas_fee_cap: U256 = Decodable::decode(buf)?;
        let gas_limit: u64 = Decodable::decode(buf)?;
        let to: TxKind = Decodable::decode(buf)?;
        let value: U256 = Decodable::decode(buf)?;
        let input: Bytes = Decodable::decode(buf)?;
        Ok(Self {
            chain_id,
            from,
            nonce,
            gas_fee_cap,
            gas_limit,
            to,
            value,
            input,
        })
    }
}

impl Typed2718 for TxUnsigned {
    fn ty(&self) -> u8 {
        ArbTxType::Unsigned as u8
    }
}

impl Decodable for TxUnsigned {
    fn decode(data: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Self::rlp_decode(data)
    }
}

impl Decodable2718 for TxUnsigned {
    fn typed_decode(ty: u8, buf: &mut &[u8]) -> Eip2718Result<Self> {
        if ty != ArbTxType::Unsigned as u8 {
            return Err(Eip2718Error::UnexpectedType(ty));
        }
        Ok(Self::rlp_decode(buf)?)
    }

    fn fallback_decode(buf: &mut &[u8]) -> Eip2718Result<Self> {
        Ok(Self::decode(buf)?)
    }
}

impl Encodable2718 for TxUnsigned {
    fn encode_2718_len(&self) -> usize {
        self.rlp_encoded_length() + 1
    }

    fn encode_2718(&self, out: &mut dyn BufMut) {
        out.put_u8(self.ty());
        self.rlp_encode(out);
    }
}

impl Transaction for TxUnsigned {
    fn chain_id(&self) -> Option<ChainId> {
        Some(self.chain_id.to())
    }

    fn nonce(&self) -> u64 {
        self.nonce
    }

    fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    fn gas_price(&self) -> Option<u128> {
        Some(self.gas_fee_cap.to())
    }

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

    fn effective_gas_price(&self, base_fee: Option<u64>) -> u128 {
        base_fee
            .map(|v| v as u128)
            .unwrap_or_else(|| self.gas_fee_cap.to())
    }

    fn is_dynamic_fee(&self) -> bool {
        false
    }

    fn kind(&self) -> TxKind {
        self.to
    }

    fn is_create(&self) -> bool {
        self.to.is_create()
    }

    fn value(&self) -> U256 {
        self.value
    }

    fn input(&self) -> &Bytes {
        &self.input
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

impl Sealable for TxUnsigned {
    fn hash_slow(&self) -> B256 {
        self.tx_hash()
    }
}

#[cfg(test)]
mod tests {
    use alloy_eips::Typed2718;
    use alloy_network_primitives::{ReceiptResponse, TransactionResponse};
    use alloy_primitives::{Address, B256, U256, address, uint};
    use alloy_provider::Provider;
    use alloy_rpc_types_eth::TransactionTrait;
    use serial_test::serial;
    use test_utils::{TestContext, UnsignedTxParams, dev_address};

    use super::TxUnsigned;

    const ADDRESS_ALIAS_OFFSET: U256 = uint!(0x1111000000000000000000000000000000001111_U256);

    fn l1_to_l2_alias(addr: Address) -> Address {
        let mut buf = [0u8; 32];
        buf[12..].copy_from_slice(addr.as_slice());
        let aliased = U256::from_be_bytes(buf).wrapping_add(ADDRESS_ALIAS_OFFSET);
        let out = aliased.to_be_bytes::<32>();
        Address::from_slice(&out[12..])
    }

    #[test]
    fn deserialize_rpc_shape_supports_gas_price_alias() {
        let raw = r#"{
            "chainId":"0x64aba",
            "from":"0x502fae7d46d88f08fc2f8ed27fcb2ab183eb3e1f",
            "nonce":"0x8",
            "gas":"0x186a0",
            "gasPrice":"0x3b9aca00",
            "to":"0x3f1eae7d46d88f08fc2f8ed27fcb2ab183eb2d0e",
            "value":"0x5af3107a4000",
            "input":"0x"
        }"#;

        let tx: TxUnsigned = serde_json::from_str(raw).expect("valid TxUnsigned RPC shape");

        assert_eq!(tx.ty(), 0x65);
        assert_eq!(
            tx.from,
            address!("0x502fae7d46d88f08fc2f8ed27fcb2ab183eb3e1f")
        );
        assert_eq!(tx.nonce, 8);
        assert_eq!(tx.gas_limit, 100_000);
        assert_eq!(tx.gas_fee_cap, uint!(1_000_000_000_U256));
    }

    #[tokio::test]
    #[serial]
    async fn send_unsigned_tx_produces_unsigned_tx_on_l2() -> Result<(), Box<dyn std::error::Error>>
    {
        let Some(ctx) = TestContext::try_from_env().await else {
            eprintln!("ARBITRUM_RPC/ETHEREUM_RPC not set, skipping");
            return Ok(());
        };

        let since = ctx.arbitrum_provider.get_block_number().await?;
        println!("[unsigned] scanning L2 from block {since}");

        let l1_sender = dev_address();
        let l2_sender = l1_to_l2_alias(l1_sender);
        let l2_nonce = ctx
            .arbitrum_provider
            .get_transaction_count(l2_sender)
            .await?;
        println!(
            "[unsigned] l1 sender: {l1_sender}, aliased l2 sender: {l2_sender}, L2 nonce: {l2_nonce}"
        );

        println!("[unsigned] submitting sendUnsignedTransaction on L1...");
        let l1_receipt = ctx
            .send_unsigned_transaction(UnsignedTxParams {
                gas_limit: uint!(100_000_U256),
                max_fee_per_gas: uint!(1_000_000_000_U256),
                nonce: U256::from(l2_nonce),
                to: l1_sender,
                data: Default::default(),
            })
            .await?;
        assert!(
            l1_receipt.status(),
            "L1 sendUnsignedTransaction tx reverted"
        );
        println!(
            "[unsigned] L1 tx confirmed: {} (block {})",
            l1_receipt.transaction_hash(),
            l1_receipt.block_number().unwrap_or_default(),
        );

        println!("[unsigned] advancing L1 by 4 blocks...");
        ctx.advance_l1_blocks(4).await?;

        println!("[unsigned] waiting for unsigned submission on L2 (timeout=120s)...");
        let l2_hash = {
            let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(120);
            let mut scan_from = since;
            let mut found = None;
            while found.is_none() {
                if tokio::time::Instant::now() >= deadline {
                    break;
                }
                let latest = ctx.arbitrum_provider.get_block_number().await?;
                let mut bn = scan_from;
                while bn <= latest {
                    let block = ctx
                        .arbitrum_provider
                        .get_block(alloy_eips::BlockId::Number(
                            alloy_rpc_types_eth::BlockNumberOrTag::Number(bn),
                        ))
                        .await?;
                    let Some(block) = block else {
                        bn = bn.saturating_add(1);
                        continue;
                    };
                    for hash in block.transactions.hashes() {
                        let hash = B256::new(*hash);
                        match ctx.arbitrum_provider.get_transaction_by_hash(hash).await {
                            Ok(Some(tx)) => {
                                let ty = tx.inner.ty();
                                let from = tx.from();
                                let nonce = tx.nonce();
                                let is_expected_unsigned =
                                    from == l2_sender && nonce == l2_nonce && ty == 0x65;
                                if is_expected_unsigned {
                                    println!(
                                        "[unsigned] block {bn} tx {hash} type=0x{ty:02x} from={from} nonce={nonce}"
                                    );
                                    found = Some(hash);
                                }
                            }
                            Ok(None) => {}
                            Err(e) => println!("[unsigned] block {bn} tx {hash} → error: {e}"),
                        }
                    }
                    bn = bn.saturating_add(1);
                }
                scan_from = latest.saturating_add(1);
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
            found
        };
        let Some(l2_hash) = l2_hash else {
            return Err(
                format!(
                    "timeout waiting for unsigned tx from aliased sender {l2_sender} nonce {l2_nonce} since block {since}"
                )
                .into(),
            );
        };
        println!("[unsigned] found L2 unsigned tx: {l2_hash}");

        let receipt = ctx
            .arbitrum_provider
            .get_transaction_receipt(l2_hash)
            .await?;
        assert!(
            receipt.is_some(),
            "missing L2 receipt for unsigned tx {l2_hash}"
        );
        println!(
            "[unsigned] L2 receipt: block {}",
            receipt.unwrap().block_number().unwrap_or_default(),
        );

        Ok(())
    }
}
