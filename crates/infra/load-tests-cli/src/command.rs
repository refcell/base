//! Load-test command parsing and execution.

use std::{num::NonZeroU64, path::PathBuf, time::Duration};

use alloy_network::{EthereumWallet, TransactionBuilder};
use alloy_primitives::{Address, U256, utils::format_ether};
use alloy_provider::Provider;
use alloy_rpc_types::{BlockNumberOrTag, TransactionRequest};
use alloy_signer_local::PrivateKeySigner;
use base_cli_utils::RuntimeManager;
use base_load_tests::{
    AccountPool, BaselineError, DEFAULT_MAX_GAS_PRICE, FundedAccount, LoadRunner, LoadTestDisplay,
    LoadTestDisplayConfig, LoadTestRunHooks, LoadTestRunOptions, MetricsSummary, QueryProvider,
    ReceiptCoverage, Result as LoadResult, RpcProviders, RpcResultExt, TestConfig,
    create_wallet_provider,
};
use clap::{ArgGroup, Args, Parser, Subcommand};
use eyre::{Result, bail};
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use tracing::{debug, info, warn};

const RESCUE_BATCH_SIZE: usize = 100;
const RESCUE_CONCURRENCY: usize = 32;
const DEFAULT_RESCUE_SCAN_COUNT: usize = 1000;

/// Standalone Base load tester CLI.
#[derive(Parser, Clone, Debug)]
#[command(
    author,
    version = env!("CARGO_PKG_VERSION"),
    about = "Base load tester",
    long_about = None,
    args_conflicts_with_subcommands = true
)]
pub struct LoadTestCli {
    /// Load-test command and arguments.
    #[command(flatten)]
    command: LoadTestCommand,
}

impl LoadTestCli {
    /// Runs the standalone CLI, including load-test-specific tracing initialization.
    pub fn run(self) -> Result<()> {
        self.command.run(true)
    }
}

/// Arguments accepted by the unified `base load-test` command.
#[derive(Args, Clone, Debug)]
#[command(args_conflicts_with_subcommands = true)]
pub struct LoadTestCommand {
    /// Load test arguments.
    #[command(flatten)]
    load: LoadArgs,
    /// Optional maintenance subcommand.
    #[command(subcommand)]
    command: Option<MaintenanceCommand>,
}

impl LoadTestCommand {
    /// Runs under an already initialized unified CLI tracing subscriber.
    pub fn run_with_existing_tracing(self) -> Result<()> {
        self.run(false)
    }

    /// Runs the command, optionally initializing a load-test tracing subscriber.
    pub fn run(self, initialize_tracing: bool) -> Result<()> {
        RuntimeManager::new().tokio_runtime()?.block_on(async move {
            match self.command {
                Some(MaintenanceCommand::Rescue(args)) => args.run(initialize_tracing).await,
                None => self.load.run(initialize_tracing).await,
            }
        })
    }
}

/// Load-test execution arguments.
#[derive(Args, Clone, Debug)]
#[command(group(
    ArgGroup::new("load_mode").multiple(false).args(["drain_only", "recover_real_tokens"])
))]
pub struct LoadArgs {
    /// Run continuously until interrupted.
    #[arg(long)]
    continuous: bool,
    /// Drain configured test accounts without running a load test.
    #[arg(long)]
    drain_only: bool,
    /// Recover real-token WETH/pair-token balances, then drain native ETH.
    #[arg(long)]
    recover_real_tokens: bool,
    /// Skip draining native ETH balances back to the funder account.
    #[arg(long)]
    skip_drain: bool,
    /// Benchmark-only directory for the ready/start handshake before measured submission.
    #[arg(long, value_name = "DIR", requires = "block_gas_limit")]
    separate_setup: Option<PathBuf>,
    /// Block gas limit used for in-flight inventory sizing instead of the latest RPC block.
    #[arg(long, requires = "separate_setup")]
    block_gas_limit: Option<NonZeroU64>,
    /// Load test YAML configuration.
    #[arg(value_name = "CONFIG", required = true)]
    config: Option<PathBuf>,
}

