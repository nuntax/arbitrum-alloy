#![doc = include_str!("../README.md")]
#![allow(clippy::too_many_arguments)]

use std::{env, time::Duration};

use alloy_core::sol;
use alloy_eips::{BlockId, eip2718::Typed2718};
use alloy_network::{Ethereum, EthereumWallet};
use alloy_primitives::{Address, B256, Bytes, U256};
use alloy_provider::{Provider, ProviderBuilder, RootProvider};
use alloy_rpc_types_eth::{BlockNumberOrTag, TransactionReceipt};
use alloy_signer_local::PrivateKeySigner;
use arbitrum_alloy::network::Arbitrum;

sol!(
    #[sol(rpc)]
    Inbox,
    "src/abis/Inbox.json"
);

/// Returns an [`EthereumWallet`] built from the `DEV_PRIVKEY` env variable.
pub fn dev_wallet() -> EthereumWallet {
    let _ = dotenvy::dotenv();
    let key = env::var("DEV_PRIVKEY").expect("Set DEV_PRIVKEY env variable");
    let signer: PrivateKeySigner = key.parse().expect("invalid DEV_PRIVKEY");
    EthereumWallet::from(signer)
}

/// Returns the Ethereum address corresponding to `DEV_PRIVKEY`.
pub fn dev_address() -> Address {
    let _ = dotenvy::dotenv();
    let key = env::var("DEV_PRIVKEY").expect("Set DEV_PRIVKEY env variable");
    let signer: PrivateKeySigner = key.parse().expect("invalid DEV_PRIVKEY");
    signer.address()
}

/// Addresses of all Arbitrum rollup contracts deployed by the testnode.
#[derive(Debug, Clone, Copy)]
pub struct Addresses {
    /// RollupProxy – the main rollup contract.
    pub rollup: Address,
    /// Inbox (proxy) – L1→L2 message entry point.
    pub inbox: Address,
    /// Outbox (proxy) – L2→L1 message execution.
    pub outbox: Address,
    /// RollupEventInbox (proxy) – rollup event emissions.
    pub rollup_event_inbox: Address,
    /// ChallengeManager (proxy) – challenge game management.
    pub challenge_manager: Address,
    /// AdminProxy – admin upgrade entry point.
    pub admin_proxy: Address,
    /// SequencerInbox (proxy) – batch data submission.
    pub sequencer_inbox: Address,
    /// Bridge (proxy) – cross-chain message routing.
    pub bridge: Address,
    /// ValidatorWalletCreator – factory for validator wallets.
    pub validator_wallet_creator: Address,
}

impl Addresses {
    /// Reads all addresses from environment variables (see `.env`).
    pub fn from_env() -> Self {
        let parse = |key: &str| -> Address {
            env::var(key)
                .unwrap_or_else(|_| panic!("missing env var {key}"))
                .parse()
                .unwrap_or_else(|_| panic!("invalid address in {key}"))
        };

        Self {
            rollup: parse("ROLLUP_ADDRESS"),
            inbox: parse("INBOX_ADDRESS"),
            outbox: parse("OUTBOX_ADDRESS"),
            rollup_event_inbox: parse("ROLLUP_EVENT_INBOX_ADDRESS"),
            challenge_manager: parse("CHALLENGE_MANAGER_ADDRESS"),
            admin_proxy: parse("ADMIN_PROXY_ADDRESS"),
            sequencer_inbox: parse("SEQUENCER_INBOX_ADDRESS"),
            bridge: parse("BRIDGE_ADDRESS"),
            validator_wallet_creator: parse("VALIDATOR_WALLET_CREATOR_ADDRESS"),
        }
    }
}

/// Parameters for [`TestContext::deposit_eth`].
#[derive(Debug, Clone, Copy)]
pub struct DepositParams {
    /// ETH value to deposit into the sender's L2 account (sent with the L1 tx).
    pub value: U256,
}

/// Parameters for [`TestContext::send_unsigned_transaction`].
///
/// Uses `Inbox.sendL1FundedUnsignedTransaction` so gas is funded from L1 ETH
/// (`gas_limit * max_fee_per_gas` is sent with the L1 tx and credited to the
/// sender's L2 alias before execution).
#[derive(Debug, Clone)]
pub struct UnsignedTxParams {
    /// Gas limit for L2 execution.
    pub gas_limit: U256,
    /// Max fee per gas for L2 execution.
    pub max_fee_per_gas: U256,
    /// Sender nonce on L2.
    pub nonce: U256,
    /// L2 call target.
    pub to: Address,
    /// Calldata for the L2 call.
    pub data: Bytes,
}

