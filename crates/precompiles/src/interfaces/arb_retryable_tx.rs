alloy_core::sol! {
    /// ArbRetryableTx: retryable ticket management.
    ///
    /// Provides methods to redeem, cancel, and inspect retryable tickets
    /// created via L1→L2 messaging.
    ///
    /// Nitro reference: `nitro/precompiles/ArbRetryableTx.go`.
    #[sol(rpc)]
    interface ArbRetryableTx {
        /*
         * @notice Schedule an attempt to redeem a redeemable tx, donating all of the call's gas to the redeem.
         * Revert if ticketId does not exist.
         * @param ticketId unique identifier of retryable message: keccak256(keccak256(ArbchainId, inbox-sequence-number), uint(0) )
         * @return txId that the redeem attempt will have
         */
        function redeem(
            bytes32 ticketId
        ) external returns (bytes32);

        /*
         * @notice Return the minimum lifetime of redeemable txn.
         * @return lifetime in seconds
         */
        function getLifetime() external view returns (uint256);

        /*
         * @notice Return the timestamp when ticketId will age out, reverting if it does not exist
         * @param ticketId unique ticket identifier
         * @return timestamp for ticket's deadline
         */
        function getTimeout(
            bytes32 ticketId
        ) external view returns (uint256);

        /*
         * @notice Adds one lifetime period to the life of ticketId.
         * Donate gas to pay for the lifetime extension.
         * If successful, emits LifetimeExtended event.
         * Revert if ticketId does not exist, or if the timeout of ticketId is already at least one lifetime period in the future.
         * @param ticketId unique ticket identifier
         * @return new timeout of ticketId
         */
        function keepalive(
            bytes32 ticketId
        ) external returns (uint256);

        /*
         * @notice Return the beneficiary of ticketId.
         * Revert if ticketId doesn't exist.
         * @param ticketId unique ticket identifier
         * @return address of beneficiary for ticket
         */
        function getBeneficiary(
            bytes32 ticketId
        ) external view returns (address);

        /*
         * @notice Cancel ticketId and refund its callvalue to its beneficiary.
         * Revert if ticketId doesn't exist, or if called by anyone other than ticketId's beneficiary.
         * @param ticketId unique ticket identifier
         */
        function cancel(
            bytes32 ticketId
        ) external;

        /*
         * @notice Gets the redeemer of the current retryable redeem attempt.
         * Returns the zero address if the current transaction is not a retryable redeem attempt.
         * If this is an auto-redeem, returns the fee refund address of the retryable.
         */
        function getCurrentRedeemer() external view returns (address);

        /*
         * @notice Do not call. This method represents a retryable submission to aid explorers.
         * Calling it will always revert.
         */
        function submitRetryable(
            bytes32 requestId,
            uint256 l1BaseFee,
            uint256 deposit,
            uint256 callvalue,
            uint256 gasFeeCap,
            uint64 gasLimit,
            uint256 maxSubmissionFee,
            address feeRefundAddress,
            address beneficiary,
            address retryTo,
            bytes calldata retryData
        ) external;

        event TicketCreated(bytes32 indexed ticketId);
        event LifetimeExtended(bytes32 indexed ticketId, uint256 newTimeout);
        event RedeemScheduled(
            bytes32 indexed ticketId,
            bytes32 indexed retryTxHash,
            uint64 indexed sequenceNum,
            uint64 donatedGas,
            address gasDonor,
            uint256 maxRefund,
            uint256 submissionFeeRefund
        );
        event Canceled(bytes32 indexed ticketId);

        // @dev DEPRECATED in favour of new RedeemScheduled event after the nitro upgrade
        event Redeemed(bytes32 indexed userTxHash);

        error NoTicketWithID();
        error NotCallable();
    }
}

#[cfg(test)]
mod tests {
    use alloy_core::sol_types::{SolCall, SolError};
    use alloy_primitives::B256;
    use alloy_provider::ProviderBuilder;
    use arbitrum_alloy_network::Arbitrum;

    use crate::addresses::ARB_RETRYABLE_TX;
    use crate::interfaces::ArbRetryableTx;

    #[test]
    fn submit_retryable_selectors_match_canonical_interface() {
        assert_eq!(
            ArbRetryableTx::submitRetryableCall::SELECTOR,
            [0xc9, 0xf9, 0x5d, 0x32]
        );
        assert_eq!(ArbRetryableTx::NotCallable::SELECTOR, [0x90, 0x28, 0x24, 0xc1]);
    }

    fn looks_like_expected_call_error(msg: &str) -> bool {
        msg.contains("server returned an error response")
            || msg.contains("error code")
            || msg.contains("execution reverted")
    }

    #[tokio::test]
    async fn arb_retryable_tx_live_view_calls() -> Result<(), Box<dyn std::error::Error>> {
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
        let retryable = ArbRetryableTx::new(ARB_RETRYABLE_TX, &provider);

        let lifetime = retryable.getLifetime().call().await?;
        assert!(lifetime > 0);

        let _current_redeemer = retryable.getCurrentRedeemer().call().await?;

        if let Err(e) = retryable.getBeneficiary(B256::ZERO).call().await {
            assert!(looks_like_expected_call_error(&e.to_string()), "{e}");
        }
        if let Err(e) = retryable.getTimeout(B256::ZERO).call().await {
            assert!(looks_like_expected_call_error(&e.to_string()), "{e}");
        }

        Ok(())
    }
}
