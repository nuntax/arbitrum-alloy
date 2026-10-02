alloy_core::sol! {
    /// ArbFilteredTransactionsManager: filtered-transaction list, managed by callers the chain owner authorises (ArbOS 60).
    ///
    /// Nitro reference: `nitro/precompiles/ArbFilteredTransactionsManager.go`.
    #[sol(rpc)]
    interface ArbFilteredTransactionsManager {
        // @notice Emitted when a transaction hash is added to the filtered transactions list.
        event FilteredTransactionAdded(bytes32 indexed txHash);

        // @notice Emitted when a transaction hash is removed from the filtered transactions list.
        event FilteredTransactionDeleted(bytes32 indexed txHash);

        // @notice Marks the given transaction hash as filtered
        // @param txHash The transaction hash to filter
        function addFilteredTransaction(
            bytes32 txHash
        ) external;

        // @notice Removes filtering mark for the given transaction hash
        // @param txHash The transaction hash to unfilter
        function deleteFilteredTransaction(
            bytes32 txHash
        ) external;

        // @notice Checks whether the given transaction hash is filtered
        // @param txHash The transaction hash to check
        // @return True if filtered, false otherwise
        function isTransactionFiltered(
            bytes32 txHash
        ) external view returns (bool);
    }
}
