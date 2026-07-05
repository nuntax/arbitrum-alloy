use alloy_consensus::TxEip1559;
use alloy_consensus::TxEip2930;
use alloy_consensus::TxEip7702;
use alloy_consensus::TxLegacy;
use alloy_consensus::transaction::RlpEcdsaDecodableTx;
use alloy_primitives::Address;
use alloy_primitives::B256;
use alloy_primitives::Bytes;
use alloy_primitives::ChainId;
use alloy_primitives::FixedBytes;
use alloy_primitives::TxKind;
use alloy_primitives::U256;
use alloy_primitives::hex::FromHex;
use alloy_primitives::keccak256;
use arb_alloy_consensus::transactions::ArbTxEnvelope;
use arb_alloy_consensus::transactions::TxContract;
use arb_alloy_consensus::transactions::TxDeposit;
use arb_alloy_consensus::transactions::TxUnsigned;
use arb_alloy_consensus::transactions::batchpostingreport::decode_fields_sequencer as decode_batch_posting_report;
use arb_alloy_consensus::transactions::submit_retryable::SubmitRetryableTx;
use crate::sequencer::feed::L1IncomingMessage;
use crate::sequencer::feed::MessageType;
use base64::prelude::*;
use eyre::Result;
use eyre::eyre;
use std::io::{Cursor, Read};
use std::str::FromStr;