/// Selected load-test operation mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadMode {
    /// Execute the configured load test.
    Run,
    /// Drain configured test accounts without generating load.
    DrainOnly,
    /// Recover configured real-token balances before draining native ETH.
    RecoverRealTokens,
}

impl LoadArgs {
    /// Returns the operation mode selected by these arguments.
    pub const fn mode(&self) -> LoadMode {
        if self.drain_only {
            LoadMode::DrainOnly
        } else if self.recover_real_tokens {
            LoadMode::RecoverRealTokens
        } else {
            LoadMode::Run
        }
    }
}

/// Load-test maintenance commands.
#[derive(Subcommand, Clone, Debug)]
pub enum MaintenanceCommand {
    /// Rescue stranded funds from load test accounts.
    Rescue(RescueArgs),
}

impl LoadArgs {
    /// Runs the requested load-test operation.
    pub async fn run(self, initialize_tracing: bool) -> Result<()> {
        let args = self;
        let mode = args.mode();
        let Some(config_path) = args.config else {
            bail!("config argument is required");
        };
        let multi_progress = if initialize_tracing {
            LoadTestDisplay::init_tracing()?
        } else {
            LoadTestDisplay::progress_for_existing_tracing()
        };
        if !config_path.exists() {
            bail!("config file not found: {}", config_path.display());
        }

        let test_config = TestConfig::load(&config_path)?;
        let skip_drain = args.skip_drain || test_config.skip_drain;
        let query_rpc = match test_config.query_rpc.clone() {
            Some(query_rpc) => query_rpc,
            None => test_config.primary_submission_rpc()?.clone(),
        };
        let client = RpcProviders::query(query_rpc.clone())?;
        let rpc_chain_id = if test_config.chain_id.is_none() {
            Some(client.get_chain_id().await.rpc("chain id")?)
        } else {
            None
        };
        let mut load_config = test_config.to_load_config(rpc_chain_id)?;
        load_config.separate_setup = args.separate_setup;
        load_config.block_gas_limit = args.block_gas_limit.map(NonZeroU64::get);
        let funding_key = TestConfig::funder_key()?;

        match mode {
            LoadMode::DrainOnly => {
                println!("=== Drain-Only Mode ===");
                println!(
                    "Re-deriving {} accounts from config and draining to funder...",
                    load_config.account_count
                );
                if skip_drain {
                    println!("Skipping drain due to --skip-drain.");
                    return Ok(());
                }
                let runner = LoadRunner::new(load_config)?;
                match runner.drain_accounts(funding_key).await {
                    Ok(drained) => {
                        println!("Drained {} ETH back to funder.", format_ether(drained))
                    }
                    Err(error) => bail!("drain failed: {error}"),
                }
                return Ok(());
            }
            LoadMode::RecoverRealTokens => {
                let real_token_setup = test_config.parse_real_token_setup(load_config.chain_id)?;
                let Some(real_token_setup) = real_token_setup.as_ref() else {
                    bail!(
                        "--recover-real-tokens requires real_token_setup in {}",
                        config_path.display()
                    );
                };
                println!("=== Real-Token Recovery Mode ===");
                println!(
                    "Re-deriving {} accounts from config and recovering real-token balances to funder...",
                    load_config.account_count
                );
                let runner = LoadRunner::new(load_config)?;
                match runner.recover_real_tokens(real_token_setup).await {
                    Ok(summary) => println!(
                        "Recovered {} pair-token raw units and unwrapped {} WETH.",
                        summary.pair_token_swapped,
                        format_ether(summary.weth_unwrapped)
                    ),
                    Err(error) => bail!("real-token recovery failed: {error}"),
                }
                if skip_drain {
                    println!("Skipping drain due to --skip-drain.");
                } else {
                    match runner.drain_accounts(funding_key).await {
                        Ok(drained) => {
                            println!("Drained {} ETH back to funder.", format_ether(drained))
                        }
                        Err(error) => bail!("drain failed: {error}"),
                    }
                }
                return Ok(());
            }
            LoadMode::Run => {}
        }

        println!("=== Base Load Test Runner ===");
        println!("Set RPCs to internal endpoints to avoid rate limiting");
        println!(
            "Config: {} | Submit RPCs: {} | Query RPC: {} | Chain: {}",
            config_path.display(),
            test_config
                .transaction_submission_rpcs
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", "),
            query_rpc,
            load_config.chain_id
        );
        let duration_display = if args.continuous {
            "continuous".to_string()
        } else {
            load_config
                .duration
                .map_or_else(|| "continuous".to_string(), |duration| format!("{duration:?}"))
        };
        let target_gps_display = load_config
            .target_gps
            .map_or_else(|| "unbounded".to_string(), |gps| format!("{gps} gas/s"));
        println!(
            "Target cap: {} | Duration: {} | Accounts: {}",
            target_gps_display, duration_display, load_config.account_count
        );
        println!();

