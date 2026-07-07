#![allow(missing_docs)]

use alloy_consensus::TxReceipt;
use alloy_eips::{Decodable2718, Encodable2718, Typed2718};
use alloy_primitives::hex;
use arbitrum_alloy_consensus::ArbReceiptEnvelope;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct FixtureFile {
    vectors: Vec<ReceiptFixture>,
}

#[derive(Debug, Deserialize)]
struct ReceiptFixture {
    name: String,
    receipt_type: String,
    raw: String,
    expect: AccessorFixture,
}

#[derive(Debug, Deserialize)]
struct AccessorFixture {
    status: bool,
    cumulative_gas_used: u64,
    gas_used_for_l1: u64,
    logs_len: usize,
}

#[test]
fn nitro_receipt_vectors_roundtrip_and_accessors() {
    let file: FixtureFile =
        serde_json::from_str(include_str!("../testdata/nitro_receipt_fixtures.json"))
            .expect("fixture JSON should deserialize");

    for fixture in file.vectors {
        let raw = parse_hex_bytes(&fixture.raw);
        let mut buf = raw.as_slice();

        let receipt: ArbReceiptEnvelope = ArbReceiptEnvelope::decode_2718(&mut buf)
            .unwrap_or_else(|e| panic!("{}: decode_2718 failed: {e}", fixture.name));
        assert!(
            buf.is_empty(),
            "{}: trailing bytes after decode",
            fixture.name
        );

        let expected_ty = parse_hex_u8(&fixture.receipt_type);
        assert_eq!(
            receipt.ty(),
            expected_ty,
            "{}: receipt type mismatch",
            fixture.name
        );

        let mut reencoded = Vec::new();
        receipt.encode_2718(&mut reencoded);
        assert_eq!(
            reencoded, raw,
            "{}: re-encoded bytes mismatch",
            fixture.name
        );

        assert_eq!(
            receipt.status(),
            fixture.expect.status,
            "{}: status mismatch",
            fixture.name
        );
        assert_eq!(
            receipt.cumulative_gas_used(),
            fixture.expect.cumulative_gas_used,
            "{}: cumulative_gas_used mismatch",
            fixture.name
        );
        assert_eq!(
            receipt.logs().len(),
            fixture.expect.logs_len,
            "{}: logs_len mismatch",
            fixture.name
        );
        assert_eq!(
            gas_used_for_l1(&receipt),
            fixture.expect.gas_used_for_l1,
            "{}: gas_used_for_l1 mismatch",
            fixture.name
        );
    }
}

const fn gas_used_for_l1<T>(receipt: &ArbReceiptEnvelope<T>) -> u64 {
    match receipt {
        ArbReceiptEnvelope::Legacy(r)
        | ArbReceiptEnvelope::Eip2930(r)
        | ArbReceiptEnvelope::Eip1559(r)
        | ArbReceiptEnvelope::Eip4844(r)
        | ArbReceiptEnvelope::Eip7702(r)
        | ArbReceiptEnvelope::Deposit(r)
        | ArbReceiptEnvelope::Unsigned(r)
        | ArbReceiptEnvelope::Contract(r)
        | ArbReceiptEnvelope::Retry(r)
        | ArbReceiptEnvelope::SubmitRetryable(r)
        | ArbReceiptEnvelope::Internal(r) => r.receipt.gas_used_for_l1,
    }
}

fn parse_hex_u8(s: &str) -> u8 {
    let stripped = s.strip_prefix("0x").unwrap_or(s);
    u8::from_str_radix(stripped, 16).expect("valid hex u8")
}

fn parse_hex_bytes(s: &str) -> Vec<u8> {
    let stripped = s.strip_prefix("0x").unwrap_or(s);
    hex::decode(stripped).expect("valid hex bytes")
}
