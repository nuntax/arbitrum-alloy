//! ArbOS `Initialize` message (L1 message kind 11) parser.
//!
//! Mirrors Nitro `arbos/arbostypes/incomingmessage.go::ParseInitMessage`.

use alloy_primitives::{Address, U256};
use base64::prelude::*;
use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};

use crate::sequencer::feed::{L1IncomingMessage, MessageType};

/// Default initial L1 base fee: 50 GWei (matches Nitro `DefaultInitialL1BaseFee`).
pub const DEFAULT_INITIAL_L1_BASE_FEE: u64 = 50_000_000_000;

/// Top-level Arbitrum chain configuration, as serialized in the Initialize message's L2msg JSON.
///
/// JSON keys follow Nitro `go-ethereum/params/config.go` (top-level geth standard):
/// - `"chainId"` (lowercase) — standard geth field
/// - `"arbitrum"` (lowercase) — wraps `ArbitrumChainParams`
///
/// Unknown fields are silently ignored (`deny_unknown_fields` is NOT set).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArbChainConfig {
    /// Chain ID (top-level `"chainId"` in geth ChainConfig JSON).
    pub chain_id: u64,
    /// Arbitrum-specific parameters (`"arbitrum"` object).
    pub arbitrum: ArbitrumChainParams,
}

/// Arbitrum-specific chain parameters nested inside the chain config JSON.
///
/// JSON keys are the verbatim Go field names from `ArbitrumChainParams` in
/// `nitro/go-ethereum/params/config_arbitrum.go` — those fields have no `json:` tags so the key
/// equals the exported Go field name (PascalCase).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArbitrumChainParams {
    /// `"InitialArbOSVersion"` — ArbOS version to start at genesis.
    #[serde(rename = "InitialArbOSVersion")]
    pub initial_arbos_version: u64,

    /// `"InitialChainOwner"` — address that owns the chain at genesis.
    #[serde(rename = "InitialChainOwner")]
    pub initial_chain_owner: Address,

    /// `"GenesisBlockNum"` — L1 block number at which the chain was created.
    #[serde(rename = "GenesisBlockNum")]
    pub genesis_block_num: u64,

    /// `"AllowDebugPrecompiles"` — whether debug precompiles are available.
    #[serde(rename = "AllowDebugPrecompiles")]
    pub allow_debug_precompiles: bool,
}

/// Parsed result of an ArbOS Initialize message.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedInitMessage {
    /// Chain ID extracted from the first 32 bytes of the L2msg payload.
    pub chain_id: U256,
    /// Initial L1 base fee. Either read from the message (version ≥ 1) or the default 50 GWei.
    pub initial_l1_base_fee: U256,
    /// Raw JSON bytes of the chain config (empty when the message is 32 bytes only).
    pub serialized_chain_config: Vec<u8>,
    /// Parsed chain config (None when the message is 32 bytes only).
    pub chain_config: Option<ArbChainConfig>,
}

