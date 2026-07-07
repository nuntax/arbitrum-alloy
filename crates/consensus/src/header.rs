use alloy_consensus::Header;
use alloy_primitives::{B256, Bytes};
use core::fmt;

// Nitro reference
// - core/types/arb_types.go:
//   - HeaderInfo.extra() writes SendRoot into Header.Extra (32 bytes)
//   - HeaderInfo.mixDigest() writes SendCount/L1BlockNumber/ArbOSFormatVersion into MixDigest[0..24]
//   - HeaderInfo.UpdateHeaderWithInfo() applies both fields to the header
//   - DeserializeHeaderExtraInformation() decodes from Header.Extra + Header.MixDigest
// - internal/ethapi/api.go:
//   - RPCMarshalHeader() returns extraData and mixHash as separate RPC fields
//   - fillArbitrumNitroHeaderInfo() derives sendRoot/sendCount/l1BlockNumber from those fields

/// Exact byte length of Arbitrum's `Header.extra_data`.
pub const ARB_HEADER_EXTRA_DATA_LEN: usize = 32;
/// Exact byte length of Arbitrum's `Header.mix_hash`.
pub const ARB_HEADER_MIX_HASH_LEN: usize = 32;
/// Number of bytes used inside `mix_hash` for Arbitrum header metadata.
pub const ARB_HEADER_MIX_HASH_INFO_LEN: usize = 8 + 8 + 8;
/// Legacy ArbOS version (`ArbosVersion_9`) under which tip collection was implicit in the version
/// itself rather than flagged in `mix_hash[25]` (Nitro `ArbosVersionCollectTipsOld`).
pub const ARBOS_VERSION_COLLECT_TIPS_OLD: u64 = 9;
/// Byte length for the fixture-packed representation (`extra_data || mix_hash[..24]`).
pub const ARB_HEADER_PACKED_LEN: usize = ARB_HEADER_EXTRA_DATA_LEN + ARB_HEADER_MIX_HASH_INFO_LEN;

/// Decoded Arbitrum information embedded in `Header.extra_data` and `Header.mix_hash`.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub struct ArbHeaderInfo {
    /// Merkle root of the delayed send queue.
    pub send_root: B256,
    /// Number of sends included so far.
    #[serde(with = "alloy_serde::quantity")]
    pub send_count: u64,
    /// L1 block number observed by ArbOS for this L2 block.
    #[serde(with = "alloy_serde::quantity")]
    pub l1_block_number: u64,
    /// ArbOS format version encoded into the header.
    #[serde(with = "alloy_serde::quantity")]
    pub arbos_format_version: u64,
    /// Whether this block collected sequencer tips (Nitro `HeaderInfo.CollectTips`). Encoded in
    /// `mix_hash[25]`, except under [`ARBOS_VERSION_COLLECT_TIPS_OLD`] where it is implicit in the
    /// version. Defaults to `false` so older serialized payloads (without the field) still decode.
    #[serde(default)]
    pub collect_tips: bool,
}

/// Error while decoding Arbitrum header info.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArbHeaderDecodeError {
    /// The `extra_data` byte length did not match Arbitrum's expected format.
    InvalidExtraDataLength {
        /// Number of bytes present in `extra_data`.
        got: usize,
    },
    /// The `mix_hash` byte length was invalid.
    InvalidMixHashLength {
        /// Number of bytes present in `mix_hash`.
        got: usize,
    },
    /// The header decoded successfully but `arbos_format_version` is 0,
    /// meaning this is not an Arbitrum header.
    NotArbitrum,
}

impl fmt::Display for ArbHeaderDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidExtraDataLength { got } => {
                write!(
                    f,
                    "invalid Arbitrum header extraData length: got {got}, expected {ARB_HEADER_EXTRA_DATA_LEN}"
                )
            }
            Self::InvalidMixHashLength { got } => {
                write!(
                    f,
                    "invalid Arbitrum header mixHash length: got {got}, expected {ARB_HEADER_MIX_HASH_LEN}"
                )
            }
            Self::NotArbitrum => {
                write!(
                    f,
                    "header has arbos_format_version 0, not an Arbitrum header"
                )
            }
        }
    }
}

impl core::error::Error for ArbHeaderDecodeError {}
impl ArbHeaderInfo {
    /// Returns true when this header info encodes an ArbOS format version.
    pub const fn is_arbitrum(&self) -> bool {
        self.arbos_format_version > 0
    }