/// Decode a single feed `L1IncomingMessage` into the transactions it produces, dispatching on the
/// L1 message kind. Mirrors Nitro `arbos/parse_l2.go::ParseL2Transactions`.
pub fn parse_message(
    msg: L1IncomingMessage,
    chain_id: ChainId,
    version: u8,
) -> Result<Vec<ArbTxEnvelope>> {
    let msg_type = MessageType::from_u8(msg.header.kind);
    tracing::debug!("Parsing message type: {:?}", msg_type);

    match msg_type {
        MessageType::L2Message => {
            tracing::debug!("Decoding L2Message base64 content");
            let mut buffer = match BASE64_STANDARD.decode(msg.l2msg) {
                Ok(buf) => {
                    tracing::debug!("Successfully decoded base64 of length: {}", buf.len());
                    buf
                }
                Err(e) => {
                    tracing::error!("Failed to decode base64: {}", e);
                    return Err(e.into());
                }
            };

            // Nitro rejects an over-large l2msg up front (before the kind switch).
            if buffer.len() as u64 > MAX_L2_MESSAGE_SIZE {
                return Err(eyre!("message too large"));
            }

            // Poster = header sender; request id is present only for L1-originated L2 messages
            // (`None` for ordinary sequencer messages). Both are threaded into `parseUnsignedTx`.
            let poster: Address = msg
                .header
                .sender
                .parse()
                .map_err(|_| eyre!("L2Message: invalid poster/sender address"))?;
            let request_id: Option<B256> =
                msg.header.request_id.as_str().and_then(|s| FixedBytes::from_hex(s).ok());

            match parse_l2_msg(buffer.as_mut_slice(), 0, poster, request_id, chain_id) {
                Ok(txs) => {
                    tracing::debug!("Successfully parsed {} L2 transactions", txs.len());
                    Ok(txs)
                }
                Err(e) => {
                    tracing::error!("Failed to parse L2 message: {}", e);
                    Err(e)
                }
            }
        }
        // Nitro returns `nil, nil` — the message produces no transactions.
        MessageType::EndOfBlock => Ok(Vec::new()),
        MessageType::EthDeposit => {
            let mut buffer_vec = BASE64_STANDARD.decode(msg.l2msg)?;
            let buffer = buffer_vec.as_mut_slice();
            tracing::debug!("Buffer: {}", hex::encode(&buffer));
            let tx = TxDeposit::decode_fields_sequencer(
                &mut &*buffer,
                U256::from(chain_id),
                FixedBytes::from_hex(
                    msg.header
                        .request_id
                        .as_str()
                        .ok_or(eyre!("failed to deserialize request_id"))?,
                )?,
                msg.header.sender.parse()?,
            )?;
            tracing::debug!("Parsed TxDeposit: {:?}", tx);
            tracing::debug!("TxDeposit hash: {}", tx.tx_hash());
            Ok(vec![tx.into()])
        }
        MessageType::SubmitRetryable => {
            let mut buffer_vec = BASE64_STANDARD.decode(msg.l2msg.clone())?;
            let buffer = buffer_vec.as_mut_slice();
            //log the whole message and the buffer
            tracing::debug!("Retyrable message: {:?}", msg);
            tracing::debug!("Retryable message buffer: {}", hex::encode(&buffer));

            let tx = parse_submit_retryable(
                &mut &*buffer,
                chain_id,
                Address::from_str(&msg.header.sender).unwrap(),
                B256::from_hex(
                    msg.header
                        .request_id
                        .as_str()
                        .ok_or(eyre!("failed to deserialize request_id"))?,
                )?,
                U256::from(
                    msg.header
                        .base_fee_l1
                        .as_u64()
                        .ok_or(eyre!("failed to deserialize base fee l1"))?,
                ),
            )?;
            Ok(vec![tx.into()])
        }
        MessageType::BatchPostingReport => {
            let mut buffer_vec = BASE64_STANDARD.decode(msg.l2msg)?;
            let buffer = buffer_vec.as_mut_slice();
            tracing::debug!("BatchPostingReport Buffer: {}", hex::encode(&buffer));
            tracing::debug!(
                "Additional args: chain_id: {}, version: {}, batch_data_stats: {:?}, legacy_batch_gas_cost: {:?}",
                chain_id,
                version,
                msg.batch_data_stats,
                msg.legacy_batch_gas_cost
            );
            let internal_tx = decode_batch_posting_report(
                &mut &*buffer,
                chain_id,
                version.into(),
                msg.batch_data_stats,
                msg.legacy_batch_gas_cost,
            )?;
            Ok(vec![internal_tx.into()])
        }
        // Nitro ignores rollup-event messages (returns an empty transaction list).
        MessageType::RollupEvent => {
            tracing::debug!("ignoring rollup event message");
            Ok(Vec::new())
        }
        // The Initialize message (kind 11) carries ArbOS genesis data; it produces no L2
        // transactions. Callers that need the genesis payload should call
        // `init_message::parse_init_message` directly.
        MessageType::Initialize => {
            tracing::debug!("ignoring Initialize message (no L2 transactions)");
            Ok(Vec::new())
        }
        // Everything else is either tx-producing-but-unported (L2FundedByL1) or a hard Nitro error
        // (BatchForGasEstimation, Invalid, ...). Fail loudly: silently returning an
        // empty list here would drop real transactions and diverge the state root.
        // TODO(stage-e): implement L2FundedByL1 (deposit + parseUnsignedTx) when the corpus needs it.
        _ => Err(eyre!("unsupported L1 message type: {:?}", msg_type)),
    }
}

/// Decode the fixed-width fields of a `SubmitRetryable` L1 message body into a `SubmitRetryableTx`.
/// Mirrors Nitro `arbos/parse_l2.go::parseSubmitRetryableMessage`.
pub fn parse_submit_retryable(
    msg: &mut &[u8],
    chain_id: ChainId,
    sender: Address,
    request_id: B256,
    l1_base_fee: U256,
) -> Result<SubmitRetryableTx> {
    let tx = SubmitRetryableTx::decode_fields_sequencer(
        msg,
        U256::from(chain_id),
        request_id,
        sender,
        l1_base_fee,
    )?;

    tracing::debug!(
        "Parsed TxSubmitRetryable: chain_id: {}, request_id: {}, sender: {}",
        chain_id,
        request_id,
        sender
    );
    Ok(tx)
}

