alloy_core::sol! {
    /// ArbWasmCache: Stylus WASM cache management.
    ///
    /// Nitro reference: `nitro/precompiles/ArbWasmCache.go`.
    #[sol(rpc)]
    interface ArbWasmCache {
        // @notice See if the user is a cache manager.
        function isCacheManager(
            address manager
        ) external view returns (bool);

        // @notice Retrieve all address managers.
        // @return managers the list of managers.
        function allCacheManagers() external view returns (address[] memory managers);

        // @dev Deprecated, replaced with cacheProgram
        // @notice Available in ArbOS version 30 only
        function cacheCodehash(
            bytes32 codehash
        ) external;

        // @notice Caches all programs with a codehash equal to the given address.
        // @notice Reverts if the programs have expired.
        // @notice Caller must be a cache manager or chain owner.
        // @notice If you're looking for how to bid for position, interact with the chain's cache manager contract.
        // @notice Available in ArbOS version 31 and above
        function cacheProgram(
            address addr
        ) external;

        // @notice Evicts all programs with the given codehash.
        // @notice Caller must be a cache manager or chain owner.
        function evictCodehash(
            bytes32 codehash
        ) external;

        // @notice Gets whether a program is cached. Note that the program may be expired.
        function codehashIsCached(
            bytes32 codehash
        ) external view returns (bool);

        event UpdateProgramCache(address indexed manager, bytes32 indexed codehash, bool cached);
    }
}
