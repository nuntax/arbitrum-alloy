alloy_core::sol! {
    /// ArbDebug: debug-only precompile.
    ///
    /// Only available on debug/dev nodes. Not accessible in production.
    ///
    /// Nitro reference: `nitro/precompiles/ArbDebug.go`.
    #[sol(rpc)]
    interface ArbDebug {
        // @notice Caller becomes a chain owner
        function becomeChainOwner() external;

        // @notice Overwrite an existing contract's code
        function overwriteContractCode(
            address target,
            bytes calldata newCode
        ) external returns (bytes memory oldCode);

        // @notice Emit events with values based on the args provided
        function events(bool flag, bytes32 value) external payable returns (address, uint256);

        // @notice Tries (and fails) to emit logs in a view context
        function eventsView() external view;

        // Events that exist for testing log creation and pricing
        event Basic(bool flag, bytes32 indexed value);
        event Mixed(
            bool indexed flag, bool not, bytes32 indexed value, address conn, address indexed caller
        );
        event Store(
            bool indexed flag, address indexed field, uint24 number, bytes32 value, bytes store
        );

        function customRevert(
            uint64 number
        ) external pure;

        // @notice Available in ArbOS version 30 and above
        function panic() external;

        function legacyError() external pure;

        error Custom(uint64, string, bool);
        error Unused();
    }
}