const MAX_BATCH_DEPTH: u32 = 16;
const MAX_L2_MESSAGE_SIZE: u64 = 256 * 1024; // 256KB

/// L2MessageKind represents the kind of message that can be received from the Arbitrum sequencer.
///
/// Discriminants MUST match Nitro `arbos/parse_l2.go` (`L2MessageKind_*`): `5` is reserved,
/// `Heartbeat` is `6`, `SignedCompressedTx` is `7`, and `8` is reserved for BLS-signed batches.
#[derive(Debug, PartialEq)]
pub enum L2MessageKind {
    /// Unsigned user transaction (Nitro `parseUnsignedTx`).
    UnsignedUserTx = 0,
    /// Contract transaction (Nitro `parseUnsignedTx`).
    ContractTx = 1,
    /// Non-mutating call. Unimplemented in the Nitro reference; only here for completeness.
    NonmutatingCall = 2,
    /// Batch transaction: a message that contains multiple (possibly nested) sub-messages.
    Batch = 3,
    /// Signed transaction: a single EIP-2718 signed transaction envelope.
    SignedTx = 4,
    // 5 is reserved.
    /// Heartbeat message. Deprecated since 2022-08-08; not used in Arbitrum anymore.
    Heartbeat = 6,
    /// Signed compressed transaction. Unimplemented in the Nitro reference.
    SignedCompressedTx = 7,
    // 8 is reserved for BLS-signed batches.
}

impl TryFrom<u8> for L2MessageKind {
    type Error = eyre::Report;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(L2MessageKind::UnsignedUserTx),
            1 => Ok(L2MessageKind::ContractTx),
            2 => Ok(L2MessageKind::NonmutatingCall),
            3 => Ok(L2MessageKind::Batch),
            4 => Ok(L2MessageKind::SignedTx),
            6 => Ok(L2MessageKind::Heartbeat),
            7 => Ok(L2MessageKind::SignedCompressedTx),
            _ => Err(eyre::eyre!("Unsupported L2 message kind: {}", value)),
        }
    }
}

