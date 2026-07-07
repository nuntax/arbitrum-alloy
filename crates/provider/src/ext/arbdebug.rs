use alloc::boxed::Box;
use alloy_network::Network;
use alloy_provider::Provider;
use alloy_rpc_types_eth::BlockNumberOrTag;
use alloy_transport::TransportResult;
use arb_alloy_network::Arbitrum;
use arb_alloy_rpc_types::{PricingModelHistory, TimeoutQueue, TimeoutQueueHistory};

/// Provider extension trait for the `arbdebug_*` JSON-RPC namespace.
///
/// These methods are only available on nodes started with the debug API
/// enabled. They expose internal pricing state and retryable queue data.
///
/// Nitro reference: `execution/gethexec/api.go` -> `ArbDebugAPI`.
#[cfg_attr(target_family = "wasm", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
pub trait ArbDebugProviderExt<N: Network = Arbitrum>: Send + Sync {
    /// Returns the L2 and L1 pricing model history for a block range.
    ///
    /// Nitro reference: `execution/gethexec/api.go` -> `PricingModel`.
    async fn arbdebug_pricing_model(
        &self,
        start: BlockNumberOrTag,
        end: BlockNumberOrTag,
    ) -> TransportResult<PricingModelHistory>;

    /// Returns the retryable timeout queue history for a block range.
    ///
    /// Nitro reference: `execution/gethexec/api.go` -> `TimeoutQueueHistory`.
    async fn arbdebug_timeout_queue_history(
        &self,
        start: BlockNumberOrTag,
        end: BlockNumberOrTag,
    ) -> TransportResult<TimeoutQueueHistory>;

    /// Returns the retryable timeout queue state at a specific block.
    ///
    /// Nitro reference: `execution/gethexec/api.go` -> `TimeoutQueue`.
    async fn arbdebug_timeout_queue(
        &self,
        block_num: BlockNumberOrTag,
    ) -> TransportResult<TimeoutQueue>;
}

#[cfg_attr(target_family = "wasm", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl<N, P> ArbDebugProviderExt<N> for P
where
    N: Network,
    P: Provider<N>,
{
    async fn arbdebug_pricing_model(
        &self,
        start: BlockNumberOrTag,
        end: BlockNumberOrTag,
    ) -> TransportResult<PricingModelHistory> {
        self.client()
            .request("arbdebug_pricingModel", (start, end))
            .await
    }

    async fn arbdebug_timeout_queue_history(
        &self,
        start: BlockNumberOrTag,
        end: BlockNumberOrTag,
    ) -> TransportResult<TimeoutQueueHistory> {
        self.client()
            .request("arbdebug_timeoutQueueHistory", (start, end))
            .await
    }

    async fn arbdebug_timeout_queue(
        &self,
        block_num: BlockNumberOrTag,
    ) -> TransportResult<TimeoutQueue> {
        self.client()
            .request("arbdebug_timeoutQueue", (block_num,))
            .await
    }
}

#[cfg(test)]
mod tests {
    use alloy_provider::{Provider, ProviderBuilder};
    use alloy_rpc_types_eth::BlockNumberOrTag;
    use alloy_transport::mock::Asserter;
    use arb_alloy_network::Arbitrum;

    use super::ArbDebugProviderExt;

    fn looks_like_rpc_server_error(msg: &str) -> bool {
        msg.contains("server returned an error response")
            || msg.contains("error code")
            || msg.contains("-32601")
            || msg.contains("method")
    }

    #[tokio::test]
    async fn arbdebug_extension_uses_expected_rpc_method_names() {
        let asserter = Asserter::new();
        let provider = ProviderBuilder::new().connect_mocked_client(asserter.clone());

        let err = provider
            .arbdebug_pricing_model(BlockNumberOrTag::Earliest, BlockNumberOrTag::Latest)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("arbdebug_pricingModel"), "{err}");

        let err = provider
            .arbdebug_timeout_queue_history(BlockNumberOrTag::Earliest, BlockNumberOrTag::Latest)
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("arbdebug_timeoutQueueHistory"),
            "{err}"
        );

        let err = provider
            .arbdebug_timeout_queue(BlockNumberOrTag::Latest)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("arbdebug_timeoutQueue"), "{err}");
    }

    #[tokio::test]
    async fn arbdebug_extension_live_local_chain_smoke() -> Result<(), Box<dyn std::error::Error>> {
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

        for res in [
            provider
                .arbdebug_pricing_model(BlockNumberOrTag::Earliest, BlockNumberOrTag::Latest)
                .await
                .map(|_| ()),
            provider
                .arbdebug_timeout_queue_history(
                    BlockNumberOrTag::Earliest,
                    BlockNumberOrTag::Latest,
                )
                .await
                .map(|_| ()),
            provider
                .arbdebug_timeout_queue(BlockNumberOrTag::Latest)
                .await
                .map(|_| ()),
        ] {
            if let Err(e) = res {
                assert!(looks_like_rpc_server_error(&e.to_string()), "{e}");
            }
        }

        Ok(())
    }
}
