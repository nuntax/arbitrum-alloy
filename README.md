# arbitrum-alloy

<img src="./arb-alloy-logo.png" alt="arb-alloy logo" width="160" />

Arbitrum integrations for the Alloy Rust SDK.

## ⚠️ Warning
This project is under active development and not yet stable. API and feature coverage may change at any time. This project is neither affiliated or endorsed by Alloy or OffchainLabs.

## Quick Start

```rust
use alloy_provider::{Provider, ProviderBuilder};
use arb_alloy::{network::Arbitrum, provider::ArbProviderExt};

#[tokio::main()]
async fn main() {
    let provider = ProviderBuilder::<_, _, Arbitrum>::default()
        .connect("http://localhost:8547")
        .await.unwrap();

    let latest = provider.get_block_number().await.unwrap();
    println!("latest block: {latest}");

    let _ = provider.arb_maintenance_status().await;
}
```

## Documentation

- [Docs Index](./docs/README.md)
- [Quickstart Guide](./docs/quickstart.md)
- [Connect To A Provider](./docs/guides/connect-provider.md)
- [Provider Extensions](./docs/guides/provider-extensions.md)
- [Use Precompiles](./docs/guides/precompiles.md)
- [Local Dev Chain](./docs/guides/local-dev-chain.md)
- [FAQ](./docs/faq.md)

## Crates

Published on crates.io under the `arbitrum-alloy-*` namespace (the shorter `arb-alloy-*`
names are held by an unrelated project):

- [`arbitrum-alloy`](https://crates.io/crates/arbitrum-alloy): umbrella crate re-exporting the components below.
- [`arbitrum-alloy-consensus`](https://crates.io/crates/arbitrum-alloy-consensus): Arbitrum consensus transaction and receipt types.
- [`arbitrum-alloy-network`](https://crates.io/crates/arbitrum-alloy-network): `Network` implementation for Arbitrum.
- [`arbitrum-alloy-rpc-types`](https://crates.io/crates/arbitrum-alloy-rpc-types): Arbitrum RPC request/response types.
- [`arbitrum-alloy-provider`](https://crates.io/crates/arbitrum-alloy-provider): provider extension traits for `arb_*` RPC methods.
- [`arbitrum-alloy-precompiles`](https://crates.io/crates/arbitrum-alloy-precompiles): Arbitrum precompile address constants and `sol!` bindings.
- [`arbitrum-alloy-sequencer`](https://crates.io/crates/arbitrum-alloy-sequencer): Arbitrum sequencer feed protocol types.

Each crate keeps its original `arb_alloy_*` import path, so the package name and the name you
`use` differ. For the umbrella:

```toml
[dependencies]
arbitrum-alloy = { version = "0.1", features = ["network", "provider"] }
```

```rust
use arb_alloy::{network::Arbitrum, provider::ArbProviderExt};
```