    /// Decodes Arbitrum header info from consensus header fields.
    ///
    /// This mirrors Nitro's `DeserializeHeaderExtraInformation` behavior:
    /// `send_root <- extra_data`, and
    /// `send_count/l1_block_number/arbos_format_version <- mix_hash[0..24]`.
    pub fn decode_header_parts(
        extra_data: &[u8],
        mix_hash: &[u8],
    ) -> Result<Self, ArbHeaderDecodeError> {
        if extra_data.len() != ARB_HEADER_EXTRA_DATA_LEN {
            return Err(ArbHeaderDecodeError::InvalidExtraDataLength {
                got: extra_data.len(),
            });
        }
        if mix_hash.len() != ARB_HEADER_MIX_HASH_LEN {
            return Err(ArbHeaderDecodeError::InvalidMixHashLength {
                got: mix_hash.len(),
            });
        }

        let mut send_root = [0u8; 32];
        send_root.copy_from_slice(extra_data);

        let mut send_count_bytes = [0u8; 8];
        send_count_bytes.copy_from_slice(&mix_hash[..8]);

        let mut l1_block_number_bytes = [0u8; 8];
        l1_block_number_bytes.copy_from_slice(&mix_hash[8..16]);

        let mut arbos_format_version_bytes = [0u8; 8];
        arbos_format_version_bytes.copy_from_slice(&mix_hash[16..24]);
        let arbos_format_version = u64::from_be_bytes(arbos_format_version_bytes);

        // Mirror Nitro `DeserializeHeaderExtraInformation`: the legacy v9 always collected tips
        // (no flag byte); otherwise the flag lives in `mix_hash[25]`.
        let collect_tips = if arbos_format_version == ARBOS_VERSION_COLLECT_TIPS_OLD {
            true
        } else {
            mix_hash[25] & 0x1 == 1
        };

        Ok(Self {
            send_root: send_root.into(),
            send_count: u64::from_be_bytes(send_count_bytes),
            l1_block_number: u64::from_be_bytes(l1_block_number_bytes),
            arbos_format_version,
            collect_tips,
        })
    }

    /// Decodes Arbitrum header info from a header.
    pub fn decode_header(header: &Header) -> Result<Self, ArbHeaderDecodeError> {
        Self::decode_header_parts(header.extra_data.as_ref(), header.mix_hash.as_slice())
    }

    /// Encodes this info into Arbitrum header `extra_data` bytes.
    pub fn encode_extra_data(&self) -> Bytes {
        let mut out = [0u8; ARB_HEADER_EXTRA_DATA_LEN];
        out.copy_from_slice(self.send_root.as_slice());
        Bytes::copy_from_slice(&out)
    }

    /// Encodes this info into Arbitrum header `mix_hash` bytes.
    pub fn encode_mix_hash(&self) -> B256 {
        let mut out = [0u8; ARB_HEADER_MIX_HASH_LEN];
        out[..8].copy_from_slice(&self.send_count.to_be_bytes());
        out[8..16].copy_from_slice(&self.l1_block_number.to_be_bytes());
        out[16..24].copy_from_slice(&self.arbos_format_version.to_be_bytes());
        // Nitro `HeaderInfo.mixDigest`: byte 25 flags tip collection, except under the legacy
        // ArbOS v9 behaviour where collection is implicit in the version (no flag written).
        if self.collect_tips && self.arbos_format_version != ARBOS_VERSION_COLLECT_TIPS_OLD {
            out[25] = 1;
        }
        B256::from(out)
    }

    /// Updates an existing header with Arbitrum header info fields.
    ///
    /// This mirrors Nitro's `HeaderInfo.UpdateHeaderWithInfo`.
    pub fn update_header(&self, header: &mut Header) {
        header.extra_data = self.encode_extra_data();
        header.mix_hash = self.encode_mix_hash();
    }

    /// Decodes from a packed compatibility representation:
    /// `extra_data || mix_hash[..24]`.
    pub fn decode_packed(packed: &[u8]) -> Result<Self, ArbHeaderDecodeError> {
        if packed.len() != ARB_HEADER_PACKED_LEN {
            return Err(ArbHeaderDecodeError::InvalidExtraDataLength { got: packed.len() });
        }
        let mut mix_hash = [0u8; ARB_HEADER_MIX_HASH_LEN];
        mix_hash[..ARB_HEADER_MIX_HASH_INFO_LEN]
            .copy_from_slice(&packed[ARB_HEADER_EXTRA_DATA_LEN..ARB_HEADER_PACKED_LEN]);
        Self::decode_header_parts(&packed[..ARB_HEADER_EXTRA_DATA_LEN], &mix_hash)
    }

    /// Encodes to a packed compatibility representation:
    /// `extra_data || mix_hash[..24]`.
    pub fn encode_packed(&self) -> Bytes {
        let mut out = [0u8; ARB_HEADER_PACKED_LEN];
        out[..ARB_HEADER_EXTRA_DATA_LEN].copy_from_slice(self.send_root.as_slice());
        out[ARB_HEADER_EXTRA_DATA_LEN..ARB_HEADER_PACKED_LEN]
            .copy_from_slice(&self.encode_mix_hash().as_slice()[..ARB_HEADER_MIX_HASH_INFO_LEN]);
        Bytes::copy_from_slice(&out)
    }

