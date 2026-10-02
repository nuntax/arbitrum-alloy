alloy_core::sol! {
    /// ArbOwnerPublic: read-only chain owner queries (callable by anyone).
    ///
    /// Nitro reference: `nitro/precompiles/ArbOwnerPublic.go`.
    #[sol(rpc)]
    interface ArbOwnerPublic {
        // @notice See if the user is a chain owner
        function isChainOwner(
            address addr
        ) external view returns (bool);

        // @notice Rectify the list of chain owners
        // @notice If successful, emits ChainOwnerRectified event
        // @notice Available in ArbOS version 11 and above
        function rectifyChainOwner(
            address ownerToRectify
        ) external;

        // @notice Retrieves the list of chain owners
        function getAllChainOwners() external view returns (address[] memory);

        // @notice Retrieves the timestamp from when the native token management
        // modifications are enabled.
        // @notice Available in ArbOS version 50 and above
        function getNativeTokenManagementFrom() external view returns (uint64);

        // @notice See if the user is a native token owner
        // @notice Available in ArbOS version 41 and above
        function isNativeTokenOwner(
            address addr
        ) external view returns (bool);

        // @notice Retrieves the list of native token owners
        // @notice Available in ArbOS version 41 and above
        function getAllNativeTokenOwners() external view returns (address[] memory);

        // @notice Retrieves the timestamp from when the transaction filtering
        // modifications are enabled.
        // @notice Available in ArbOS version 60 and above
        function getTransactionFilteringFrom() external view returns (uint64);

        // @notice See if the user is a transaction filterer
        // @notice Available in ArbOS version 60 and above
        function isTransactionFilterer(
            address filterer
        ) external view returns (bool);

        // @notice Retrieves the list of transaction filterers
        // @notice Available in ArbOS version 60 and above
        function getAllTransactionFilterers() external view returns (address[] memory);

        // @notice Get the address that receives funds redirected from filtered transactions.
        // @notice Returns address(0) if not explicitly set (networkFeeAccount is used as fallback at runtime).
        // @notice Available in ArbOS version 60 and above
        function getFilteredFundsRecipient() external view returns (address);

        // @notice Gets the network fee collector
        function getNetworkFeeAccount() external view returns (address);

        // @notice Get the infrastructure fee collector
        function getInfraFeeAccount() external view returns (address);

        // @notice Get the Brotli compression level used for fast compression
        // @notice Available in ArbOS version 20 and above
        function getBrotliCompressionLevel() external view returns (uint64);

        // @notice Get how much L1 charges per non-zero byte of calldata
        // @notice Available in ArbOS version 50 and above
        function getParentGasFloorPerToken() external view returns (uint64);

        // @notice Get the next scheduled ArbOS version upgrade and its activation timestamp.
        // @notice Returns (0, 0) if no ArbOS upgrade is scheduled.
        // @notice Available in ArbOS version 20 and above
        function getScheduledUpgrade()
            external
            view
            returns (uint64 arbosVersion, uint64 scheduledForTimestamp);

        // @notice Checks if the increased calldata price feature (EIP-7623) is enabled
        // @notice Available in ArbOS version 40 with default as false
        function isCalldataPriceIncreaseEnabled() external view returns (bool);

        // @notice Returns whether tip collection is enabled.
        // @notice Available in ArbOS version 60 and above
        function getCollectTips() external view returns (bool);

        // @notice Get the max amount of stylus contract fragments that can be used to deploy a stylus contract
        // @notice Available in ArbOS version 60 and above
        function getMaxStylusContractFragments() external view returns (uint8);

        event ChainOwnerRectified(address rectifiedOwner);
    }
}
