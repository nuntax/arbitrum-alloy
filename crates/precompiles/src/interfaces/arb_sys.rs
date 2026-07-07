alloy_core::sol! {
    /// ArbSys: core L2 system operations.
    ///
    /// Provides block number, block hash, chain ID, L2→L1 messaging,
    /// and address aliasing utilities.
    ///
    /// Nitro reference: `nitro/precompiles/ArbSys.go`.
    #[sol(rpc)]
    interface ArbSys {
        /// Returns the current L2 block number.
        function arbBlockNumber() external view returns (uint256);

        /// Returns the L2 block hash for the given block number.
        /// Only available for the last 256 blocks.
        function arbBlockHash(uint256 arbBlockNum) external view returns (bytes32);

        /// Returns the rollup chain ID.
        function arbChainID() external view returns (uint256);

        /// Returns the current ArbOS version number.
        function arbOSVersion() external view returns (uint256);

        /// Returns 0 (Nitro has no storage gas concept; kept for Classic compat).
        function getStorageGasAvailable() external view returns (uint256);

        /// Returns true if the current call is top-level (not an internal call).
        function isTopLevelCall() external view returns (bool);

        /// Maps an L1 contract address to its L2 alias.
        function mapL1SenderContractAddressToL2Alias(address sender, address unused)
            external
            view
            returns (address);

        /// Returns true if the caller's address was aliased from L1.
        function wasMyCallersAddressAliased() external view returns (bool);

        /// Returns the caller's original (un-aliased) L1 address.
        function myCallersAddressWithoutAliasing() external view returns (address);

        /// Returns the send Merkle tree state: (size, root, partials).
        function sendMerkleTreeState()
            external
            view
            returns (uint256 size, bytes32 root, bytes32[] memory partials);

        /// Sends a transaction to L1. Returns the L2→L1 message sequence number.
        function sendTxToL1(address destination, bytes calldata data)
            external
            payable
            returns (uint256);

        /// Withdraws ETH to an L1 destination. Returns the L2→L1 message sequence number.
        function withdrawEth(address destination)
            external
            payable
            returns (uint256);

        /// Emitted when a transaction is sent from L2 to L1.
        event L2ToL1Tx(
            address caller,
            address indexed destination,
            uint256 indexed hash,
            uint256 indexed position,
            uint256 arbBlockNum,
            uint256 ethBlockNum,
            uint256 timestamp,
            uint256 callvalue,
            bytes data
        );

        /// Emitted when a new send Merkle tree update is published.
        event SendMerkleUpdate(
            uint256 indexed reserved,
            bytes32 blockHash,
            uint256 sendCount
        );
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::address;
    use alloy_provider::{Provider, ProviderBuilder};
    use arbitrum_alloy_network::Arbitrum;

    use crate::addresses::ARB_SYS;
    use crate::interfaces::ArbSys;

    #[tokio::test]
    async fn arb_sys_live_view_calls() -> Result<(), Box<dyn std::error::Error>> {
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

        let arb_sys = ArbSys::new(ARB_SYS, &provider);

        let chain_id = arb_sys.arbChainID().call().await?;
        assert_eq!(chain_id, provider.get_chain_id().await?);

        let block_num = arb_sys.arbBlockNumber().call().await?;
        assert!(block_num > 0);

        let mapped = arb_sys
            .mapL1SenderContractAddressToL2Alias(
                address!("0x1000000000000000000000000000000000000000"),
                address!("0x0000000000000000000000000000000000000000"),
            )
            .call()
            .await?;
        assert_ne!(
            mapped,
            address!("0x0000000000000000000000000000000000000000")
        );

        Ok(())
    }
}
