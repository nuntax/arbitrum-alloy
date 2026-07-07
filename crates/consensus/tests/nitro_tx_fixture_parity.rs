#![allow(missing_docs)]

use alloy_consensus::{Transaction, Typed2718};
use alloy_eips::{Decodable2718, Encodable2718};
use alloy_primitives::{Address, Bytes, TxKind, U256, hex};
use arbitrum_alloy_consensus::ArbTxEnvelope;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct FixtureFile {
    vectors: Vec<TxFixture>,
}

#[derive(Debug, Deserialize)]
struct TxFixture {
    name: String,
    tx_type: String,
    raw: String,
    hash: String,
    expect: AccessorFixture,
}

#[derive(Debug, Deserialize)]
struct AccessorFixture {
    from: String,
    to: Option<String>,
    nonce: u64,
    gas_limit: u64,
    value: String,
    input: String,
}

#[test]
fn nitro_tx_vectors_roundtrip_hash_and_accessors() {
    let file: FixtureFile =
        serde_json::from_str(include_str!("../testdata/nitro_tx_fixtures.json"))
            .expect("fixture JSON should deserialize");

    for fixture in file.vectors {
        let raw = parse_hex_bytes(&fixture.raw);
        let mut buf = raw.as_slice();

        let tx = ArbTxEnvelope::decode_2718(&mut buf)
            .unwrap_or_else(|e| panic!("{}: decode_2718 failed: {e}", fixture.name));
        assert!(
            buf.is_empty(),
            "{}: trailing bytes after decode",
            fixture.name
        );

        let expected_ty = parse_hex_u8(&fixture.tx_type);
        assert_eq!(tx.ty(), expected_ty, "{}: tx type mismatch", fixture.name);

        let mut reencoded = Vec::new();
        tx.encode_2718(&mut reencoded);
        assert_eq!(
            reencoded, raw,
            "{}: re-encoded bytes mismatch",
            fixture.name
        );

        let got_hash = format!("{:#x}", tx.hash());
        assert_eq!(
            got_hash,
            fixture.hash.to_ascii_lowercase(),
            "{}: tx hash mismatch",
            fixture.name
        );

        let sender = tx
            .sender()
            .unwrap_or_else(|e| panic!("{}: sender recovery failed: {e}", fixture.name));
        assert_eq!(
            sender,
            fixture
                .expect
                .from
                .parse::<Address>()
                .expect("valid expected from address"),
            "{}: from mismatch",
            fixture.name
        );

        let expected_to = fixture
            .expect
            .to
            .as_deref()
            .map(|addr| addr.parse::<Address>().expect("valid expected to address"));
        let actual_to = match tx.kind() {
            TxKind::Call(to) => Some(to),
            TxKind::Create => None,
        };

        assert_eq!(actual_to, expected_to, "{}: to mismatch", fixture.name);
        assert_eq!(
            tx.nonce(),
            fixture.expect.nonce,
            "{}: nonce mismatch",
            fixture.name
        );
        assert_eq!(
            tx.gas_limit(),
            fixture.expect.gas_limit,
            "{}: gas_limit mismatch",
            fixture.name
        );
        assert_eq!(
            tx.value(),
            parse_hex_u256(&fixture.expect.value),
            "{}: value mismatch",
            fixture.name
        );
        assert_eq!(
            tx.input(),
            &Bytes::from(parse_hex_bytes(&fixture.expect.input)),
            "{}: input mismatch",
            fixture.name
        );
    }
}

fn parse_hex_u8(s: &str) -> u8 {
    let stripped = s.strip_prefix("0x").unwrap_or(s);
    u8::from_str_radix(stripped, 16).expect("valid hex u8")
}

fn parse_hex_u256(s: &str) -> U256 {
    let stripped = s.strip_prefix("0x").unwrap_or(s);
    if stripped.is_empty() {
        return U256::ZERO;
    }
    U256::from_str_radix(stripped, 16).expect("valid hex U256")
}

fn parse_hex_bytes(s: &str) -> Vec<u8> {
    let stripped = s.strip_prefix("0x").unwrap_or(s);
    hex::decode(stripped).expect("valid hex bytes")
}
