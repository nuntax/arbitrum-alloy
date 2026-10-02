alloy_core::sol! {
    /// ArbFunctionTable: function table for classic Arbitrum contracts.
    ///
    /// Largely vestigial post-Nitro; kept for backward compatibility.
    ///
    /// Nitro reference: `nitro/precompiles/ArbFunctionTable.go`.
    #[sol(rpc)]
    interface ArbFunctionTable {
        // @notice Reverts since the table is empty
        function upload(
            bytes calldata buf
        ) external;

        // @notice Returns the empty table's size, which is 0
        function size(
            address addr
        ) external view returns (uint256);

        // @notice No-op
        function get(address addr, uint256 index) external view returns (uint256, bool, uint256);
    }
}
