use alloc::boxed::Box;
use alloy_network::Network;
use alloy_primitives::TxHash;
use alloy_provider::Provider;
use alloy_transport::TransportResult;
use arb_alloy_network::Arbitrum;
use arb_alloy_rpc_types::TraceFilter;

/// Provider extension trait for the `arbtrace_*` JSON-RPC namespace.
///
/// Nitro forwards these calls to a fallback trace client (classic Arbitrum
/// node or Erigon). The request/response payloads follow the OpenEthereum
/// `trace_*` convention and are passed as opaque JSON.
///
/// Nitro reference: `execution/gethexec/api.go` -> `ArbTraceForwarderAPI`.
#[cfg_attr(target_family = "wasm", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
pub trait ArbTraceProviderExt<N: Network = Arbitrum>: Send + Sync {
    /// Trace a call.
    async fn arbtrace_call(
        &self,
        call_args: serde_json::Value,
        trace_types: serde_json::Value,
        block: serde_json::Value,
    ) -> TransportResult<serde_json::Value>;

    /// Trace multiple calls.
    async fn arbtrace_call_many(
        &self,
        calls: serde_json::Value,
        block: serde_json::Value,
    ) -> TransportResult<serde_json::Value>;

    /// Replay all transactions in a block with tracing.
    async fn arbtrace_replay_block_transactions(
        &self,
        block: serde_json::Value,
        trace_types: serde_json::Value,
    ) -> TransportResult<serde_json::Value>;

    /// Replay a single transaction with tracing.
    async fn arbtrace_replay_transaction(
        &self,
        tx_hash: TxHash,
        trace_types: serde_json::Value,
    ) -> TransportResult<serde_json::Value>;

    /// Get the trace of a transaction by hash.
    async fn arbtrace_transaction(&self, tx_hash: TxHash) -> TransportResult<serde_json::Value>;

    /// Get a specific trace by transaction hash and index path.
    async fn arbtrace_get(
        &self,
        tx_hash: TxHash,
        path: serde_json::Value,
    ) -> TransportResult<serde_json::Value>;

    /// Trace an entire block.
    async fn arbtrace_block(&self, block: serde_json::Value) -> TransportResult<serde_json::Value>;

    /// Filter traces matching the given criteria.
    async fn arbtrace_filter(&self, filter: TraceFilter) -> TransportResult<serde_json::Value>;
}

#[cfg_attr(target_family = "wasm", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
impl<N, P> ArbTraceProviderExt<N> for P
where
    N: Network,
    P: Provider<N>,
{
    async fn arbtrace_call(
        &self,
        call_args: serde_json::Value,
        trace_types: serde_json::Value,
        block: serde_json::Value,
    ) -> TransportResult<serde_json::Value> {
        self.client()
            .request("arbtrace_call", (call_args, trace_types, block))
            .await
    }

    async fn arbtrace_call_many(
        &self,
        calls: serde_json::Value,
        block: serde_json::Value,
    ) -> TransportResult<serde_json::Value> {
        self.client()
            .request("arbtrace_callMany", (calls, block))
            .await
    }

    async fn arbtrace_replay_block_transactions(
        &self,
        block: serde_json::Value,
        trace_types: serde_json::Value,
    ) -> TransportResult<serde_json::Value> {
        self.client()
            .request("arbtrace_replayBlockTransactions", (block, trace_types))
            .await
    }

    async fn arbtrace_replay_transaction(
        &self,
        tx_hash: TxHash,
        trace_types: serde_json::Value,
    ) -> TransportResult<serde_json::Value> {
        self.client()
            .request("arbtrace_replayTransaction", (tx_hash, trace_types))
            .await
    }

    async fn arbtrace_transaction(&self, tx_hash: TxHash) -> TransportResult<serde_json::Value> {
        self.client()
            .request("arbtrace_transaction", (tx_hash,))
            .await
    }

    async fn arbtrace_get(
        &self,
        tx_hash: TxHash,
        path: serde_json::Value,
    ) -> TransportResult<serde_json::Value> {
        self.client().request("arbtrace_get", (tx_hash, path)).await
    }

    async fn arbtrace_block(&self, block: serde_json::Value) -> TransportResult<serde_json::Value> {
        self.client().request("arbtrace_block", (block,)).await
    }

    async fn arbtrace_filter(&self, filter: TraceFilter) -> TransportResult<serde_json::Value> {
        self.client().request("arbtrace_filter", (filter,)).await
    }
}

#[cfg(test)]
mod tests {
    use alloy_provider::{Provider, ProviderBuilder};
    use alloy_transport::mock::Asserter;
    use arb_alloy_network::Arbitrum;

    use super::ArbTraceProviderExt;

    fn looks_like_rpc_server_error(msg: &str) -> bool {
        msg.contains("server returned an error response")
            || msg.contains("error code")
            || msg.contains("-32601")
            || msg.contains("method")
    }

    #[tokio::test]
    async fn arbtrace_extension_uses_expected_rpc_method_names() {
        let asserter = Asserter::new();
        let provider = ProviderBuilder::new().connect_mocked_client(asserter.clone());

        let err = provider
            .arbtrace_call(
                serde_json::json!({}),
                serde_json::json!(["trace"]),
                serde_json::json!("latest"),
            )
            .await
            .unwrap_err();
        assert!(err.to_string().contains("arbtrace_call"), "{err}");

        let err = provider
            .arbtrace_block(serde_json::json!("latest"))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("arbtrace_block"), "{err}");

        let err = provider
            .arbtrace_filter(Default::default())
            .await
            .unwrap_err();
        assert!(err.to_string().contains("arbtrace_filter"), "{err}");
    }

    #[tokio::test]
    async fn arbtrace_extension_live_local_chain_smoke() -> Result<(), Box<dyn std::error::Error>> {
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
                .arbtrace_call(
                    serde_json::json!({}),
                    serde_json::json!(["trace"]),
                    serde_json::json!("latest"),
                )
                .await
                .map(|_| ()),
            provider
                .arbtrace_block(serde_json::json!("latest"))
                .await
                .map(|_| ()),
            provider
                .arbtrace_filter(Default::default())
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
