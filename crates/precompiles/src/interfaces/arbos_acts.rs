alloy_core::sol! {
    /// ArbosActs: internal ArbOS actor.
    ///
    /// These methods are only callable by ArbOS itself (address 0xA4B05).
    /// They are used internally by the sequencer to mark block boundaries
    /// and report batch posting costs. Exposed here for calldata decoding
    /// of `ArbInternalTx` payloads.
    ///
    /// Nitro reference: `nitro/precompiles/ArbosActs.go`.
    #[sol(rpc)]
    interface ArbosActs {
        /*
         * @notice ArbOS "calls" this when starting a block
         * @param l1BaseFee the L1 BaseFee
         * @param l1BlockNumber the L1 block number
         * @param timePassed number of seconds since the last block
         */
        function startBlock(
            uint256 l1BaseFee,
            uint64 l1BlockNumber,
            uint64 l2BlockNumber,
            uint64 timePassed
        ) external;

        function batchPostingReport(
            uint256 batchTimestamp,
            address batchPosterAddress,
            uint64 batchNumber,
            uint64 batchDataGas,
            uint256 l1BaseFeeWei
        ) external;

        function batchPostingReportV2(
            uint256 batchTimestamp,
            address batchPosterAddress,
            uint64 batchNumber,
            uint64 batchCalldataLength,
            uint64 batchCalldataNonZeros,
            uint64 batchExtraGas,
            uint256 l1BaseFeeWei
        ) external;

        error CallerNotArbOS();
    }
}