        let display_duration = if args.continuous { None } else { load_config.duration };
        let output = base_load_tests::LoadTestExecutor::run_prepared(
            test_config,
            load_config,
            funding_key,
            LoadTestRunOptions {
                continuous: args.continuous,
                install_signal_handler: true,
                skip_drain,
            },
            LoadTestRunHooks {
                display: multi_progress.map(|multi_progress| LoadTestDisplayConfig {
                    multi_progress,
                    duration: display_duration,
                }),
                before_cleanup: LoadTestCommand::present_load_test_summary,
            },
        )
        .await?;
        if let Some(error) = output.run_error {
            return Err(error.into());
        }
        Ok(())
    }
}

impl LoadTestCommand {
    /// Prints and optionally persists a completed load-test summary.
    pub fn present_load_test_summary(summary: &MetricsSummary) {
        if summary.error.is_none() || summary.throughput.total_submitted > 0 {
            println!();
            println!("=== Results ===");
            if let Some(ref error) = summary.error {
                println!("Error: {error}");
            }
            let throughput = &summary.throughput;
            println!("TPS: {:.2} | GPS: {:.0}", throughput.tps, throughput.gps);
            let pacing = &summary.pacing;
            let target_gps = summary
                .config
                .as_ref()
                .and_then(|config| config.target_gps)
                .map_or_else(|| "unbounded".to_string(), |target| target.to_string());
            println!(
                "Pacing: target_gps={}  offered_gps={:.0}  achieved_gps={:.0}",
                target_gps, pacing.offered_gps, throughput.gps
            );
            println!(
                "Depth: mean/floor={:.2}x  blocks={}  under_floor={}  max_gas={}  max_queued_gas={}",
                pacing.mean_depth_to_floor_ratio,
                pacing.blocks_observed,
                pacing.blocks_under_floor,
                pacing.max_depth_gas,
                pacing.max_queued_gas
            );
            println!(
                "Shortfalls: capacity={}  presign={}  rpc={}  chain={}",
                pacing.capacity_limited_cycles,
                pacing.presign_starved_cycles,
                pacing.rpc_bound_cycles,
                pacing.chain_bound_cycles
            );
            println!(
                "Refill Sources: canonical={}  flashblock={}  safety={}",
                pacing.canonical_cycles, pacing.flashblock_cycles, pacing.safety_cycles
            );
            println!(
                "Block Fill: mean={:.1}%  load_test_estimated={:.1}%",
                pacing.mean_block_fill_ratio * 100.0,
                pacing.mean_our_block_ratio * 100.0
            );
            println!(
                "Refill Lag: p50={:.1?}  p95={:.1?}  p99={:.1?}  max={:.1?}",
                pacing.refill_lag.p50,
                pacing.refill_lag.p95,
                pacing.refill_lag.p99,
                pacing.refill_lag.max
            );
            println!(
                "Cycle Work: plan_p95={:.1?}  submit_p95={:.1?}",
                pacing.plan_time.p95, pacing.submit_time.p95
            );
            println!(
                "Availability Lag: p50={:.1?}  p95={:.1?}  max={:.1?}",
                pacing.availability_lag.p50,
                pacing.availability_lag.p95,
                pacing.availability_lag.max
            );
            let block_latency = &summary.block_latency;
            println!(
                "Block Latency:       min={:.1?}  p50={:.1?}  mean={:.1?}  p95={:.1?}  p99={:.1?}  max={:.1?}",
                block_latency.min,
                block_latency.p50,
                block_latency.mean,
                block_latency.p95,
                block_latency.p99,
                block_latency.max
            );
            println!();
            println!(
                "Totals: Submitted={} | Confirmed={} | Failed={} | Reverted={} | Success={:.1}%",
                throughput.total_submitted,
                throughput.total_confirmed,
                throughput.total_failed,
                throughput.total_reverted,
                throughput.success_rate()
            );
            println!("Gas: total={}  avg/tx={}", summary.gas.total_gas, summary.gas.avg_gas);
            if pacing.undrained_transactions > 0 {
                println!(
                    "Undrained inventory: transactions={}  gas={}",
                    pacing.undrained_transactions, pacing.undrained_gas
                );
            }
            let receipt_coverage: &ReceiptCoverage = &summary.receipt_coverage;
            if !receipt_coverage.is_complete() {
                println!(
                    "Receipts: INCOMPLETE - gas/revert metrics are partial: receipts missing for {} of {} block(s), {} of {} confirmed tx(s) not enriched",
                    receipt_coverage.blocks_failed,
                    receipt_coverage.blocks_total,
                    receipt_coverage.transactions_missing,
                    receipt_coverage.transactions_total
                );
            }
            let block_range = &summary.block_range;
            match (block_range.first_block, block_range.last_block) {
                (Some(first), Some(last)) => println!(
                    "Blocks: first={first}  last={last}  span={} block(s)",
                    block_range.block_count
                ),
                _ => println!("Blocks: no confirmed transactions"),
            }
            if !summary.top_failure_reasons.is_empty() {
                println!("Top failures:");
                for (reason, count) in &summary.top_failure_reasons {
                    println!("  {count:>6}x  {reason}");
                }
            }
        } else if let Some(ref error) = summary.error {
            println!();
            println!("=== Error ===");
            println!("{error}");
        }

        if let Ok(output_path) = std::env::var("LOAD_TEST_OUTPUT") {
            match summary.to_json() {
                Ok(json) => match std::fs::write(&output_path, &json) {
                    Ok(()) => println!("Results written to {output_path}"),
                    Err(error) => {
                        eprintln!("Warning: failed to write results to {output_path}: {error}")
                    }
                },
                Err(error) => eprintln!("Warning: failed to serialize results: {error}"),
            }
        }
    }
}