/// Recursively decode an L2 message body (a `kind` byte followed by its payload) into the
/// transactions it carries. Mirrors Nitro `arbos/parse_l2.go::parseL2Message`.
///
/// `poster` is the message header's sender (used as the `From` of unsigned/contract txs);
/// `request_id` is the header's L1 request id (present for L1-originated messages, `None` for
/// ordinary sequencer messages), threaded into nested batch segments as
/// `keccak(request_id, index)` to match Nitro.
pub fn parse_l2_msg(
    bytes: &mut [u8],
    depth: u32,
    poster: Address,
    request_id: Option<B256>,
    chain_id: ChainId,
) -> Result<Vec<ArbTxEnvelope>> {
    if depth >= MAX_BATCH_DEPTH {
        return Err(eyre::eyre!("Maximum batch depth exceeded: {}", depth));
    }

    // Nitro reads the kind byte via `rd.Read`, which errors on an empty reader.
    if bytes.is_empty() {
        return Err(eyre!("L2 message kind missing"));
    }
    let kind = L2MessageKind::try_from(bytes[0])?;
    let mut transactions: Vec<ArbTxEnvelope> = Vec::new();

    match kind {
        // Nitro `parseUnsignedTx`: both kinds share a fixed-width layout and differ only in the
        // nonce field (UnsignedUserTx) vs the L1 request id (ContractTx).
        tk @ (L2MessageKind::UnsignedUserTx | L2MessageKind::ContractTx) => {
            let tx = parse_unsigned_tx(&bytes[1..], poster, request_id, chain_id, &tk)?;
            transactions.push(tx);
        }
        // Nitro: "L2 message kind NonmutatingCall is unimplemented".
        L2MessageKind::NonmutatingCall => {
            return Err(eyre!("L2 message kind NonmutatingCall is unimplemented"));
        }
        L2MessageKind::Batch => {
            let mut cursor = Cursor::new(&bytes[1..]); // skip the kind byte
            let mut index: u64 = 0;
            loop {
                // Each segment is an 8-byte big-endian length prefix followed by that many bytes.
                let mut length_buf = [0u8; 8];
                if cursor.read_exact(&mut length_buf).is_err() {
                    break; // no further segments in the batch
                }

                let msg_len = u64::from_be_bytes(length_buf);
                // Nitro's `BytestringFromReader` returns an error on an oversized length, which
                // `parseL2Message` treats as end-of-batch (`return segments, nil`). Match that:
                // stop gracefully rather than failing the whole message.
                if msg_len > MAX_L2_MESSAGE_SIZE {
                    break;
                }

                let mut msg_buf = vec![0u8; msg_len as usize];
                if cursor.read_exact(&mut msg_buf).is_err() {
                    break;
                }

                // Nested request id per Nitro `parseL2Message`: keccak(request_id, index) for each
                // segment, so a nested ContractTx derives the right id. `None` stays `None`.
                let nested_request_id = request_id.map(|rid| {
                    let mut buf = [0u8; 64];
                    buf[..32].copy_from_slice(rid.as_slice());
                    buf[32..].copy_from_slice(&U256::from(index).to_be_bytes::<32>());
                    keccak256(buf)
                });
                // recurse for nested batch / signed-tx segments
                let nested_txs =
                    parse_l2_msg(&mut msg_buf, depth + 1, poster, nested_request_id, chain_id)?;
                transactions.extend(nested_txs);
                index += 1;
            }
        }
        L2MessageKind::SignedTx => {
            let tx = parse_raw_tx(&bytes[1..])?;
            transactions.push(tx);
        }
        L2MessageKind::Heartbeat => {
            // Deprecated. Nitro errors if `timestamp >= HeartbeatsDisabledAt` (2022-08-08) and
            // otherwise does nothing; we lack the timestamp here and heartbeats never appear in
            // the modern feed, so we ignore them. (Known minor divergence pre-2022.)
        }
        // Nitro: "L2 message kind SignedCompressedTx is unimplemented".
        L2MessageKind::SignedCompressedTx => {
            return Err(eyre!("L2 message kind SignedCompressedTx is unimplemented"));
        }
    }

    Ok(transactions)
}

/// Decode an unsigned-tx L2 message body (Nitro `arbos/parse_l2.go::parseUnsignedTx`). `tx_kind`
/// selects `UnsignedUserTx` (carries a nonce; `From` is the poster) or `ContractTx` (carries the
/// L1 request id, no nonce). `body` is the message payload after the kind byte; the layout is
/// fixed-width big-endian:
///   gasLimit(32) | maxFeePerGas(32) | [nonce(32) — UnsignedUserTx only] | to(32, address in the
///   low 20 bytes; zero => contract creation) | value(32) | calldata(remaining bytes).
fn parse_unsigned_tx(
    mut body: &[u8],
    poster: Address,
    request_id: Option<B256>,
    chain_id: ChainId,
    tx_kind: &L2MessageKind,
) -> Result<ArbTxEnvelope> {
    fn take<'a>(cur: &mut &'a [u8], n: usize) -> Result<&'a [u8]> {
        if cur.len() < n {
            return Err(eyre!("unsigned tx: truncated field (need {n}, have {})", cur.len()));
        }
        let (head, tail) = cur.split_at(n);
        *cur = tail;
        Ok(head)
    }

    let gas_limit = u64::try_from(U256::from_be_slice(take(&mut body, 32)?))
        .map_err(|_| eyre!("unsigned user tx gas limit >= 2^64"))?;
    let gas_fee_cap = U256::from_be_slice(take(&mut body, 32)?);
    let nonce = if *tx_kind == L2MessageKind::UnsignedUserTx {
        u64::try_from(U256::from_be_slice(take(&mut body, 32)?))
            .map_err(|_| eyre!("unsigned user tx nonce >= 2^64"))?
    } else {
        0
    };
    // The target address is right-aligned in a 32-byte word; the zero address means create.
    let to_word = take(&mut body, 32)?;
    let to_addr = Address::from_slice(&to_word[12..32]);
    let to = if to_addr.is_zero() { TxKind::Create } else { TxKind::Call(to_addr) };
    let value = U256::from_be_slice(take(&mut body, 32)?);
    let input = Bytes::copy_from_slice(body); // remainder is calldata

    let env: ArbTxEnvelope = match tx_kind {
        L2MessageKind::UnsignedUserTx => TxUnsigned {
            chain_id: U256::from(chain_id),
            from: poster,
            nonce,
            gas_fee_cap,
            gas_limit,
            to,
            value,
            input,
        }
        .into(),
        L2MessageKind::ContractTx => TxContract {
            chain_id: U256::from(chain_id),
            request_id: request_id
                .ok_or_else(|| eyre!("cannot issue contract tx without L1 request id"))?,
            from: poster,
            gas_fee_cap,
            gas_limit,
            to,
            value,
            input,
        }
        .into(),
        _ => return Err(eyre!("invalid L2 tx type in parseUnsignedTx")),
    };
    Ok(env)
}

