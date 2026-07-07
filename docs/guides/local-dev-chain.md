# Local Dev Chain Setup

For local integration work, run a local Nitro stack and point `arbitrum-alloy` to it.

## Environment

```bash
export ARBITRUM_RPC=http://localhost:8547
export ETHEREUM_RPC=http://localhost:8545
export DEV_PRIVKEY=0x...
```

## Run Library Tests

```bash
cargo test --workspace --all-targets
```

Tests that require unavailable environment variables should skip.

## Typical Endpoints

- L2 HTTP: `http://localhost:8547`
- L2 WS: `ws://localhost:8548`
- L1 HTTP: `http://localhost:8545`