/// Parameters for [`TestContext::send_contract_transaction`].
///
/// Uses `Inbox.sendL1FundedContractTransaction` so gas is funded from L1 ETH
/// (`gas_limit * max_fee_per_gas` is sent with the L1 tx and credited to the
/// sender's L2 alias before execution).
#[derive(Debug, Clone)]
pub struct ContractTxParams {
    /// Gas limit for L2 execution.
    pub gas_limit: U256,
    /// Max fee per gas for L2 execution.
    pub max_fee_per_gas: U256,
    /// L2 call target.
    pub to: Address,
    /// Calldata for the L2 call.
    pub data: Bytes,
}

/// Parameters for [`TestContext::submit_retryable`].
#[derive(Debug, Clone)]
pub struct RetryableParams {
    /// L2 destination address.
    pub to: Address,
    /// Value to pass to the L2 call (in wei).
    pub l2_call_value: U256,
    /// Max cost of submitting the retryable (paid on L1).
    pub max_submission_cost: U256,
    /// Address to refund excess submission fees on L2.
    pub excess_fee_refund_address: Address,
    /// Address to refund `l2_call_value` on L2 if the call fails.
    pub call_value_refund_address: Address,
    /// Gas limit for the L2 execution.
    pub gas_limit: U256,
    /// Max fee per gas for the L2 execution.
    pub max_fee_per_gas: U256,
    /// Calldata for the L2 call.
    pub data: Bytes,
    /// ETH sent with the L1 transaction (`l2_call_value + max_submission_cost + gas_limit * max_fee_per_gas`).
    pub value: U256,
}

/// Shared test context holding providers and all deployed contract addresses.
#[derive(Debug)]
pub struct TestContext<AP: Provider<Arbitrum>, EP: Provider<Ethereum>> {
    /// Arbitrum L2 provider (read-only).
    pub arbitrum_provider: AP,
    /// Ethereum L1 provider (read-only).
    pub ethereum_provider: EP,
    /// Deployed rollup contract addresses.
    pub addresses: Addresses,
}

impl TestContext<RootProvider<Arbitrum>, RootProvider<Ethereum>> {
    /// Builds a `TestContext` from environment variables, loading `.env` first.
    ///
    /// Panics if the required env vars are missing or the RPC connections fail.
    pub async fn from_env() -> Self {
        let _ = dotenvy::dotenv();
        let ethereum_rpc = env::var("ETHEREUM_RPC").expect("Set ETHEREUM_RPC env variable");
        let arbitrum_rpc = env::var("ARBITRUM_RPC").expect("Set ARBITRUM_RPC env variable");
        let addresses = Addresses::from_env();
        Self::from_urls_and_addresses(&arbitrum_rpc, &ethereum_rpc, addresses).await
    }

    /// Returns a `TestContext` if all required env vars are set, or `None` to allow tests
    /// to skip gracefully when the testnode is not running.
    pub async fn try_from_env() -> Option<Self> {
        let _ = dotenvy::dotenv();
        if env::var("ARBITRUM_RPC").is_err() || env::var("ETHEREUM_RPC").is_err() {
            return None;
        }
        Some(Self::from_env().await)
    }

    /// Builds a `TestContext` from explicit RPC URLs, inferring addresses from env.
    pub async fn from_urls(arbitrum_rpc: &str, ethereum_rpc: &str) -> Self {
        let _ = dotenvy::dotenv();
        let addresses = Addresses::from_env();
        Self::from_urls_and_addresses(arbitrum_rpc, ethereum_rpc, addresses).await
    }

    /// Builds a `TestContext` from explicit RPC URLs and an [`Addresses`] set.
    pub async fn from_urls_and_addresses(
        arbitrum_rpc: &str,
        ethereum_rpc: &str,
        addresses: Addresses,
    ) -> Self {
        let ethereum_provider = ProviderBuilder::<_, _, Ethereum>::default()
            .connect(ethereum_rpc)
            .await
            .expect("couldn't connect to Ethereum RPC");

        let arbitrum_provider = ProviderBuilder::<_, _, Arbitrum>::default()
            .connect(arbitrum_rpc)
            .await
            .expect("couldn't connect to Arbitrum RPC");

        Self {
            arbitrum_provider,
            ethereum_provider,
            addresses,
        }
    }
}

