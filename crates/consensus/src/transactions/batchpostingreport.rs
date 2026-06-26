#![allow(clippy::too_many_arguments)]

use crate::transactions::internal::ArbInternalTx;
use crate::transactions::util::decode;
use alloy_core::sol;
use alloy_core::sol_types::SolCall;
use alloy_primitives::{Address, Bytes, ChainId, FixedBytes, U256};
use serde::{Deserialize, Serialize};

/// Batch data tokenization stats used by newer batch posting reports.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatchDataStats {
    /// Total byte length of batch data.
    #[serde(rename = "Length", alias = "length")]
    pub length: u64,
    /// Number of non-zero bytes in batch data.
    #[serde(rename = "NonZeros", alias = "nonzeros")]
    pub non_zeros: u64,
}
sol! {
   #[sol(rpc)]
   "./src/interfaces/ArbosActs.sol"
}

fn construct_batchpostreport_data(
    batch_timestamp: U256,
    batch_poster: Address,
    batch_num: u64,
    batchgas: u64,
    l1_base_fee: U256,
) -> Bytes {
    ArbosActs::batchPostingReportCall::new((
        batch_timestamp,
        batch_poster,
        batch_num,
        batchgas,
        l1_base_fee,
    ))
    .abi_encode()
    .into()
}

fn construct_batchreportv2_data(
    batch_timestamp: U256,
    batch_poster: Address,
    batch_num: u64,
    batch_data_stats_length: u64,
    batch_data_stats_non_zeros: u64,
    extra_gas: u64,
    l1_base_fee: U256,
) -> Bytes {
    ArbosActs::batchPostingReportV2Call::new((
        batch_timestamp,
        batch_poster,
        batch_num,
        batch_data_stats_length,
        batch_data_stats_non_zeros,
        extra_gas,
        l1_base_fee,
    ))
    .abi_encode()
    .into()
}

/// Decoded fields from an ArbOS batch posting report payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchPostingReportFields {
    /// Batch timestamp recorded in the report.
    pub batch_timestamp: U256,
    /// Address that posted the batch.
    pub batch_poster: Address,
    /// Batch sequence number.
    pub batch_num: u64,
    /// L1 base fee used for cost accounting.
    pub l1_base_fee: U256,
    /// Additional gas charged by ArbOS for this batch.
    pub extra_gas: u64,
    /// Hash of the reported batch data.
    pub data_hash: FixedBytes<32>,
}

const fn get_legacy_costs_from_batch_stats(stats: &BatchDataStats) -> u64 {
    let mut gas = 4 * (stats.length - stats.non_zeros) + 16 * stats.non_zeros;
    let keccak_words = words_for_bytes(stats.length);
    gas += 30 + (keccak_words * 6);
    gas += 2 * 20000;
    gas
}

const fn words_for_bytes(nbytes: u64) -> u64 {
    nbytes.div_ceil(32)
}

/// Decode a batch posting report message from the sequencer feed and build an
/// Arbitrum internal tx whose calldata matches Nitro's ArbosActs encoding.
pub fn decode_fields_sequencer(
    buf: &mut &[u8],
    chain_id: ChainId,
    arbos_version: u64,
    batch_data_stats: Option<BatchDataStats>,
    legacy_batch_gas: Option<u64>,
) -> alloy_rlp::Result<ArbInternalTx> {
    let batch_timestamp: U256 = decode(buf)?;
    let batch_poster: Address = decode(buf)?;
    let _data_hash: FixedBytes<32> = decode(buf)?;
    let batch_num: U256 = decode(buf)?;
    let l1_base_fee: U256 = decode(buf)?;
    let extra_gas = if buf.is_empty() { 0 } else { decode(buf)? };

    let legacy_gas = if let Some(ref stats) = batch_data_stats {
        let legacy_gas = get_legacy_costs_from_batch_stats(stats);
        if legacy_batch_gas.is_some() && legacy_batch_gas.unwrap() != legacy_gas {
            return Err(alloy_rlp::Error::Custom(
                "Legacy gas doesn't fit local compute.",
            ));
        }
        legacy_gas
    } else {
        legacy_batch_gas.ok_or(alloy_rlp::Error::Custom(
            "Legacy gas missing for batch posting report",
        ))?
    };

    let data = if arbos_version < 50_u64 {
        let batchgas = legacy_gas.saturating_add(extra_gas);
        construct_batchpostreport_data(
            batch_timestamp,
            batch_poster,
            batch_num
                .try_into()
                .map_err(|_| alloy_rlp::Error::Overflow)?,
            batchgas,
            l1_base_fee,
        )
    } else {
        let stats = batch_data_stats.ok_or(alloy_rlp::Error::Custom(
            "Batch data stats required for arbos version >= 50",
        ))?;
        construct_batchreportv2_data(
            batch_timestamp,
            batch_poster,
            batch_num
                .try_into()
                .map_err(|_| alloy_rlp::Error::Overflow)?,
            stats.length,
            stats.non_zeros,
            extra_gas,
            l1_base_fee,
        )
    };

    Ok(ArbInternalTx::new(chain_id, data))
}