/// Rescue command arguments.
#[derive(Args, Clone, Debug)]
#[command(group(
    ArgGroup::new("account_source").required(true).multiple(false).args(["seed", "mnemonic"])
))]
pub struct RescueArgs {
    /// RPC endpoint used to scan balances and submit drain transactions.
    #[arg(long = "rpc-url", alias = "rpc", value_name = "URL")]
    rpc_url: url::Url,
    /// Seed used for deterministic account generation.
    #[arg(long)]
    seed: Option<u64>,
    /// Number of accounts to scan.
    #[arg(long = "count", default_value_t = DEFAULT_RESCUE_SCAN_COUNT)]
    scan_count: usize,
    /// Starting account offset.
    #[arg(long, default_value_t = 0)]
    offset: usize,
    /// Mnemonic used for account generation.
    #[arg(long)]
    mnemonic: Option<String>,
    /// Private key of the funder account that receives drained funds.
    #[arg(long = "funder-key", env = "FUNDER_KEY", hide_env_values = true)]
    funder_key: PrivateKeySigner,
}

/// Parameters shared across rescue batches.
#[derive(Debug)]
pub struct DrainParams {
    funder_address: Address,
    chain_id: u64,
    max_fee: u128,
    max_priority_fee: u128,
    drain_gas_cost: U256,
    drain_gas_limit: u128,
    rpc_url: url::Url,
}

