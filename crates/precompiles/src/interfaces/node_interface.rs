alloy_core::sol! {
    /// NodeInterface: virtual meta-contract for node-level queries.
    ///
    /// This is not a real on-chain contract, calls are intercepted and
    /// handled by the node software. Useful for gas estimation, batch
    /// queries, and L1 confirmation checks.
    ///
    /// Nitro reference: `nitro/nodeInterface/NodeInterface.go`.
    #[sol(rpc)]
    interface NodeInterface {
        /// Returns the Nitro genesis block number.
        function nitroGenesisBlock() external view returns (uint256);

        /// Returns the batch number containing the given L2 block.
        function findBatchContainingBlock(uint64 blockNum) external view returns (uint64);

        /// Returns the number of L1 confirmations for the given block hash.
        function getL1Confirmations(bytes32 blockHash) external view returns (uint64);

        /// Estimates the cost of submitting a retryable ticket.
        function estimateRetryableTicket(
            address sender,
            uint256 deposit,
            address to,
            uint256 l2CallValue,
            address excessFeeRefundAddress,
            address callValueRefundAddress,
            bytes calldata data
        ) external;

        /// Returns the L1 component of gas estimation for a transaction.
        /// Returns: (gasEstimateForL1, baseFee, l1BaseFeeEstimate).
        function gasEstimateL1Component(
            address to,
            bool contractCreation,
            bytes calldata data
        ) external payable returns (uint64, uint256, uint256);

        /// Returns components of gas estimation.
        /// Returns: (gasEstimate, gasEstimateForL1, baseFee, l1BaseFeeEstimate).
        function gasEstimateComponents(
            address to,
            bool contractCreation,
            bytes calldata data
        ) external payable returns (uint64, uint64, uint256, uint256);

        /// Returns the L2 block range corresponding to a given L1 block number.
        function l2BlockRangeForL1(uint64 blockNum)
            external
            view
            returns (uint64 firstBlock, uint64 lastBlock);

        /// Constructs an outbox proof for an L2→L1 send.
        function constructOutboxProof(uint64 size, uint64 leaf)
            external
            view
            returns (bytes32 send, bytes32 root, bytes32[] memory proof);

        /// Returns the L1 block number corresponding to a given message index.
        function blockL1Num(uint64 l2BlockNum) external view returns (uint64);
    }
}

#[cfg(test)]
mod tests {
    use alloy_provider::{Provider, ProviderBuilder};
    use arbitrum_alloy_network::Arbitrum;

    use crate::addresses::NODE_INTERFACE;
    use crate::interfaces::NodeInterface;

    #[tokio::test]
    async fn node_interface_live_view_calls() -> Result<(), Box<dyn std::error::Error>> {
        let rpc = match std::env::var("ARBITRUM_RPC") {
            Ok(v) if !v.trim().is_empty() => v,
            _ => {
                eprintln!("ARBITRUM_RPC not set, skipping");
                return Ok(());
            }
        };

        let provider = ProviderBuilder::<_, _, Arbitrum>::default()
            .connect(&rpc)
            .await?;
        let ni = NodeInterface::new(NODE_INTERFACE, &provider);

        let latest = provider.get_block_number().await?;
        let _genesis = ni.nitroGenesisBlock().call().await?;
        let l1_num = ni.blockL1Num(latest).call().await?;
        assert!(l1_num > 0);

        Ok(())
    }
}
