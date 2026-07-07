# Quickstart

This gets you running with `arbitrum-alloy` in a few minutes.

## 1. Add Dependencies

```bash
cargo add arbitrum-alloy alloy-provider tokio -F tokio/macros -F tokio/rt-multi-thread
```

## 2. Create A Typed Arbitrum Provider

```rust
use alloy_provider::{Provider, ProviderBuilder};
use arbitrum_alloy::network::Arbitrum;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = ProviderBuilder::<_, _, Arbitrum>::default()
        .connect("http://localhost:8547")
        .await?;

    let latest = provider.get_block_number().await?;
    println!("latest block: {latest}");

    Ok(())
}
```

## 3. Run

```bash
cargo run
```

## Next

- Call Arbitrum RPC extensions: [provider extensions guide](./guides/provider-extensions.md)
- Use local testing stack: [local dev chain guide](./guides/local-dev-chain.md)
