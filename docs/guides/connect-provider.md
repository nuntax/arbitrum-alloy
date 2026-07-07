# Connect To A Provider

`arbitrum-alloy` works with Alloy providers but uses the `Arbitrum` network type.

## HTTP Provider

```rust
use alloy_provider::{Provider, ProviderBuilder};
use arbitrum_alloy::network::Arbitrum;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let provider = ProviderBuilder::<_, _, Arbitrum>::default()
    .connect("http://localhost:8547")
    .await?;

let chain_id = provider.get_chain_id().await?;
println!("chain id: {chain_id}");
# Ok(())
# }
```

## WebSocket Provider

```rust
use alloy_provider::ProviderBuilder;
use arbitrum_alloy::network::Arbitrum;

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let provider = ProviderBuilder::<_, _, Arbitrum>::default()
    .connect("ws://localhost:8548")
    .await?;
# Ok(())
# }
```

## Why The Network Type Matters

The `Arbitrum` type enables typed handling of Arbitrum-specific transaction and receipt variants.
