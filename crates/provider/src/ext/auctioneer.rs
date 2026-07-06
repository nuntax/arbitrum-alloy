use alloc::boxed::Box;
use alloy_network::Network;
use alloy_primitives::Bytes;
use alloy_provider::Provider;
use alloy_transport::TransportResult;
use arb_alloy_network::Arbitrum;

/// Provider extension trait for the `auctioneer_*` JSON-RPC namespace.
///
/// Nitro reference: `execution/gethexec/api.go` -> `ArbTimeboostAuctioneerAPI`.
#[cfg_attr(target_family = "wasm", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
pub trait AuctioneerProviderExt<N: Network = Arbitrum>: Send + Sync {
    /// Submit a signed auction resolution transaction.
    ///
    /// The `raw_tx` parameter is the RLP-encoded signed transaction bytes.
    ///
    /// Nitro reference: `execution/gethexec/api.go` -> `SubmitAuctionResolutionTransaction`.
    async fn auctioneer_submit_auction_resolution_transaction(
        &self,
        raw_tx: Bytes,
    ) -> TransportResult<()>;
}

#[cfg_attr(target_family = "wasm", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl<N, P> AuctioneerProviderExt<N> for P
where
    N: Network,
    P: Provider<N>,
{
    async fn auctioneer_submit_auction_resolution_transaction(
        &self,
        raw_tx: Bytes,
    ) -> TransportResult<()> {
        self.client()
            .request("auctioneer_submitAuctionResolutionTransaction", (raw_tx,))
            .await
    }
}

#[cfg(test)]
mod tests {
    use alloy_provider::{Provider, ProviderBuilder};
    use alloy_transport::mock::Asserter;
    use arb_alloy_network::Arbitrum;

    use super::AuctioneerProviderExt;

    fn looks_like_rpc_server_error(msg: &str) -> bool {
        msg.contains("server returned an error response")
            || msg.contains("error code")
            || msg.contains("-32601")
            || msg.contains("method")
    }

    #[tokio::test]
    async fn auctioneer_extension_uses_expected_rpc_method_names() {
        let asserter = Asserter::new();
        let provider = ProviderBuilder::new().connect_mocked_client(asserter.clone());

        let err = provider
            .auctioneer_submit_auction_resolution_transaction(alloy_primitives::Bytes::new())
            .await
            .unwrap_err();
        assert!(
            err.to_string()
                .contains("auctioneer_submitAuctionResolutionTransaction"),
            "{err}"
        );
    }

    #[tokio::test]
    async fn auctioneer_extension_live_local_chain_smoke() -> Result<(), Box<dyn std::error::Error>>
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

        if let Err(e) = provider
            .auctioneer_submit_auction_resolution_transaction(alloy_primitives::Bytes::new())
            .await
        {
            assert!(looks_like_rpc_server_error(&e.to_string()), "{e}");
        }

        Ok(())
    }
}
