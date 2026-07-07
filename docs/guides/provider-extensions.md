# Call Arbitrum RPC Extensions

Use extension traits from `arbitrum-alloy-provider` through the umbrella crate.

## Example

```rust
use alloy_provider::{Provider, ProviderBuilder};
use alloy_rpc_types_eth::BlockNumberOrTag;
use arbitrum_alloy::{network::Arbitrum, provider::ArbProviderExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = ProviderBuilder::<_, _, Arbitrum>::default()
        .connect("http://localhost:8547")
        .await?;

    let _ = provider.arb_check_publisher_health().await;
    let _ = provider.arb_maintenance_status().await;
    let _ = provider
        .arb_get_raw_block_metadata(BlockNumberOrTag::Latest, BlockNumberOrTag::Latest)
        .await;

    Ok(())
}
```

## Method Availability

Some methods may be disabled by node config or Nitro version. Treat `method not found` as a normal runtime case and handle it explicitly.
