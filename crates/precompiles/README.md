# arbitrum-alloy-precompiles

> Unofficial. Not affiliated with, or endorsed by, Offchain Labs or the Arbitrum Foundation.

Arbitrum precompile address constants and `sol!` interface bindings.

Provides typed Solidity bindings for all Arbitrum precompile contracts, generated
via the `alloy::sol!` macro. These can be used with any alloy `Provider` to call
precompile methods in a fully typed manner.

## Usage

```rust,ignore
use alloy_provider::ProviderBuilder;
use arbitrum_alloy_precompiles::{ArbSys, addresses};

let provider = ProviderBuilder::new()
    .connect_http("https://arb1.arbitrum.io/rpc".parse().unwrap());

let arb_sys = ArbSys::new(addresses::ARB_SYS, &provider);
let block_num = arb_sys.arbBlockNumber().call().await?;
```