/// Parse an ArbOS Initialize message (`L1IncomingMessage` with kind == 11).
///
/// Mirrors `nitro/arbos/arbostypes/incomingmessage.go::ParseInitMessage`.
///
/// # Errors
///
/// Returns an error if:
/// - `msg.header.kind` is not `MessageType::Initialize`.
/// - Base64 decoding of `msg.l2msg` fails.
/// - The decoded payload is shorter than 32 bytes.
/// - The version byte is not 0 or 1.
/// - The chain config JSON cannot be parsed.
pub fn parse_init_message(msg: &L1IncomingMessage) -> Result<ParsedInitMessage> {
    if MessageType::from_u8(msg.header.kind) != MessageType::Initialize {
        return Err(eyre!(
            "parse_init_message: expected kind {} (Initialize), got {}",
            MessageType::Initialize as u8,
            msg.header.kind,
        ));
    }

    // `l2msg` is a base64 String in the feed JSON — decode exactly like the L2Message arm.
    let data = BASE64_STANDARD.decode(&msg.l2msg)?;

    if data.len() < 32 {
        return Err(eyre!(
            "invalid init message data: expected at least 32 bytes, got {}",
            data.len()
        ));
    }

    // First 32 bytes: chain ID (big-endian U256).
    let chain_id = U256::from_be_slice(&data[0..32]);

    // Exactly 32 bytes: no version, no config, default base fee.
    if data.len() == 32 {
        return Ok(ParsedInitMessage {
            chain_id,
            initial_l1_base_fee: U256::from(DEFAULT_INITIAL_L1_BASE_FEE),
            serialized_chain_config: Vec::new(),
            chain_config: None,
        });
    }

    // data.len() > 32: byte 32 is the format version.
    let version = data[32];
    // `rest` starts at byte 33 (after chain_id + version byte).
    let mut rest = &data[33..];

    let mut base_fee = U256::from(DEFAULT_INITIAL_L1_BASE_FEE);

    match version {
        1 => {
            // Version 1: next 32 bytes are the initial L1 base fee (big-endian U256).
            if rest.len() < 32 {
                return Err(eyre!(
                    "init message version 1: expected 32 base-fee bytes, got {}",
                    rest.len()
                ));
            }
            base_fee = U256::from_be_slice(&rest[0..32]);
            rest = &rest[32..];
            // Fall through to version 0: remaining bytes are the chain config JSON.
        }
        0 => {
            // Version 0: no explicit base fee — rest is the chain config JSON directly.
        }
        other => {
            return Err(eyre!("init message: unknown version {}", other));
        }
    }

    // Remaining bytes are the serialized chain config JSON.
    let serialized_chain_config = rest.to_vec();
    let chain_config: ArbChainConfig = serde_json::from_slice(&serialized_chain_config)
        .map_err(|e| eyre!("init message: failed to parse chain config JSON: {}", e))?;

    Ok(ParsedInitMessage {
        chain_id,
        initial_l1_base_fee: base_fee,
        serialized_chain_config,
        chain_config: Some(chain_config),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sequencer::feed::{Header, L1IncomingMessage};

    /// Build an `L1IncomingMessage` for the Initialize kind from raw payload bytes.
    fn init_msg_from_bytes(payload: &[u8]) -> L1IncomingMessage {
        L1IncomingMessage {
            header: Header {
                kind: MessageType::Initialize as u8,
                ..Default::default()
            },
            l2msg: BASE64_STANDARD.encode(payload),
            ..Default::default()
        }
    }

    // -----------------------------------------------------------------------
    // Helpers for building payloads
    // -----------------------------------------------------------------------

    fn chain_id_bytes(id: u64) -> [u8; 32] {
        let mut buf = [0u8; 32];
        buf[24..].copy_from_slice(&id.to_be_bytes());
        buf
    }

    fn u256_be_bytes(val: u64) -> [u8; 32] {
        let mut buf = [0u8; 32];
        buf[24..].copy_from_slice(&val.to_be_bytes());
        buf
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    /// Version-1 message: chain_id + version byte + base_fee + JSON config.
    /// Asserts exact parsed values as required by the task spec.
    #[test]
    fn parse_init_v1_exact_values() {
        let chain_id_val: u64 = 412346;
        let base_fee_val: u64 = 70_000_000_000;
        let json = br#"{"chainId":412346,"arbitrum":{"InitialArbOSVersion":32,"InitialChainOwner":"0x00000000000000000000000000000000000a11ce","GenesisBlockNum":0,"AllowDebugPrecompiles":true}}"#;

        let mut payload = Vec::new();
        payload.extend_from_slice(&chain_id_bytes(chain_id_val));
        payload.push(1u8); // version = 1
        payload.extend_from_slice(&u256_be_bytes(base_fee_val));
        payload.extend_from_slice(json);

        let msg = init_msg_from_bytes(&payload);
        let parsed = parse_init_message(&msg).expect("parse_init_message failed");

        assert_eq!(parsed.chain_id, U256::from(412346u64), "chain_id mismatch");
        assert_eq!(
            parsed.initial_l1_base_fee,
            U256::from(70_000_000_000u64),
            "base_fee mismatch"
        );

        let cfg = parsed.chain_config.expect("chain_config should be Some");
        assert_eq!(cfg.chain_id, 412346u64, "config chain_id mismatch");
        assert_eq!(
            cfg.arbitrum.initial_arbos_version, 32,
            "initial_arbos_version mismatch"
        );
        assert_eq!(
            cfg.arbitrum.initial_chain_owner,
            "0x00000000000000000000000000000000000a11ce"
                .parse::<Address>()
                .unwrap(),
            "initial_chain_owner mismatch"
        );
        assert_eq!(cfg.arbitrum.genesis_block_num, 0, "genesis_block_num mismatch");
        assert!(cfg.arbitrum.allow_debug_precompiles, "allow_debug_precompiles must be true");

        // serialized_chain_config must round-trip to the same JSON bytes
        assert_eq!(
            parsed.serialized_chain_config,
            json.to_vec(),
            "serialized_chain_config mismatch"
        );
    }

    /// Version-0 message: chain_id + version byte + JSON config.
    /// Base fee must default to 50 GWei (DEFAULT_INITIAL_L1_BASE_FEE).
    #[test]
    fn parse_init_v0_default_base_fee() {
        let chain_id_val: u64 = 412346;
        let json = br#"{"chainId":412346,"arbitrum":{"InitialArbOSVersion":32,"InitialChainOwner":"0x00000000000000000000000000000000000a11ce","GenesisBlockNum":0,"AllowDebugPrecompiles":false}}"#;

        let mut payload = Vec::new();
        payload.extend_from_slice(&chain_id_bytes(chain_id_val));
        payload.push(0u8); // version = 0
        payload.extend_from_slice(json);

        let msg = init_msg_from_bytes(&payload);
        let parsed = parse_init_message(&msg).expect("parse_init_message failed");

        assert_eq!(parsed.chain_id, U256::from(412346u64));
        assert_eq!(
            parsed.initial_l1_base_fee,
            U256::from(DEFAULT_INITIAL_L1_BASE_FEE),
            "v0 must use default base fee"
        );

        let cfg = parsed.chain_config.expect("chain_config should be Some");
        assert_eq!(cfg.chain_id, 412346u64);
        assert_eq!(cfg.arbitrum.initial_arbos_version, 32);
        assert!(!cfg.arbitrum.allow_debug_precompiles);
        assert_eq!(parsed.serialized_chain_config, json.to_vec());
    }

    /// 32-byte-only message: chain_id set, chain_config None, base_fee = 50 GWei.
    #[test]
    fn parse_init_32_bytes_only() {
        let payload = chain_id_bytes(99999u64);
        let msg = init_msg_from_bytes(&payload);
        let parsed = parse_init_message(&msg).expect("parse_init_message failed");

        assert_eq!(parsed.chain_id, U256::from(99999u64), "chain_id mismatch");
        assert_eq!(
            parsed.initial_l1_base_fee,
            U256::from(DEFAULT_INITIAL_L1_BASE_FEE),
            "32-byte message must use default base fee"
        );
        assert!(parsed.chain_config.is_none(), "chain_config must be None");
        assert!(
            parsed.serialized_chain_config.is_empty(),
            "serialized_chain_config must be empty"
        );
    }

    /// Payload shorter than 32 bytes must error.
    #[test]
    fn parse_init_too_short_errors() {
        let msg = init_msg_from_bytes(&[0u8; 16]);
        assert!(parse_init_message(&msg).is_err());
    }

    /// Wrong kind byte must error.
    #[test]
    fn parse_init_wrong_kind_errors() {
        let payload = chain_id_bytes(1u64);
        let mut msg = init_msg_from_bytes(&payload);
        msg.header.kind = MessageType::L2Message as u8; // wrong kind
        assert!(parse_init_message(&msg).is_err());
    }

    /// Unknown version byte must error.
    #[test]
    fn parse_init_unknown_version_errors() {
        let mut payload = chain_id_bytes(1u64).to_vec();
        payload.push(42u8); // unknown version
        payload.extend_from_slice(b"{}");
        let msg = init_msg_from_bytes(&payload);
        assert!(parse_init_message(&msg).is_err());
    }
}