impl RescueArgs {
    /// Scans derived accounts and drains recoverable balances to the funder.
    pub async fn run(self, initialize_tracing: bool) -> Result<()> {
        let args = self;
        if initialize_tracing {
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "info".into()),
                )
                .try_init()
                .map_err(|error| eyre::eyre!("failed to initialize tracing: {error}"))?;
        }

        let client = RpcProviders::query(args.rpc_url.clone())?;
        let chain_id = client.get_chain_id().await.rpc("chain id")?;
        let funder_address = args.funder_key.address();
        let seed = args.seed.unwrap_or(0);
        println!("=== Load Test Rescue ===");
        println!("RPC: {} | Chain: {} | Funder: {}", args.rpc_url, chain_id, funder_address);
        println!("Scanning {} accounts (seed={}, offset={})\n", args.scan_count, seed, args.offset);

        let gas_price = client.get_gas_price().await.rpc("get gas price")?;
        let max_priority_fee = (gas_price / 10).max(1);
        let max_fee = gas_price.saturating_mul(2).max(max_priority_fee).min(DEFAULT_MAX_GAS_PRICE);
        let drain_gas_limit = 21_000u128;
        let l1_fee_buffer = 1_000_000_000_000_000u128;
        let drain_gas_cost =
            U256::from(drain_gas_limit.saturating_mul(max_fee).saturating_add(l1_fee_buffer));
        let params = DrainParams {
            funder_address,
            chain_id,
            max_fee,
            max_priority_fee,
            drain_gas_cost,
            drain_gas_limit,
            rpc_url: args.rpc_url.clone(),
        };

        let mut total_rescued = U256::ZERO;
        let mut total_accounts_drained = 0usize;
        let mut batch_offset = args.offset;
        let mut remaining = args.scan_count;
        let progress = Self::progress_bar(args.scan_count as u64, "Scanning accounts");
        while remaining > 0 {
            let batch_count = remaining.min(RESCUE_BATCH_SIZE);
            let accounts = if let Some(ref mnemonic) = args.mnemonic {
                AccountPool::from_mnemonic(mnemonic, batch_count, batch_offset)?
            } else {
                AccountPool::with_offset(seed, batch_count, batch_offset)?
            };
            let (rescued, drained) = params.rescue_batch(&client, &accounts, &progress).await?;
            total_rescued = total_rescued.saturating_add(rescued);
            total_accounts_drained += drained;
            batch_offset += batch_count;
            remaining -= batch_count;
        }
        progress.finish_and_clear();
        println!("\n=== Rescue Complete ===");
        println!(
            "Drained {} accounts | Total rescued: {} ETH",
            total_accounts_drained,
            format_ether(total_rescued)
        );
        Ok(())
    }
}

