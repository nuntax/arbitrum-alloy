use alloc::boxed::Box;
use alloy_network::Network;
use alloy_provider::Provider;
use alloy_transport::TransportResult;
use arbitrum_alloy_network::Arbitrum;
use arbitrum_alloy_rpc_types::JsonExpressLaneSubmission;

/// Provider extension trait for the `timeboost_*` JSON-RPC namespace.
///
/// Nitro reference: `execution/gethexec/api.go` -> `ArbTimeboostAPI`.
#[cfg_attr(target_family = "wasm", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
pub trait TimeboostProviderExt<N: Network = Arbitrum>: Send + Sync {
    /// Submit an express-lane transaction for the current auction round.
    ///
    /// Nitro reference: `execution/gethexec/api.go` -> `SendExpressLaneTransaction`.
    async fn timeboost_send_express_lane_transaction(
        &self,
        submission: JsonExpressLaneSubmission,
    ) -> TransportResult<()>;
}

#[cfg_attr(target_family = "wasm", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl<N, P> TimeboostProviderExt<N> for P
where
    N: Network,
    P: Provider<N>,
{
    async fn timeboost_send_express_lane_transaction(
        &self,
        submission: JsonExpressLaneSubmission,
    ) -> TransportResult<()> {
        self.client()
            .request("timeboost_sendExpressLaneTransaction", (submission,))
            .await
    }
}

#[cfg(test)]
mod tests {
    use alloy_provider::{Provider, ProviderBuilder};
    use alloy_transport::mock::Asserter;
    use arbitrum_alloy_network::Arbitrum;

    use super::TimeboostProviderExt;

    fn looks_like_rpc_server_error(msg: &str) -> bool {
        msg.contains("server returned an error response")
            || msg.contains("error code")
            || msg.contains("-32601")
            || msg.contains("method")
    }

    #[tokio::test]
    async fn timeboost_extension_uses_expected_rpc_method_names() {
        let asserter = Asserter::new();
        let provider = ProviderBuilder::new().connect_mocked_client(asserter.clone());

        let submission = arbitrum_alloy_rpc_types::JsonExpressLaneSubmission {
            chain_id: alloy_primitives::U256::from(42161),
            round: 1,
            auction_contract_address: alloy_primitives::Address::ZERO,
            transaction: alloy_primitives::Bytes::new(),
            options: None,
            sequence_number: 0,
            signature: alloy_primitives::Bytes::new(),
        };

        let err = provider
            .timeboost_send_express_lane_transaction(submission)
            .await
            .unwrap_err();
        assert!(
            err.to_string()
                .contains("timeboost_sendExpressLaneTransaction"),
            "{err}"
        );
    }

    #[tokio::test]
    async fn timeboost_extension_live_local_chain_smoke() -> Result<(), Box<dyn std::error::Error>>
    {
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
        let _ = provider.get_block_number().await?;

        let submission = arbitrum_alloy_rpc_types::JsonExpressLaneSubmission {
            chain_id: alloy_primitives::U256::from(42161),
            round: 1,
            auction_contract_address: alloy_primitives::Address::ZERO,
            transaction: alloy_primitives::Bytes::new(),
            options: None,
            sequence_number: 0,
            signature: alloy_primitives::Bytes::new(),
        };

        if let Err(e) = provider
            .timeboost_send_express_lane_transaction(submission)
            .await
        {
            assert!(looks_like_rpc_server_error(&e.to_string()), "{e}");
        }

        Ok(())
    }
}
