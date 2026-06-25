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

/// Arbitrum L1-originated contract transaction (`type = 0x66`).
#[derive(PartialEq, Debug, Clone, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxContract {
    /// Arbitrum chain identifier.
    #[serde(alias = "chain_id")]
    pub chain_id: U256,
    /// L1 request identifier for this transaction.
    #[serde(alias = "request_id")]
    pub request_id: B256,
    /// Sender address supplied by ArbOS.
    pub from: Address,
    /// Maximum fee per gas.
    #[serde(alias = "maxFeePerGas")]
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

impl TxContract {
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
        self.request_id.encode(out);
        self.from.encode(out);
        self.gas_fee_cap.encode(out);
        self.gas_limit.encode(out);
        self.to.encode(out);
        self.value.encode(out);
        self.input.encode(out);
    }

    /// Returns the encoded RLP payload length for the inner fields.
    pub fn rlp_encoded_fields_length(&self) -> usize {
        self.chain_id.length()
            + self.request_id.length()
            + self.from.length()
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
        let request_id: B256 = Decodable::decode(buf)?;
        let from: Address = Decodable::decode(buf)?;
        let gas_fee_cap: U256 = Decodable::decode(buf)?;
        let gas_limit: u64 = Decodable::decode(buf)?;
        let to: TxKind = Decodable::decode(buf)?;
        let value: U256 = Decodable::decode(buf)?;
        let input: Bytes = Decodable::decode(buf)?;
        Ok(Self {
            chain_id,
            request_id,
            from,
            gas_fee_cap,
            gas_limit,
            to,
            value,
            input,
        })
    }
}

impl Typed2718 for TxContract {
    fn ty(&self) -> u8 {
        ArbTxType::Contract as u8
    }
}

impl Decodable for TxContract {
    fn decode(data: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Self::rlp_decode(data)
    }
}

impl Decodable2718 for TxContract {
    fn typed_decode(ty: u8, buf: &mut &[u8]) -> Eip2718Result<Self> {
        if ty != ArbTxType::Contract as u8 {
            return Err(Eip2718Error::UnexpectedType(ty));
        }
        Ok(Self::rlp_decode(buf)?)
    }

    fn fallback_decode(buf: &mut &[u8]) -> Eip2718Result<Self> {
        Ok(Self::decode(buf)?)
    }
}

impl Encodable2718 for TxContract {
    fn encode_2718_len(&self) -> usize {
        self.rlp_encoded_length() + 1
    }

    fn encode_2718(&self, out: &mut dyn BufMut) {
        out.put_u8(self.ty());
        self.rlp_encode(out);
    }
}

impl Transaction for TxContract {
    fn chain_id(&self) -> Option<ChainId> {
        Some(self.chain_id.to())
    }

    fn nonce(&self) -> u64 {
        0
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

impl Sealable for TxContract {
    fn hash_slow(&self) -> B256 {
        self.tx_hash()
    }
}

#[cfg(test)]
mod tests {
    use alloy_eips::Typed2718;
    use alloy_network_primitives::ReceiptResponse;
    use alloy_primitives::{address, b256, uint};
    use alloy_provider::Provider;
    use serial_test::serial;
    use test_utils::{ContractTxParams, TestContext, dev_address};

    use super::TxContract;

    #[test]
    fn deserialize_rpc_shape_supports_max_fee_per_gas() {
        let raw = r#"{
            "chainId":"0x64aba",
            "requestId":"0x46f8b53a9fbf03f72358216811dc1d1b64d8c26f9b99751ca2e5bc0fd8824f6d",
            "from":"0x502fae7d46d88f08fc2f8ed27fcb2ab183eb3e1f",
            "gas":"0x186a0",
            "maxFeePerGas":"0x3b9aca00",
            "to":"0x3f1eae7d46d88f08fc2f8ed27fcb2ab183eb2d0e",
            "value":"0x0",
            "input":"0x"
        }"#;

        let tx: TxContract = serde_json::from_str(raw).expect("valid TxContract RPC shape");

        assert_eq!(tx.ty(), 0x66);
        assert_eq!(
            tx.request_id,
            b256!("46f8b53a9fbf03f72358216811dc1d1b64d8c26f9b99751ca2e5bc0fd8824f6d")
        );
        assert_eq!(
            tx.from,
            address!("0x502fae7d46d88f08fc2f8ed27fcb2ab183eb3e1f")
        );
        assert_eq!(tx.gas_limit, 100_000);
        assert_eq!(tx.gas_fee_cap, uint!(1_000_000_000_U256));
    }

    #[tokio::test]
    #[serial]
    async fn send_contract_tx_produces_contract_tx_on_l2() -> Result<(), Box<dyn std::error::Error>>
    {
        let Some(ctx) = TestContext::try_from_env().await else {
            eprintln!("ARBITRUM_RPC/ETHEREUM_RPC not set — skipping");
            return Ok(());
        };

        let since = ctx.arbitrum_provider.get_block_number().await?;
        println!("[contract] scanning L2 from block {since}");

        let dev_addr = dev_address();
        println!("[contract] dev addr (L2 target): {dev_addr}");

        println!("[contract] submitting sendContractTransaction on L1...");
        let l1_receipt = ctx
            .send_contract_transaction(ContractTxParams {
                gas_limit: uint!(100_000_U256),
                max_fee_per_gas: uint!(1_000_000_000_U256),
                to: dev_addr,
                data: Default::default(),
            })
            .await?;
        assert!(
            l1_receipt.status(),
            "L1 sendContractTransaction tx reverted"
        );
        println!(
            "[contract] L1 tx confirmed: {} (block {})",
            l1_receipt.transaction_hash(),
            l1_receipt.block_number().unwrap_or_default(),
        );

        println!("[contract] advancing L1 by 2 blocks...");
        ctx.advance_l1_blocks(2).await?;

        println!("[contract] waiting for TxContract (0x66) on L2...");
        let l2_hash = ctx
            .wait_for_l2_tx_type(0x66, since, std::time::Duration::from_secs(60))
            .await?;
        println!("[contract] found L2 TxContract: {l2_hash}");

        let receipt = ctx
            .arbitrum_provider
            .get_transaction_receipt(l2_hash)
            .await?;
        assert!(
            receipt.is_some(),
            "missing L2 receipt for TxContract {l2_hash}"
        );
        println!(
            "[contract] L2 receipt: block {}",
            receipt.unwrap().block_number().unwrap_or_default(),
        );

        Ok(())
    }
}