impl<AP: Provider<Arbitrum>, EP: Provider<Ethereum>> TestContext<AP, EP> {
    /// Returns an [`Inbox`] instance bound to the L1 provider.
    pub const fn inbox(&self) -> Inbox::InboxInstance<&EP, Ethereum> {
        Inbox::new(self.addresses.inbox, &self.ethereum_provider)
    }

    /// Returns an [`EthereumWallet`] built from the `DEV_PRIVKEY` env variable.
    pub fn dev_wallet() -> EthereumWallet {
        let _ = dotenvy::dotenv();
        let key = env::var("DEV_PRIVKEY").expect("Set DEV_PRIVKEY env variable");
        let signer: PrivateKeySigner = key.parse().expect("invalid DEV_PRIVKEY");
        EthereumWallet::from(signer)
    }

    /// Returns the Ethereum address corresponding to `DEV_PRIVKEY`.
    pub fn dev_address() -> Address {
        let _ = dotenvy::dotenv();
        let key = env::var("DEV_PRIVKEY").expect("Set DEV_PRIVKEY env variable");
        let signer: PrivateKeySigner = key.parse().expect("invalid DEV_PRIVKEY");
        signer.address()
    }

    /// Advances the L1 chain by `n` blocks by sending zero-value self-transfers from
    /// the dev wallet.  Geth in `--dev` mode mines one block per transaction, so each
    /// dummy transfer produces exactly one new block.
    ///
    /// The nitro-testnode sequencer has a `finalize-distance` of 1, so calling this
    /// with `n = 2` after submitting an L1→L2 message ensures the message has crossed
    /// the finality threshold and the sequencer will include it in the next L2 block.
    pub async fn advance_l1_blocks(&self, n: u64) -> Result<(), Box<dyn std::error::Error>> {
        use alloy_rpc_types_eth::TransactionRequest;

        let _ = dotenvy::dotenv();
        let key = env::var("DEV_PRIVKEY").expect("Set DEV_PRIVKEY");
        let signer: PrivateKeySigner = key.parse().expect("invalid DEV_PRIVKEY");
        let sender = signer.address();

        let l1 = self.l1_provider();
        for _ in 0..n {
            l1.send_transaction(TransactionRequest {
                from: Some(sender),
                to: Some(alloy_primitives::TxKind::Call(sender)),
                value: Some(U256::ZERO),
                ..Default::default()
            })
            .await?
            .get_receipt()
            .await?;
        }
        Ok(())
    }