    /// Returns the L1 block number from an Arbitrum header.
    ///
    /// Returns an error if the header does not decode or if the header
    /// has `arbos_format_version == 0` (not an Arbitrum header).
    pub fn parent_l1_block_number(header: &Header) -> Result<u64, ArbHeaderDecodeError> {
        let info = Self::decode_header(header)?;
        if !info.is_arbitrum() {
            return Err(ArbHeaderDecodeError::NotArbitrum);
        }
        Ok(info.l1_block_number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_info_roundtrip_header_fields() {
        let info = ArbHeaderInfo {
            send_root: B256::from([0x11; 32]),
            send_count: 42,
            l1_block_number: 99_001,
            arbos_format_version: 32,
            collect_tips: false,
        };

        let mut header = Header::default();
        info.update_header(&mut header);
        let decoded = ArbHeaderInfo::decode_header(&header).unwrap();

        assert_eq!(decoded, info);
    }

    #[test]
    fn collect_tips_flag_roundtrips_via_mix_hash_byte_25() {
        // collect_tips=true on a modern version sets mix_hash[25]=1 and round-trips.
        let info = ArbHeaderInfo {
            send_root: B256::from([0x22; 32]),
            send_count: 7,
            l1_block_number: 123,
            arbos_format_version: 40,
            collect_tips: true,
        };
        let mix = info.encode_mix_hash();
        assert_eq!(mix.as_slice()[25], 1, "mix_hash[25] must flag collect_tips");
        let mut header = Header::default();
        info.update_header(&mut header);
        assert_eq!(ArbHeaderInfo::decode_header(&header).unwrap(), info);

        // collect_tips=false leaves byte 25 zero.
        let mut info_off = info;
        info_off.collect_tips = false;
        assert_eq!(info_off.encode_mix_hash().as_slice()[25], 0);

        // Legacy v9: collection is implicit in the version, byte 25 is not written but decode
        // still reports collect_tips=true.
        let legacy = ArbHeaderInfo {
            arbos_format_version: ARBOS_VERSION_COLLECT_TIPS_OLD,
            collect_tips: true,
            ..info
        };
        assert_eq!(legacy.encode_mix_hash().as_slice()[25], 0, "v9 writes no flag byte");
        let mut legacy_header = Header::default();
        legacy.update_header(&mut legacy_header);
        assert!(
            ArbHeaderInfo::decode_header(&legacy_header).unwrap().collect_tips,
            "v9 must decode as collect_tips=true"
        );
    }

    #[test]
    fn decode_rejects_invalid_extra_data_length() {
        let mix_hash = [0u8; ARB_HEADER_MIX_HASH_LEN];
        let err = ArbHeaderInfo::decode_header_parts(&[0u8; 31], &mix_hash).unwrap_err();
        assert_eq!(
            err,
            ArbHeaderDecodeError::InvalidExtraDataLength { got: 31 }
        );
    }

    #[test]
    fn packed_roundtrip() {
        let info = ArbHeaderInfo {
            send_root: B256::from([0x33; 32]),
            send_count: 123,
            l1_block_number: 456,
            arbos_format_version: 50,
            collect_tips: false,
        };
        let packed = info.encode_packed();
        let decoded = ArbHeaderInfo::decode_packed(packed.as_ref()).unwrap();
        assert_eq!(decoded, info);
    }

    #[test]
    fn parent_l1_block_number_errors_for_legacy_headers() {
        let header = Header {
            number: 1234,
            extra_data: Bytes::new(),
            ..Default::default()
        };
        assert!(ArbHeaderInfo::parent_l1_block_number(&header).is_err());
    }

    #[test]
    fn parent_l1_block_number_errors_for_non_arbitrum() {
        let info = ArbHeaderInfo {
            send_root: B256::from([0xAA; 32]),
            send_count: 1,
            l1_block_number: 8_888_888,
            arbos_format_version: 0,
            collect_tips: false,
        };
        let header = Header {
            number: 7777,
            extra_data: info.encode_extra_data(),
            mix_hash: info.encode_mix_hash(),
            ..Default::default()
        };
        assert_eq!(
            ArbHeaderInfo::parent_l1_block_number(&header).unwrap_err(),
            ArbHeaderDecodeError::NotArbitrum
        );
    }

    #[test]
    fn parent_l1_block_number_uses_decoded_info() {
        let info = ArbHeaderInfo {
            send_root: B256::from([0xAA; 32]),
            send_count: 1,
            l1_block_number: 8_888_888,
            arbos_format_version: 50,
            collect_tips: false,
        };

        let header = Header {
            number: 7777,
            extra_data: info.encode_extra_data(),
            mix_hash: info.encode_mix_hash(),
            ..Default::default()
        };

        assert_eq!(
            ArbHeaderInfo::parent_l1_block_number(&header).unwrap(),
            8_888_888
        );
    }
}
