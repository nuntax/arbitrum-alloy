alloy_core::sol! {
    // @title ArbMultiGasConstraintsTypes
    // @notice Type definitions for configuring and inspecting multi-dimensional gas pricing constraints.
    // @dev These structures are shared by the `ArbOwner` and `ArbGasInfo` precompiles.
    library ArbMultiGasConstraintsTypes {
        // @notice Enumerates the distinct resource kinds used for multi-dimensional gas accounting.
        // @dev The numeric values of this enum must remain consistent with the corresponding
        // @dev The values defined here must stay synchronized with `go-ethereum/arbitrum/multigas/resources.go`.
        enum ResourceKind {
            Unknown,
            Computation,
            HistoryGrowth,
            StorageAccessRead,
            StorageAccessWrite,
            StorageGrowth,
            SingleDim,
            L2Calldata,
            WasmComputation
        }

        // @notice Defines the relative contribution of a specific resource type within a constraint.
        // @param resource  The resource kind being weighted.
        // @param weight    The proportional weight that determines this resource kind’s influence on backlog growth.
        struct WeightedResource {
            ResourceKind resource;
            uint64 weight;
        }

        // @notice Describes a single pricing constraint that applies to one or more weighted resources.
        // @param resources             The list of weighted resources that participate in this constraint.
        // @param adjustmentWindowSecs  Time window (seconds) over which the price rises by a factor of e if demand is 2x the target (uint64, seconds)
        // @param targetPerSec          The target gas usage per second for this constraint (uint64, gas/sec).
        // @param backlog               The current or initial backlogfor this constraint (uint64, gas units).
        struct ResourceConstraint {
            WeightedResource[] resources;
            uint32 adjustmentWindowSecs;
            uint64 targetPerSec;
            uint64 backlog;
        }
    }

    /// ArbOwner: chain owner administration.
    ///
    /// All methods are restricted to chain owners. For read-only queries
    /// accessible by anyone, see `ArbOwnerPublic`.
    ///
    /// Nitro reference: `nitro/precompiles/ArbOwner.go`.
    #[sol(rpc)]
    interface ArbOwner {
        // @notice Emitted when an address is added as a transaction filterer.
        event TransactionFiltererAdded(address indexed filterer);

        // @notice Emitted when an address is removed as a transaction filterer.
        event TransactionFiltererRemoved(address indexed filterer);

        // @notice Emitted when the filtered funds recipient address is changed.
        // @notice Available in ArbOS version 60 and above
        event FilteredFundsRecipientSet(address indexed newRecipient);

        // @notice Emitted when an address is added as a chain owner.
        // @notice Available in ArbOS version 60 and above
        event ChainOwnerAdded(address indexed owner);

        // @notice Emitted when an address is removed as a chain owner.
        // @notice Available in ArbOS version 60 and above
        event ChainOwnerRemoved(address indexed owner);

        // @notice Emitted when an address is added as a native token owner.
        // @notice Available in ArbOS version 60 and above
        event NativeTokenOwnerAdded(address indexed owner);

        // @notice Emitted when an address is removed as a native token owner.
        // @notice Available in ArbOS version 60 and above
        event NativeTokenOwnerRemoved(address indexed owner);

        // @notice Add account as a chain owner
        function addChainOwner(
            address newOwner
        ) external;

        // @notice Remove account from the list of chain owners
        function removeChainOwner(
            address ownerToRemove
        ) external;

        // @notice See if the user is a chain owner
        function isChainOwner(
            address addr
        ) external view returns (bool);

        // @notice Retrieves the list of chain owners
        function getAllChainOwners() external view returns (address[] memory);

        // @notice Sets the NativeTokenManagementFrom time
        // @notice Available in ArbOS version 41 and above
        function setNativeTokenManagementFrom(
            uint64 timestamp
        ) external;

        // @notice Sets the TransactionFilteringFrom time
        // @notice Available in ArbOS version 60 and above
        function setTransactionFilteringFrom(
            uint64 timestamp
        ) external;

        // @notice Add account as a native token owner
        // @notice Available in ArbOS version 41 and above
        function addNativeTokenOwner(
            address newOwner
        ) external;

        // @notice Remove account from the list of native token owners
        // @notice Available in ArbOS version 41 and above
        function removeNativeTokenOwner(
            address ownerToRemove
        ) external;

        // @notice See if the user is a native token owner
        // @notice Available in ArbOS version 41 and above
        function isNativeTokenOwner(
            address addr
        ) external view returns (bool);

        // @notice Retrieves the list of native token owners
        // @notice Available in ArbOS version 41 and above
        function getAllNativeTokenOwners() external view returns (address[] memory);

        // @notice Add account as a transaction filterer (authorized to use ArbFilteredTransactionsManager)
        // @notice Available in ArbOS version 60 and above
        function addTransactionFilterer(
            address filterer
        ) external;

        // @notice Remove account from the list of transaction filterers
        // @notice Available in ArbOS version 60 and above
        function removeTransactionFilterer(
            address filterer
        ) external;

        // @notice See if the user is a transaction filterer
        // @notice Available in ArbOS version 60 and above
        function isTransactionFilterer(
            address filterer
        ) external view returns (bool);

        // @notice Retrieves the list of transaction filterers
        // @notice Available in ArbOS version 60 and above
        function getAllTransactionFilterers() external view returns (address[] memory);

        // @notice Set the address that receives funds redirected from filtered transactions.
        // @notice Set to address(0) to use the networkFeeAccount as fallback.
        // @notice Available in ArbOS version 60 and above
        function setFilteredFundsRecipient(
            address newRecipient
        ) external;

        // @notice Get the address that receives funds redirected from filtered transactions.
        // @notice Returns address(0) if not explicitly set (networkFeeAccount is used as fallback at runtime).
        // @notice Available in ArbOS version 60 and above
        function getFilteredFundsRecipient() external view returns (address);

        // @notice Set how slowly ArbOS updates its estimate of the L1 basefee
        function setL1BaseFeeEstimateInertia(
            uint64 inertia
        ) external;

        // @notice Set the L2 basefee directly, bypassing the pool calculus
        function setL2BaseFee(
            uint256 priceInWei
        ) external;

        // @notice Set the minimum basefee needed for a transaction to succeed
        function setMinimumL2BaseFee(
            uint256 priceInWei
        ) external;

        // @notice Set the computational speed limit for the chain
        // @notice Starting from ArbOS version 50, continues to set the legacy single-constraint speed limit value.
        // @notice It does not modify the multi-constraint pricing model configuration. To configure multiple constraints, use `setGasPricingConstraints`.
        function setSpeedLimit(
            uint64 limit
        ) external;

        // @notice Set the maximum size a tx can be
        function setMaxTxGasLimit(
            uint64 limit
        ) external;

        // @notice Set the maximum size a block can be
        // @notice Available in ArbOS version 50 and above
        function setMaxBlockGasLimit(
            uint64 limit
        ) external;

        // @notice Set the L2 gas pricing inertia
        // @notice Starting from ArbOS version 50, continues to set the single-constraint pricing inertia value.
        // @notice It does not modify the multi-constraint pricing model configuration. To configure multiple constraints, use `setGasPricingConstraints`.
        function setL2GasPricingInertia(
            uint64 sec
        ) external;

        // @notice Set the L2 gas backlog tolerance
        // @notice Starting from ArbOS version 50, continues to set the single-constraint backlog tolerance value.
        // @notice It does not modify the multi-constraint pricing model configuration. There is no tolerance for backlogged gas in the new multi-constraint pricing model.
        function setL2GasBacklogTolerance(
            uint64 sec
        ) external;

        // @notice Get the network fee collector
        function getNetworkFeeAccount() external view returns (address);

        // @notice Get the infrastructure fee collector
        // @notice Available in ArbOS version 5 and above
        function getInfraFeeAccount() external view returns (address);

        // @notice Set the network fee collector
        function setNetworkFeeAccount(
            address newNetworkFeeAccount
        ) external;

        // @notice Set the infrastructure fee collector
        // @notice Available in ArbOS version 5 and above
        function setInfraFeeAccount(
            address newInfraFeeAccount
        ) external;

        // @notice Upgrades ArbOS to the requested version at the requested timestamp
        function scheduleArbOSUpgrade(uint64 newVersion, uint64 timestamp) external;

        // @notice Sets equilibration units parameter for L1 price adjustment algorithm
        function setL1PricingEquilibrationUnits(
            uint256 equilibrationUnits
        ) external;

        // @notice Sets inertia parameter for L1 price adjustment algorithm
        function setL1PricingInertia(
            uint64 inertia
        ) external;

        // @notice Sets reward recipient address for L1 price adjustment algorithm
        function setL1PricingRewardRecipient(
            address recipient
        ) external;

        // @notice Sets reward amount for L1 price adjustment algorithm, in wei per unit
        function setL1PricingRewardRate(
            uint64 weiPerUnit
        ) external;

        // @notice Set how much ArbOS charges per L1 gas spent on transaction data.
        function setL1PricePerUnit(
            uint256 pricePerUnit
        ) external;

        // @notice Set how much L1 charges per non-zero byte of calldata
        // @notice Available in ArbOS version 50 and above
        function setParentGasFloorPerToken(
            uint64 floorPerToken
        ) external;

        // @notice Sets the base charge (in L1 gas) attributed to each data batch in the calldata pricer
        function setPerBatchGasCharge(
            int64 cost
        ) external;

        // @notice Sets the Brotli compression level used for fast compression
        // @notice Available in ArbOS version 20 and above
        function setBrotliCompressionLevel(
            uint64 level
        ) external;

        // @notice Sets the cost amortization cap in basis points
        function setAmortizedCostCapBips(
            uint64 cap
        ) external;

        // @notice Releases surplus funds from L1PricerFundsPoolAddress for use
        // @notice Available in ArbOS version 10 and above
        function releaseL1PricerSurplusFunds(
            uint256 maxWeiToRelease
        ) external returns (uint256);

        // @notice Sets the amount of ink 1 gas buys
        // @notice Available in ArbOS version 30 and above
        // @param price the conversion rate (must fit in a uint24)
        function setInkPrice(
            uint32 price
        ) external;

        // @notice Sets the maximum depth (in wasm words) a wasm stack may grow
        // @notice Available in ArbOS version 30 and above
        function setWasmMaxStackDepth(
            uint32 depth
        ) external;

        // @notice Sets the number of free wasm pages a tx gets
        // @notice Available in ArbOS version 30 and above
        function setWasmFreePages(
            uint16 pages
        ) external;

        // @notice Sets the base cost of each additional wasm page
        // @notice Available in ArbOS version 30 and above
        function setWasmPageGas(
            uint16 gas
        ) external;

        // @notice Sets the maximum number of pages a wasm may allocate
        // @notice Available in ArbOS version 30 and above
        function setWasmPageLimit(
            uint16 limit
        ) external;

        // @notice Sets the maximum size of the uncompressed wasm code in bytes
        // @notice Available in ArbOS version 30 and above
        function setWasmMaxSize(
            uint32 size
        ) external;

        // @notice Sets the minimum costs to invoke a program
        // @notice Available in ArbOS version 30 and above
        // @param gas amount of gas paid in increments of 256 when not the program is not cached
        // @param cached amount of gas paid in increments of 64 when the program is cached
        function setWasmMinInitGas(uint8 gas, uint16 cached) external;

        // @notice Sets the linear adjustment made to program init costs.
        // @notice Available in ArbOS version 30 and above
        // @param percent the adjustment (100% = no adjustment).
        function setWasmInitCostScalar(
            uint64 percent
        ) external;

        // @notice Sets the number of days after which programs deactivate
        // @notice Available in ArbOS version 30 and above
        function setWasmExpiryDays(
            uint16 _days
        ) external;

        // @notice Sets the age a program must be to perform a keepalive
        // @notice Available in ArbOS version 30 and above
        function setWasmKeepaliveDays(
            uint16 _days
        ) external;

        // @notice Sets the number of extra programs ArbOS caches during a given block
        // @notice Available in ArbOS version 30 and above
        function setWasmBlockCacheSize(
            uint16 count
        ) external;

        // @notice Adds account as a wasm cache manager
        // @notice Available in ArbOS version 30 and above
        function addWasmCacheManager(
            address manager
        ) external;

        // @notice Removes account from the list of wasm cache managers
        // @notice Available in ArbOS version 30 and above
        function removeWasmCacheManager(
            address manager
        ) external;

        // @notice Sets serialized chain config in ArbOS state
        // @notice Available in ArbOS version 11 and above
        function setChainConfig(
            string calldata chainConfig
        ) external;

        // @notice Sets the increased calldata price feature on or off (EIP-7623)
        // @notice Available in ArbOS version 40 and above with default as false
        function setCalldataPriceIncrease(
            bool enable
        ) external;

        // @notice Set the backlogged amount of gas burnt used by the single-constraint pricing model only.
        // @notice To configure backlogs for the multi-constraint pricing model, use `setGasPricingConstraints()`.
        // @param backlog The backlog value in gas units to assign to the single-constraint pricing model.
        function setGasBacklog(
            uint64 backlog
        ) external;

        // @notice Sets the list of gas pricing constraints for the multi-constraint pricing model.
        // @notice Replaces the existing constraints configuration and sets each constraint's starting backlog value.
        // @notice All existing backlogs are replaced by the provided values.
        // @notice Any changes to gas targets, inertia, or starting backlogs may cause immediate price fluctuations.
        // @notice Operators are fully responsible for the resulting behavior and should adjust parameters carefully.
        // @notice Use `ArbGasInfo.getGasPricingConstraints()` to retrieve the current configuration.
        // @notice Model selection:
        // @notice - If one or more constraints are provided, the chain switches to the multi-constraint pricing model
        // @notice   and uses exactly the provided parameters.
        // @notice - If zero constraints are provided, the chain uses the single-constraint pricing model.
        // @notice   In that case, the single-constraint backlog can be set via `setBacklog`, which you may derive from multi-constraint
        // @notice   parameters if desired.
        // @notice Available in ArbOS version 50 and above.
        // @param constraints Array of triples (gas_target_per_second, adjustment_window_seconds, starting_backlog_value)
        //        - gas_target_per_second: target gas usage per second for the constraint (uint64, gas/sec)
        //        - adjustment_window_seconds: time over which the price will rise by a factor of e if demand is 2x the target (uint64, seconds)
        //        - starting_backlog_value: initial backlog for this constraint (uint64, gas units)
        function setGasPricingConstraints(
            uint64[3][] calldata constraints
        ) external;

        // @notice Sets the list of multi-gas pricing constraints for the multi-dimensional multi-constraint pricing model.
        // @notice Replaces the existing configuration and initializes each constraint’s starting backlog value.
        // @notice Any previous multi-gas constraints are discarded.
        //
        // @notice This method defines both the target throughput and relative weights
        // @notice of each resource kind participating in multi-dimensional gas pricing.
        // @notice Each constraint may specify multiple resources with different weight multipliers.
        // @notice The final base fee is determined by the most congested resource dimension.
        //
        // @notice Model selection:
        // @notice - If one or more multi-gas constraints are provided, ArbOS switches to the multi-dimensional
        // @notice   pricing model and uses exactly the provided parameters.
        // @notice - If zero constraints are provided, the chain reverts to the previous pricing model
        // @notice   (either the single-constraint or single-dimensional multi-constraint configuration).
        //
        // @notice Available in ArbOS version 60 and above.
        // @param constraints  Array of `ResourceConstraint` structs, each containing:
        //        - `resources`: list of (resource kind, weight) pairs (see the ArbMultiGasConstraintsTypes library for definitions)
        //        - `adjustmentWindowSecs`: time window (seconds) over which the price rises by a factor of e if demand is 2x the target (uint64, seconds)
        //        - `targetPerSec`: target gas usage per second for this constraint (uint64, gas/sec)
        //        - `backlog`: initial backlog value for this constraint  (uint64, gas units)
        function setMultiGasPricingConstraints(
            ArbMultiGasConstraintsTypes.ResourceConstraint[] calldata constraints
        ) external;

        // @notice Enables or disables tip collection.
        // @notice When enabled, transaction tips are sent to the network fee account.
        // @notice Available in ArbOS version 60 and above
        function setCollectTips(
            bool collectTips
        ) external;

        // @notice Sets the max amount of stylus contract fragments that can be used to deploy a stylus contract
        // @notice Available in ArbOS version 60 and above
        function setMaxStylusContractFragments(
            uint8 maxFragments
        ) external;

        // @notice Sets the constant gas charge applied before each stylus contract activation.
        // @notice Defaults to zero.
        // @notice Available in ArbOS version 60 and above
        function setWasmActivationGas(
            uint64 gas
        ) external;

        // Emitted when a successful call is made to this precompile
        event OwnerActs(bytes4 indexed method, address indexed owner, bytes data);
    }
}