    /// Scans L2 blocks starting at `since_block`, returning the hash of the first
    /// transaction whose EIP-2718 type byte equals `ty`.
    ///
    /// Blocks are fetched with hash-only transactions; each hash is then individually
    /// resolved via `eth_getTransactionByHash` to check the type. This avoids the
    /// full-block deserialization issues that arise with Arbitrum-specific tx types.
    ///
    /// Returns an error if `timeout` elapses before a matching transaction is found.
    pub async fn wait_for_l2_tx_type(
        &self,
        ty: u8,
        since_block: u64,
        timeout: Duration,
    ) -> Result<B256, Box<dyn std::error::Error>> {
        let deadline = tokio::time::Instant::now() + timeout;
        let mut scan_from = since_block;

        loop {
            if tokio::time::Instant::now() >= deadline {
                return Err(format!(
                    "timeout waiting for L2 tx of type 0x{ty:02x} since block {since_block}"
                )
                .into());
            }

            let latest = self.arbitrum_provider.get_block_number().await?;

            let mut bn = scan_from;
            while bn <= latest {
                let block = self
                    .arbitrum_provider
                    .get_block(BlockId::Number(BlockNumberOrTag::Number(bn)))
                    .await?;
                let Some(block) = block else {
                    bn = bn.saturating_add(1);
                    continue;
                };

                for hash in block.transactions.hashes() {
                    let hash = B256::new(*hash);
                    let tx = self.arbitrum_provider.get_transaction_by_hash(hash).await?;
                    if let Some(tx) = tx
                        && tx.inner.ty() == ty
                    {
                        return Ok(hash);
                    }
                }
                bn = bn.saturating_add(1);
            }
            scan_from = latest.saturating_add(1);

            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    /// Polls the L2 provider for a receipt of `tx_hash` until one appears or `timeout` elapses.
    pub async fn wait_for_l2_receipt(
        &self,
        tx_hash: B256,
        timeout: Duration,
    ) -> Result<arbitrum_alloy::rpc_types::ArbTransactionReceipt, Box<dyn std::error::Error>> {
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            if tokio::time::Instant::now() >= deadline {
                return Err(format!("timeout waiting for L2 receipt of {tx_hash}").into());
            }

            if let Some(receipt) = self
                .arbitrum_provider
                .get_transaction_receipt(tx_hash)
                .await?
            {
                return Ok(receipt);
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    // --- L1→L2 submission helpers ---

    /// Submits a retryable ticket to the L1 Inbox and waits for the L1 receipt.
    ///
    /// Signs with `DEV_PRIVKEY` from the environment.
    /// The `value` in [`RetryableParams`] must cover `l2_call_value +
    /// max_submission_cost + gas_limit * max_fee_per_gas`.
    pub async fn submit_retryable(
        &self,
        params: RetryableParams,
    ) -> Result<TransactionReceipt, Box<dyn std::error::Error>> {
        let l1 = self.l1_provider();
        let receipt = Inbox::new(self.addresses.inbox, &l1)
            .createRetryableTicket(
                params.to,
                params.l2_call_value,
                params.max_submission_cost,
                params.excess_fee_refund_address,
                params.call_value_refund_address,
                params.gas_limit,
                params.max_fee_per_gas,
                params.data,
            )
            .value(params.value)
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(receipt)
    }

    /// Deposits ETH from L1 to the sender's L2 account via `Inbox.depositEth()`.
    ///
    /// Produces a [`TxDeposit`](arbitrum_alloy::consensus::TxDeposit) (type `0x64`) on L2.
    /// Signs with `DEV_PRIVKEY` from the environment.
    pub async fn deposit_eth(
        &self,
        params: DepositParams,
    ) -> Result<TransactionReceipt, Box<dyn std::error::Error>> {
        let l1 = self.l1_provider();
        let receipt = Inbox::new(self.addresses.inbox, &l1)
            .depositEth_1()
            .value(params.value)
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(receipt)
    }

    /// Sends an unsigned L1-to-L2 transaction via `Inbox.sendL1FundedUnsignedTransaction()`.
    ///
    /// Produces a [`TxUnsigned`](arbitrum_alloy::consensus::TxUnsigned) (type `0x65`) on L2.
    /// ETH equal to `gas_limit * max_fee_per_gas` is sent from L1 and credited to the
    /// sender's L2 alias so it can pay for gas without a pre-funded alias balance.
    /// Signs with `DEV_PRIVKEY` from the environment.
    pub async fn send_unsigned_transaction(
        &self,
        params: UnsignedTxParams,
    ) -> Result<TransactionReceipt, Box<dyn std::error::Error>> {
        let l1 = self.l1_provider();
        let gas_eth = params.gas_limit * params.max_fee_per_gas;
        let receipt = Inbox::new(self.addresses.inbox, &l1)
            .sendL1FundedUnsignedTransaction(
                params.gas_limit,
                params.max_fee_per_gas,
                params.nonce,
                params.to,
                params.data,
            )
            .value(gas_eth)
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(receipt)
    }

    /// Sends a contract-originated L1-to-L2 transaction via `Inbox.sendL1FundedContractTransaction()`.
    ///
    /// Produces a [`TxContract`](arbitrum_alloy::consensus::TxContract) (type `0x66`) on L2.
    /// ETH equal to `gas_limit * max_fee_per_gas` is sent from L1 and credited to the
    /// sender's L2 alias so it can pay for gas without a pre-funded alias balance.
    /// Signs with `DEV_PRIVKEY` from the environment.
    pub async fn send_contract_transaction(
        &self,
        params: ContractTxParams,
    ) -> Result<TransactionReceipt, Box<dyn std::error::Error>> {
        let l1 = self.l1_provider();
        let gas_eth = params.gas_limit * params.max_fee_per_gas;
        let receipt = Inbox::new(self.addresses.inbox, &l1)
            .sendL1FundedContractTransaction(
                params.gas_limit,
                params.max_fee_per_gas,
                params.to,
                params.data,
            )
            .value(gas_eth)
            .send()
            .await?
            .get_receipt()
            .await?;
        Ok(receipt)
    }

    /// Builds a wallet-equipped L1 provider from `ETHEREUM_RPC` + `DEV_PRIVKEY`.
    fn l1_provider(&self) -> impl Provider<Ethereum> {
        let eth_rpc = env::var("ETHEREUM_RPC").expect("Set ETHEREUM_RPC");
        ProviderBuilder::new()
            .wallet(Self::dev_wallet())
            .connect_http(eth_rpc.parse().expect("invalid ETHEREUM_RPC URL"))
    }
}