impl DrainParams {
    /// Drains one batch of accounts with recoverable balances.
    pub async fn rescue_batch(
        &self,
        client: &QueryProvider,
        accounts: &AccountPool,
        progress: &ProgressBar,
    ) -> LoadResult<(U256, usize)> {
        let balance_futures: Vec<_> = accounts
            .accounts()
            .iter()
            .map(|account| {
                let client = client.clone();
                let address = account.address;
                async move {
                    let balance = client
                        .get_balance(address)
                        .block_id(BlockNumberOrTag::Pending.into())
                        .await
                        .rpc("get pending balance")?;
                    Ok::<_, BaselineError>((address, balance))
                }
            })
            .collect();
        let balance_results: Vec<_> =
            stream::iter(balance_futures).buffered(RESCUE_CONCURRENCY).collect().await;
        let mut to_drain: Vec<(&FundedAccount, U256)> = Vec::new();
        for (result, account) in balance_results.into_iter().zip(accounts.accounts().iter()) {
            progress.inc(1);
            let (_, balance) = result?;
            if balance > self.drain_gas_cost {
                to_drain.push((account, balance));
            }
        }
        if to_drain.is_empty() {
            return Ok((U256::ZERO, 0));
        }

        let recoverable: U256 = to_drain
            .iter()
            .map(|(_, balance)| balance.saturating_sub(self.drain_gas_cost))
            .fold(U256::ZERO, |total, balance| total.saturating_add(balance));
        info!(
            accounts = to_drain.len(),
            recoverable_eth = %format_ether(recoverable),
            "found accounts with recoverable balance"
        );
        let drain_futures: Vec<_> = to_drain
            .iter()
            .map(|&(account, balance)| {
                let rpc_url = self.rpc_url.clone();
                let funder_address = self.funder_address;
                let chain_id = self.chain_id;
                let max_fee = self.max_fee;
                let max_priority_fee = self.max_priority_fee;
                let drain_gas_cost = self.drain_gas_cost;
                let drain_gas_limit = self.drain_gas_limit;
                let signer = account.signer.clone();
                let address = account.address;
                async move {
                    let send_amount = balance.saturating_sub(drain_gas_cost);
                    let wallet = EthereumWallet::from(signer);
                    let provider = create_wallet_provider(rpc_url, wallet);
                    let nonce = provider
                        .get_transaction_count(address)
                        .pending()
                        .await
                        .rpc("get pending transaction count")?;
                    let transaction = TransactionRequest::default()
                        .with_to(funder_address)
                        .with_value(send_amount)
                        .with_nonce(nonce)
                        .with_chain_id(chain_id)
                        .with_gas_limit(drain_gas_limit as u64)
                        .with_max_fee_per_gas(max_fee)
                        .with_max_priority_fee_per_gas(max_priority_fee);
                    match provider.send_transaction(transaction).await {
                        Ok(pending) => {
                            let transaction_hash = *pending.tx_hash();
                            debug!(
                                from = %address,
                                amount = %format_ether(send_amount),
                                transaction_hash = %transaction_hash,
                                "rescue drain transaction sent"
                            );
                            Ok(Some((address, send_amount)))
                        }
                        Err(error) => {
                            warn!(
                                from = %address,
                                error = %error,
                                "rescue drain transaction failed, skipping"
                            );
                            Ok(None)
                        }
                    }
                }
            })
            .collect();
        let drain_results: Vec<_> =
            stream::iter(drain_futures).buffer_unordered(RESCUE_CONCURRENCY).collect().await;
        let mut pending_accounts = Vec::new();
        let mut total_drained = U256::ZERO;
        let mut drain_count = 0usize;
        for result in drain_results {
            let result: LoadResult<Option<(Address, U256)>> = result;
            if let Some((address, amount)) = result? {
                pending_accounts.push(address);
                total_drained = total_drained.saturating_add(amount);
                drain_count += 1;
            }
        }
        if !pending_accounts.is_empty() {
            self.await_drained_balances(client, &mut pending_accounts).await?;
        }
        Ok((total_drained, drain_count))
    }

    /// Waits until submitted rescue drains settle or the bounded polling window expires.
    pub async fn await_drained_balances(
        &self,
        client: &QueryProvider,
        pending_accounts: &mut Vec<Address>,
    ) -> LoadResult<()> {
        let timeout = Duration::from_secs(60);
        let poll_interval = Duration::from_millis(500);
        let start = std::time::Instant::now();
        while !pending_accounts.is_empty() && start.elapsed() < timeout {
            tokio::time::sleep(poll_interval).await;
            let mut still_pending = Vec::new();
            for address in pending_accounts.drain(..) {
                match client.get_balance(address).await.rpc("get balance") {
                    Ok(balance) if balance <= self.drain_gas_cost => {
                        debug!(
                            address = %address,
                            balance = %balance,
                            "rescue drain balance settled"
                        );
                    }
                    Ok(_) => still_pending.push(address),
                    Err(error) => {
                        warn!(
                            address = %address,
                            error = %error,
                            "failed to check rescue drain balance"
                        );
                        still_pending.push(address);
                    }
                }
            }
            *pending_accounts = still_pending;
        }
        if !pending_accounts.is_empty() {
            let sample: Vec<_> = pending_accounts.iter().take(3).copied().collect();
            warn!(
                pending_account_count = pending_accounts.len(),
                pending_account_sample = ?sample,
                "some rescue balances did not settle within timeout"
            );
        }
        Ok(())
    }
}