/// The EIP-2718 transaction types accepted inside an `L2MessageKind::SignedTx`.
///
/// Mirrors Nitro's SignedTx handling, which rejects blob transactions (`0x03`) and all Arbitrum
/// types (`>= ArbitrumDepositTxType`, i.e. `0x64`): a signed user transaction may only be a
/// standard Ethereum envelope.
#[derive(Debug)]
pub enum TxType {
    /// Legacy transaction. Indicated by an RLP-list first byte (> 0x7f).
    Legacy,
    /// EIP-2930 transaction. Indicated by type byte 0x01.
    Eip2930 = 1,
    /// EIP-1559 transaction. Indicated by type byte 0x02.
    Eip1559 = 2,
    /// EIP-7702 transaction. Indicated by type byte 0x04.
    Eip7702 = 4,
}

impl TxType {
    /// Converts an EIP-2718 type byte to a `TxType`, rejecting blob and Arbitrum types.
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            x if x > 0x7f => Ok(TxType::Legacy),
            1 => Ok(TxType::Eip2930),
            2 => Ok(TxType::Eip1559),
            4 => Ok(TxType::Eip7702),
            _ => Err(eyre::eyre!(
                "Invalid signed-tx type: {}. Nitro rejects blob (0x03) and Arbitrum (>=0x64) types.",
                value
            )),
        }
    }
}

