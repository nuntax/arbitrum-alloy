alloy_core::sol! {
    /// ArbStatistics: chain statistics.
    ///
    /// Returns mostly legacy pre-Nitro statistics. The first return value
    /// (block number) is the only meaningful field post-Nitro.
    ///
    /// Nitro reference: `nitro/precompiles/ArbStatistics.go`.
    #[sol(rpc)]
    interface ArbStatistics {
        // @notice Get Arbitrum block number and other statistics as they were right before the Nitro upgrade.
        // @return (
        //      Number of accounts,
        //      Total storage allocated (includes storage that was later deallocated),
        //      Total ArbGas used,
        //      Number of transaction receipt issued,
        //      Number of contracts created,
        //    )
        function getStats()
            external
            view
            returns (uint256, uint256, uint256, uint256, uint256, uint256);
    }
}
