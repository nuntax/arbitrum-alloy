# Use Precompile Bindings

`arbitrum-alloy-precompiles` exposes constants and `sol!` contract bindings for Arbitrum precompiles.

## Example: ArbSys

```rust
use alloy_provider::ProviderBuilder;
use arbitrum_alloy::{network::Arbitrum, precompiles::{addresses::ARB_SYS, interfaces::ArbSys}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = ProviderBuilder::<_, _, Arbitrum>::default()
        .connect("http://localhost:8547")
        .await?;

    let arb_sys = ArbSys::new(ARB_SYS, &provider);
    let chain_id = arb_sys.arbChainID().call().await?;
    println!("arbChainID: {chain_id}");

    Ok(())
}
```
