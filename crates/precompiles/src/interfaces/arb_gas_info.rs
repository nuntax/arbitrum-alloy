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

    /// ArbGasInfo: gas pricing and L1/L2 fee information.
    ///
    /// Provides methods to query the current gas pricing model, L1 calldata
    /// costs, and various pricing parameters used by ArbOS.
    ///
    /// Nitro reference: `nitro/precompiles/ArbGasInfo.go`.
    #[sol(rpc)]
    interface ArbGasInfo {
        // @notice Get gas prices for a provided aggregator
        // @return return gas prices in wei
        //        (
        //            per L2 tx,
        //            per L1 calldata byte
        //            per storage allocation,
        //            per ArbGas base,
        //            per ArbGas congestion,
        //            per ArbGas total
        //        )
        function getPricesInWeiWithAggregator(
            address aggregator
        ) external view returns (uint256, uint256, uint256, uint256, uint256, uint256);

        // @notice Get gas prices. Uses the caller's preferred aggregator, or the default if the caller doesn't have a preferred one.
        // @return return gas prices in wei
        //        (
        //            per L2 tx,
        //            per L1 calldata byte
        //            per storage allocation,
        //            per ArbGas base,
        //            per ArbGas congestion,
        //            per ArbGas total
        //        )
        function getPricesInWei()
            external
            view
            returns (uint256, uint256, uint256, uint256, uint256, uint256);

        // @notice Get prices in ArbGas for the supplied aggregator
        // @return (per L2 tx, per L1 calldata byte, per storage allocation)
        function getPricesInArbGasWithAggregator(
            address aggregator
        ) external view returns (uint256, uint256, uint256);

        // @notice Get prices in ArbGas. Assumes the callers preferred validator, or the default if caller doesn't have a preferred one.
        // @return (per L2 tx, per L1 calldata byte, per storage allocation)
        function getPricesInArbGas() external view returns (uint256, uint256, uint256);

        // @notice Get the gas accounting parameters. `gasPoolMax` is always zero, as the exponential pricing model has no such notion.
        // @notice Starting from ArbOS version 50, `speedLimitPerSecond` corresponds to the target of the single-constraint pricing model.
        // @notice For multi-constraint pricing model, use `getGasPricingConstraints` to retrieve the constraint set.
        // @return (speedLimitPerSecond, gasPoolMax, maxBlockGasLimit)
        function getGasAccountingParams() external view returns (uint256, uint256, uint256);

        // @notice Get the maxTxGasLimit
        // @notice Available in ArbOS version 50 and above
        function getMaxTxGasLimit() external view returns (uint256);

        // @notice Get the minimum gas price needed for a tx to succeed
        function getMinimumGasPrice() external view returns (uint256);

        // @notice Get ArbOS's estimate of the L1 basefee in wei
        function getL1BaseFeeEstimate() external view returns (uint256);

        // @notice Get how slowly ArbOS updates its estimate of the L1 basefee
        function getL1BaseFeeEstimateInertia() external view returns (uint64);

        // @notice Get the L1 pricer reward rate, in wei per unit
        // @notice Available in ArbOS version 11 and above
        function getL1RewardRate() external view returns (uint64);

        // @notice Get the L1 pricer reward recipient
        // @notice Available in ArbOS version 11 and above
        function getL1RewardRecipient() external view returns (address);

        // @notice Deprecated -- Same as getL1BaseFeeEstimate()
        function getL1GasPriceEstimate() external view returns (uint256);

        // @notice Get L1 gas fees paid by the current transaction
        function getCurrentTxL1GasFees() external view returns (uint256);

        // @notice Get the backlogged amount of gas burnt in excess of the speed limit
        // @notice Starting from ArbOS version 50, returns the backlog value from the single-constraint pricing model.
        // @notice For multi-constraint pricing model, use `getGasPricingConstraints` to get backlog values for each configured constraint.
        function getGasBacklog() external view returns (uint64);

        // @notice Get how slowly ArbOS updates the L2 basefee in response to backlogged gas
        // @notice Starting from ArbOS version 50, returns the inertia value used by the single-constraint pricing model.
        // @notice For new integrations, prefer `getGasPricingConstraints`.
        function getPricingInertia() external view returns (uint64);

        // @notice Get the forgivable amount of backlogged gas ArbOS will ignore when raising the basefee
        // @notice Starting from ArbOS version 50, returns the backlog tolerance value used by the single-constraint pricing model.
        // @notice There is no tolerance for backlogged gas in the new multi-constraint pricing model.
        function getGasBacklogTolerance() external view returns (uint64);

        // @notice Returns the surplus of funds for L1 batch posting payments (may be negative).
        function getL1PricingSurplus() external view returns (int256);

        // @notice Returns the base charge (in L1 gas) attributed to each data batch in the calldata pricer
        function getPerBatchGasCharge() external view returns (int64);

        // @notice Returns the cost amortization cap in basis points
        function getAmortizedCostCapBips() external view returns (uint64);

        // @notice Returns the available funds from L1 fees
        // @notice Available in ArbOS version 10 and above
        function getL1FeesAvailable() external view returns (uint256);

        // @notice Returns the equilibration units parameter for L1 price adjustment algorithm
        // @notice Available in ArbOS version 20 and above
        function getL1PricingEquilibrationUnits() external view returns (uint256);

        // @notice Returns the last time the L1 calldata pricer was updated.
        // @notice Available in ArbOS version 20 and above
        function getLastL1PricingUpdateTime() external view returns (uint64);

        // @notice Returns the amount of L1 calldata payments due for rewards (per the L1 reward rate)
        // @notice Available in ArbOS version 20 and above
        function getL1PricingFundsDueForRewards() external view returns (uint256);

        // @notice Returns the amount of L1 calldata posted since the last update.
        // @notice Available in ArbOS version 20 and above
        function getL1PricingUnitsSinceUpdate() external view returns (uint64);

        // @notice Returns the L1 pricing surplus as of the last update (may be negative).
        // @notice Available in ArbOS version 20 and above
        function getLastL1PricingSurplus() external view returns (int256);

        // @notice Get the maximum block gas limit
        // @notice Available in ArbOS version 50 and above
        function getMaxBlockGasLimit() external view returns (uint64);

        // @notice Get the current gas pricing constraints used by the multi-constraint pricing model.
        // @notice Each constraint contains the following values:
        //         - gas_target_per_second: target gas usage per second for the constraint (uint64, gas/sec)
        //         - adjustment_window_seconds: time over which the price will rise by a factor of e if demand is 2x the target (uint64, seconds)
        //         - uint64 backlog: current backlog in gas units for this constraint (uint64, gas units)
        // @return constraints Array of triples (gas_target_per_second, adjustment_window_seconds, backlog)
        // @notice Available in ArbOS version 50 and above.
        function getGasPricingConstraints() external view returns (uint64[3][] memory constraints);

        // @notice Get the current multi-gas pricing constraints used by the multi-dimensional multi-constraint pricing model.
        // @notice Each constraint includes its target throughput, adjustment window, backlog, and weighted resource configuration.
        //
        // @notice The array contains one `ResourceConstraint` struct per active constraint.
        // @notice Each constraint provides the following fields:
        //         - `resources`: list of (resource kind, weight) pairs (see the ArbMultiGasConstraintsTypes library for definitions)
        //         - `adjustmentWindowSecs`: time window (seconds) over which the price rises by a factor of e if demand is 2x the target (uint64, seconds)
        //         - `targetPerSec`: target gas usage per second for this constraint (uint64, gas/sec)
        //         - `backlog`: current backlog value for this constraint (uint64, gas units)
        // @return constraints Array of `ResourceConstraint` structs representing the current multi-gas pricing configuration.
        // @notice Available in ArbOS version 60 and above.
        // @notice See the `ArbMultiGasConstraintsTypes` library for struct definitions and field details.
        function getMultiGasPricingConstraints()
            external
            view
            returns (ArbMultiGasConstraintsTypes.ResourceConstraint[] memory constraints);

        // @notice Get the current per-resource-kind base fees used by the multi-gas pricing model.
        // @dev The returned array is indexed by `uint256(ResourceKind)`.
        // @dev For example, `baseFees[uint256(ResourceKind.Computation)]` is the base fee
        // @dev for computation gas, and `baseFees[uint256(ResourceKind.StorageGrowth)]` is
        // @dev the base fee for storage growth. If a resource kind is unused, its entry will be zero.
        // @return baseFees An array of base fees in wei, one entry for each `ResourceKind` value.
        // @notice Available in ArbOS version 60 and above.
        function getMultiGasBaseFee() external view returns (uint256[] memory baseFees);
    }
}