fn parse_raw_tx(bytes: &[u8]) -> Result<ArbTxEnvelope> {
    let tx_type = bytes.first().ok_or(eyre!("Missing transaction type"))?;
    let tx_type = TxType::from_u8(*tx_type)?;
    let tx: ArbTxEnvelope = match tx_type {
        TxType::Legacy => ArbTxEnvelope::Legacy(TxLegacy::rlp_decode_signed(&mut &bytes[0..])?),
        TxType::Eip2930 => ArbTxEnvelope::Eip2930(TxEip2930::rlp_decode_signed(&mut &bytes[1..])?),
        TxType::Eip1559 => ArbTxEnvelope::Eip1559(TxEip1559::rlp_decode_signed(&mut &bytes[1..])?),
        TxType::Eip7702 => ArbTxEnvelope::Eip7702(TxEip7702::rlp_decode_signed(&mut &bytes[1..])?),
    };

    Ok(tx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sequencer::feed::{Header, L1IncomingMessage};

    const CHAIN_ID: ChainId = 42161;

    fn msg_with_kind(kind: u8) -> L1IncomingMessage {
        L1IncomingMessage {
            header: Header { kind, ..Default::default() },
            ..Default::default()
        }
    }

    /// `L2MessageKind` discriminants must match Nitro `L2MessageKind_*`: 5 reserved, Heartbeat=6,
    /// SignedCompressedTx=7. (The legacy port had Heartbeat=5/SignedCompressedTx=6 — off by one.)
    #[test]
    fn l2_message_kind_discriminants_match_nitro() {
        assert_eq!(L2MessageKind::UnsignedUserTx as u8, 0);
        assert_eq!(L2MessageKind::ContractTx as u8, 1);
        assert_eq!(L2MessageKind::NonmutatingCall as u8, 2);
        assert_eq!(L2MessageKind::Batch as u8, 3);
        assert_eq!(L2MessageKind::SignedTx as u8, 4);
        assert_eq!(L2MessageKind::Heartbeat as u8, 6);
        assert_eq!(L2MessageKind::SignedCompressedTx as u8, 7);
    }

    #[test]
    fn l2_message_kind_try_from_matches_nitro() {
        assert_eq!(L2MessageKind::try_from(4).unwrap(), L2MessageKind::SignedTx);
        assert!(L2MessageKind::try_from(5).is_err(), "5 is reserved");
        assert_eq!(L2MessageKind::try_from(6).unwrap(), L2MessageKind::Heartbeat);
        assert_eq!(
            L2MessageKind::try_from(7).unwrap(),
            L2MessageKind::SignedCompressedTx
        );
        assert!(L2MessageKind::try_from(8).is_err(), "8 is reserved (BLS)");
    }

    #[test]
    fn parse_l2_msg_empty_is_error_not_panic() {
        assert!(parse_l2_msg(&mut [], 0, Address::ZERO, None, CHAIN_ID).is_err());
    }

    #[test]
    fn parse_l2_msg_heartbeat_yields_no_txs() {
        assert!(parse_l2_msg(&mut [6], 0, Address::ZERO, None, CHAIN_ID).unwrap().is_empty());
    }

    #[test]
    fn parse_l2_msg_unimplemented_kinds_error() {
        // Genuinely unimplemented in Nitro too: NonmutatingCall(2) and SignedCompressedTx(7).
        for kind in [2u8, 7] {
            assert!(
                parse_l2_msg(&mut [kind], 0, Address::ZERO, None, CHAIN_ID).is_err(),
                "kind {kind} must error"
            );
        }
    }

    /// Build the fixed-width `parseUnsignedTx` body: gasLimit|maxFee|[nonce]|to|value|calldata.
    fn unsigned_body(kind: u8, gas: u64, max_fee: u64, nonce: u64, to: Address, value: u64, data: &[u8]) -> Vec<u8> {
        let mut b = vec![kind];
        let word = |x: u64| -> [u8; 32] { U256::from(x).to_be_bytes() };
        b.extend_from_slice(&word(gas));
        b.extend_from_slice(&word(max_fee));
        if kind == 0 {
            b.extend_from_slice(&word(nonce));
        }
        let mut to_word = [0u8; 32];
        to_word[12..].copy_from_slice(to.as_slice());
        b.extend_from_slice(&to_word);
        b.extend_from_slice(&word(value));
        b.extend_from_slice(data);
        b
    }

    #[test]
    fn parse_unsigned_user_tx_roundtrips() {
        let poster = Address::repeat_byte(0xab);
        let to = Address::repeat_byte(0xcd);
        let mut body = unsigned_body(0, 21_000, 1_000_000_000, 7, to, 42, b"hi");
        let txs = parse_l2_msg(&mut body, 0, poster, None, CHAIN_ID).unwrap();
        assert_eq!(txs.len(), 1);
        match &txs[0] {
            ArbTxEnvelope::Unsigned(tx) => {
                assert_eq!(tx.from, poster);
                assert_eq!(tx.nonce, 7);
                assert_eq!(tx.gas_limit, 21_000);
                assert_eq!(tx.to, TxKind::Call(to));
                assert_eq!(tx.value, U256::from(42));
                assert_eq!(&tx.input[..], b"hi");
            }
            other => panic!("expected Unsigned, got {other:?}"),
        }
    }

    #[test]
    fn parse_contract_tx_needs_request_id() {
        let mut body = unsigned_body(1, 21_000, 1, 0, Address::ZERO, 0, b"");
        // ContractTx (kind 1) with no request id errors, matching Nitro.
        assert!(parse_l2_msg(&mut body.clone(), 0, Address::ZERO, None, CHAIN_ID).is_err());
        // With a request id it decodes; zero `to` means contract creation.
        let txs = parse_l2_msg(&mut body, 0, Address::ZERO, Some(B256::repeat_byte(9)), CHAIN_ID).unwrap();
        match &txs[0] {
            ArbTxEnvelope::Contract(tx) => {
                assert_eq!(tx.request_id, B256::repeat_byte(9));
                assert_eq!(tx.to, TxKind::Create);
            }
            other => panic!("expected Contract, got {other:?}"),
        }
    }

    /// Batch framing (8-byte BE length prefix + payload) + recursion + Heartbeat=6, end to end.
    #[test]
    fn parse_l2_msg_batch_of_heartbeat_segment() {
        // kind=Batch(3), one segment of length 1 whose body is Heartbeat(6).
        let mut bytes = vec![3u8, 0, 0, 0, 0, 0, 0, 0, 1, 6];
        assert!(parse_l2_msg(&mut bytes, 0, Address::ZERO, None, CHAIN_ID).unwrap().is_empty());
    }

    /// An oversized segment length must stop the batch gracefully (Nitro `return segments, nil`),
    /// not hard-error.
    #[test]
    fn parse_l2_msg_batch_oversize_segment_breaks_gracefully() {
        let mut bytes = vec![3u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        assert!(parse_l2_msg(&mut bytes, 0, Address::ZERO, None, CHAIN_ID).unwrap().is_empty());
    }

    /// A SignedTx must reject blob (0x03) and Arbitrum (>=0x64) inner types, matching Nitro.
    #[test]
    fn signed_tx_rejects_blob_and_arbitrum_types() {
        assert!(parse_l2_msg(&mut [4, 0x03], 0, Address::ZERO, None, CHAIN_ID).is_err(), "blob rejected");
        assert!(parse_l2_msg(&mut [4, 0x64], 0, Address::ZERO, None, CHAIN_ID).is_err(), "deposit rejected");
        assert!(parse_l2_msg(&mut [4], 0, Address::ZERO, None, CHAIN_ID).is_err(), "missing inner type");
    }

    #[test]
    fn parse_message_end_of_block_is_empty_not_panic() {
        let m = msg_with_kind(MessageType::EndOfBlock as u8);
        assert!(parse_message(m, CHAIN_ID, 0).unwrap().is_empty());
    }

    #[test]
    fn parse_message_rollup_event_is_empty() {
        let m = msg_with_kind(MessageType::RollupEvent as u8);
        assert!(parse_message(m, CHAIN_ID, 0).unwrap().is_empty());
    }

    /// Initialize (kind 11) produces no L2 transactions — same as EndOfBlock.
    /// Callers that need genesis data use `init_message::parse_init_message` directly.
    #[test]
    fn parse_message_initialize_is_empty_not_error() {
        let m = msg_with_kind(MessageType::Initialize as u8);
        assert!(
            parse_message(m, CHAIN_ID, 0).unwrap().is_empty(),
            "Initialize must return empty Vec, not error"
        );
    }

    /// Tx-producing-but-unported and Nitro-error types must fail loudly, never silently drop.
    #[test]
    fn parse_message_unsupported_types_error() {
        for kind in [
            MessageType::L2FundedByL1 as u8,
            MessageType::BatchForGasEstimation as u8,
            MessageType::Invalid as u8,
        ] {
            assert!(
                parse_message(msg_with_kind(kind), CHAIN_ID, 0).is_err(),
                "kind {kind} must error, not return empty"
            );
        }
    }
}
