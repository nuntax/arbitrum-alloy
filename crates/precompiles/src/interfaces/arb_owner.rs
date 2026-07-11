alloy_core::sol! {
    /// ArbOwner: chain owner administration.
    ///
    /// All methods are restricted to chain owners. For read-only queries
    /// accessible by anyone, see `ArbOwnerPublic`.
    ///
    /// Nitro reference: `nitro/precompiles/ArbOwner.go`.
    #[sol(rpc)]
    interface ArbOwner {
        function addChainOwner(address newOwner) external;
        function removeChainOwner(address owner) external;

        function addNativeTokenOwner(address newOwner) external;
        function removeNativeTokenOwner(address owner) external;
        function setNativeTokenManagementFrom(uint64 timestamp) external;

        function setTransactionFilteringFrom(uint64 timestamp) external;
        function addTransactionFilterer(address filterer) external;
        function removeTransactionFilterer(address filterer) external;
        function setFilteredFundsRecipient(address newRecipient) external;

        function setNetworkFeeAccount(address newNetworkFeeAccount) external;
        function setInfraFeeAccount(address newInfraFeeAccount) external;

        function setL2BaseFee(uint256 priceInWei) external;
        function setMinimumL2BaseFee(uint256 priceInWei) external;
        function setSpeedLimit(uint64 limit) external;
        function setMaxTxGasLimit(uint64 limit) external;
        function setMaxBlockGasLimit(uint64 limit) external;
        function setL2GasPricingInertia(uint64 sec) external;
        function setL2GasBacklogTolerance(uint64 sec) external;
        function setGasBacklog(uint64 backlog) external;

        function setL1BaseFeeEstimateInertia(uint64 inertia) external;
        function setL1PricingEquilibrationUnits(uint256 equilibrationUnits) external;
        function setL1PricingInertia(uint64 inertia) external;
        function setL1PricingRewardRecipient(address recipient) external;
        function setL1PricingRewardRate(uint64 weiPerUnit) external;
        function setL1PricePerUnit(uint256 pricePerUnit) external;
        function setParentGasFloorPerToken(uint64 gasFloorPerToken) external;
        function setPerBatchGasCharge(int64 cost) external;
        function setAmortizedCostCapBips(uint64 cap) external;
        function releaseL1PricerSurplusFunds(uint256 maxWeiToRelease)
            external
            returns (uint256);

        function setBrotliCompressionLevel(uint64 level) external;
        function setCalldataPriceIncrease(bool enable) external;

        function scheduleArbOSUpgrade(uint64 newVersion, uint64 timestamp) external;
        function setChainConfig(bytes calldata serializedChainConfig) external;

        // Multi-constraint gas pricing (ArbOS 50+): install a set of gas constraints, each
        // {target, adjustmentWindow, backlog}. Replaces the legacy single speed-limit backlog model.
        function setGasPricingConstraints(uint64[3][] calldata constraints) external;

        // -- Stylus / WASM settings --
        function setInkPrice(uint32 inkPrice) external;
        function setWasmMaxStackDepth(uint32 depth) external;
        function setWasmFreePages(uint16 pages) external;
        function setWasmPageGas(uint16 gas) external;
        function setWasmPageLimit(uint16 limit) external;
        function setWasmMinInitGas(uint64 gas, uint64 cached) external;
        function setWasmInitCostScalar(uint64 percent) external;
        function setWasmExpiryDays(uint16 days) external;
        function setWasmKeepaliveDays(uint16 keepaliveDays) external;
        function setWasmBlockCacheSize(uint16 count) external;
        function setWasmMaxSize(uint32 maxWasmSize) external;
        function addWasmCacheManager(address manager) external;
        function removeWasmCacheManager(address manager) external;
        function setMaxStylusContractFragments(uint8 maxFragments) external;
    }
}
