alloy_core::sol! {
    /// ArbInfo: account balance and code queries.
    ///
    /// Nitro reference: `nitro/precompiles/ArbInfo.go`.
    #[sol(rpc)]
    interface ArbInfo {
        // @notice Retrieves an account's balance
        function getBalance(
            address account
        ) external view returns (uint256);

        // @notice Retrieves a contract's deployed code
        function getCode(
            address account
        ) external view returns (bytes memory);
    }
}