impl RescueArgs {
    /// Creates the progress indicator used while scanning rescue accounts.
    pub fn progress_bar(total: u64, prefix: &str) -> ProgressBar {
        let progress = ProgressBar::new(total);
        progress.set_style(
            ProgressStyle::with_template("{prefix} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .expect("valid template")
                .progress_chars("█▓░"),
        );
        progress.set_prefix(prefix.to_string());
        progress
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use clap::Parser;

    use super::LoadTestCli;

    // Anvil's public default test key; not a credential.
    const PUBLIC_TEST_FUNDER_KEY: &str =
        "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
    const PUBLIC_TEST_SEED: &str = "88001";

    #[test]
    fn parses_normal_and_continuous_load_modes() {
        LoadTestCli::try_parse_from(["load-test", "config.yaml"]).unwrap();
        LoadTestCli::try_parse_from(["load-test", "--continuous", "config.yaml"]).unwrap();
    }

    #[test]
    fn rejects_conflicting_load_modes() {
        let error = LoadTestCli::try_parse_from([
            "load-test",
            "--drain-only",
            "--recover-real-tokens",
            "config.yaml",
        ])
        .unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn rejects_incomplete_benchmark_handshake_arguments() {
        let error = LoadTestCli::try_parse_from([
            "load-test",
            "--separate-setup",
            "/tmp/control",
            "config.yaml",
        ])
        .unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rejects_normal_load_without_config() {
        let error = LoadTestCli::try_parse_from(["load-test"]).unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn parses_rescue_without_load_config() {
        LoadTestCli::try_parse_from([
            "load-test",
            "rescue",
            "--rpc-url",
            "http://localhost:8545",
            "--seed",
            PUBLIC_TEST_SEED,
            "--count",
            "4",
            "--funder-key",
            PUBLIC_TEST_FUNDER_KEY,
        ])
        .unwrap();
    }

    #[test]
    fn rescue_requires_exactly_one_account_source() {
        let missing = LoadTestCli::try_parse_from([
            "load-test",
            "rescue",
            "--rpc-url",
            "http://localhost:8545",
            "--funder-key",
            PUBLIC_TEST_FUNDER_KEY,
        ])
        .unwrap_err();
        assert_eq!(missing.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn missing_config_is_reported_before_runtime_network_access() {
        let directory = tempfile::tempdir().unwrap();
        let cli = LoadTestCli::try_parse_from([
            "load-test",
            directory.path().join("missing.yaml").to_str().unwrap(),
        ])
        .unwrap();
        let error = cli.command.run(false).unwrap_err();
        assert!(error.to_string().contains("config file not found"));
    }

    #[test]
    fn unified_mode_reuses_an_existing_tracing_subscriber() {
        let directory = tempfile::tempdir().unwrap();
        let cli = LoadTestCli::try_parse_from([
            "load-test",
            directory.path().join("missing.yaml").to_str().unwrap(),
        ])
        .unwrap();
        let subscriber = tracing_subscriber::fmt().finish();
        let error =
            tracing::subscriber::with_default(subscriber, || cli.command.run(false)).unwrap_err();
        assert!(error.to_string().contains("config file not found"));
    }

    #[test]
    fn yaml_path_is_not_reinterpreted_as_cli_chain_configuration() {
        let directory = tempfile::tempdir().unwrap();
        let config = directory.path().join("custom-chain.yaml");
        fs::write(&config, "chain_id: 84532\ntransaction_submission_rpcs: []\n").unwrap();
        let cli = LoadTestCli::try_parse_from(["load-test", config.to_str().unwrap()]).unwrap();
        assert_eq!(cli.command.load.config, Some(config));
    }
}
