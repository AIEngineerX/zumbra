use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::Serialize;
use zcash_protocol::consensus::Network;

mod benchmark;
mod daemon;
mod helpers;
mod payment;
mod policy;
mod wallet;

// ---------------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(
    name = "zumbra",
    about = "Headless Zcash light wallet for AI agents",
    version
)]
struct Cli {
    /// Wallet data directory
    #[arg(long, global = true)]
    data_dir: Option<String>,

    /// Use Zcash testnet
    #[arg(long, global = true)]
    testnet: bool,

    /// Override lightwalletd server URL
    #[arg(long, global = true)]
    server: Option<String>,

    /// Human-readable output instead of JSON
    #[arg(long, global = true)]
    human: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print version and build info
    Info,

    /// Wallet lifecycle
    #[command(subcommand)]
    Wallet(WalletCmd),

    /// Sync with the Zcash network
    #[command(subcommand)]
    Sync(SyncCmd),

    /// Show wallet balance
    Balance,

    /// Show wallet addresses
    Address,

    /// Export viewing keys (UFVK, UIVK)
    Keys,

    /// Show transaction history
    Transactions {
        /// Maximum number of transactions to show
        #[arg(long, default_value = "20")]
        limit: usize,
    },

    /// Send ZEC (two-step: propose then confirm)
    #[command(subcommand)]
    Send(SendCmd),

    /// Shield transparent funds into the shielded pool
    Shield,

    /// Consolidate shielded notes (send-to-self to reduce note count)
    Consolidate,

    /// Spending policy management
    #[command(subcommand)]
    Policy(PolicyCmd),

    /// View the audit log
    Audit {
        /// Maximum number of entries
        #[arg(long, default_value = "50")]
        limit: usize,

        /// Only show entries since this ISO 8601 timestamp
        #[arg(long)]
        since: Option<String>,
    },

    /// Daemon mode (long-running background process)
    #[command(subcommand)]
    Daemon(DaemonCmd),

    /// Pay an HTTP 402 paywall (x402 protocol)
    #[command(subcommand)]
    X402(X402Cmd),









    /// Ironwood pool transfer — migrate Orchard funds per ZIP 318
    #[command(subcommand)]
    Ironwood(IronwoodCmd),

}



#[derive(Subcommand)]
enum WalletCmd {
    /// One-command setup: creates OWS vault + Zcash wallet + default policy.
    /// Prints seed phrase once, then MCP config JSON for Claude/Cursor.
    Init,

    /// Restore wallet from an existing seed phrase
    Restore {
        /// 24-word BIP39 seed phrase
        #[arg(long)]
        seed: String,

        /// Wallet birthday (block height to scan from)
        #[arg(long, default_value_t = 419200)]
        birthday: u32,
    },

    /// Delete wallet data from disk
    Delete {
        /// Required flag to confirm deletion
        #[arg(long)]
        confirm: bool,
    },
}

#[derive(Subcommand)]
enum SyncCmd {
    /// Start syncing (blocks until fully synced, Ctrl+C to stop)
    Start,

    /// Show current sync progress
    Status,

    /// Benchmark sync progress for the current wallet
    Benchmark {
        /// Stop after this many seconds even if not caught up (0 = no limit)
        #[arg(long, default_value_t = 0)]
        max_seconds: u64,

        /// Poll interval in milliseconds
        #[arg(long, default_value_t = 1000)]
        poll_ms: u64,

        /// Number of scan batches to prefetch (0 disables prefetch for baseline runs)
        #[arg(long, default_value_t = 3)]
        prefetch_depth: usize,

        /// Fetch scan batches from multiple known lightwalletd servers with primary fallback
        #[arg(long, default_value_t = false)]
        multi_server: bool,
    },
}

#[derive(Subcommand)]
enum SendCmd {
    /// Create a send proposal (no seed required)
    Propose {
        /// Destination address
        #[arg(long)]
        to: String,

        /// Amount in zatoshis (ignored if --max is set)
        #[arg(long, default_value_t = 0)]
        amount: u64,

        /// Send the maximum spendable amount (overrides --amount)
        #[arg(long, default_value_t = false)]
        max: bool,

        /// Optional memo (shielded only)
        #[arg(long)]
        memo: Option<String>,

        /// Use priority fee (4x marginal fee for faster confirmation during congestion)
        #[arg(long, default_value_t = false)]
        priority: bool,

        /// Context identifier for audit trail
        #[arg(long)]
        context_id: Option<String>,
    },

    /// Sign and broadcast a pending proposal (requires seed)
    Confirm,

    /// Show maximum sendable amount to an address
    Max {
        /// Destination address
        #[arg(long)]
        to: String,
    },

    /// Create a PCZT (unsigned transaction) for external signing via OWS
    Pczt {
        /// Destination address
        #[arg(long)]
        to: String,

        /// Amount in zatoshis
        #[arg(long)]
        amount: u64,

        /// Optional memo (shielded only)
        #[arg(long)]
        memo: Option<String>,
    },

    /// Store a signed PCZT back into the wallet DB (prevents double-spends)
    StorePczt {
        /// Hex-encoded signed PCZT bytes
        #[arg(long)]
        pczt: String,
    },
}

#[derive(Subcommand)]
enum PolicyCmd {
    /// Display the current spending policy
    Show,

    /// Set a policy field (e.g., max_per_tx, daily_limit, min_spend_interval_ms, approval_threshold, require_context_id)
    Set {
        /// Field name
        #[arg(long)]
        field: String,

        /// Field value
        #[arg(long)]
        value: String,
    },

    /// Add an address to the allowlist
    AddAllowlist {
        /// Address to allow
        #[arg(long)]
        address: String,
    },

    /// Remove an address from the allowlist
    RemoveAllowlist {
        /// Address to remove
        #[arg(long)]
        address: String,
    },
}

#[derive(Subcommand)]
enum DaemonCmd {
    /// Start the daemon (foreground process with sync loop + IPC socket)
    Start,

    /// Check if the daemon is running
    Status,

    /// Ask the daemon to stop
    Stop,

    /// Zeroize seed material in memory (wallet becomes read-only, sync continues)
    Lock,

    /// Unlock spending (daemon decrypts seed from the OWS vault)
    Unlock,
}


#[derive(Subcommand)]
enum X402Cmd {
    /// Parse a 402 response and create a send proposal (no seed required)
    Propose {
        /// The HTTP 402 response body JSON (reads from stdin if omitted)
        #[arg(long)]
        body: Option<String>,

        /// Context identifier for audit trail
        #[arg(long)]
        context_id: Option<String>,
    },

    /// Parse a 402 response, pay, and return the PAYMENT-SIGNATURE header (requires seed)
    Pay {
        /// The HTTP 402 response body JSON (reads from stdin if omitted)
        #[arg(long)]
        body: Option<String>,

        /// Context identifier for audit trail
        #[arg(long)]
        context_id: Option<String>,
    },
}


#[derive(Subcommand)]
enum IronwoodCmd {
    /// Show the SDK transfer plan (requires an unlocked signing seed)
    Plan,

    /// Legacy execution entry point (currently unavailable; use the wallet app)
    Confirm {
        /// Enable Tor for broadcasting migration transactions
        #[arg(long)]
        tor: bool,
    },

    /// Show progress of an active transfer
    Status,

    /// Legacy pause entry point (currently unavailable; use the wallet app)
    Pause,

    /// Legacy resume entry point (currently unavailable; use the wallet app)
    Resume,
}

// ---------------------------------------------------------------------------
// JSON output helpers
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct CliOutput<T: Serialize> {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn print_ok<T: Serialize>(data: T, human: bool, human_fmt: impl FnOnce(&T)) {
    if human {
        human_fmt(&data);
    } else {
        let output = CliOutput {
            ok: true,
            data: Some(data),
            error: None,
        };
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }
}

fn print_err(e: &anyhow::Error, human: bool) {
    if human {
        eprintln!("Error: {:#}", e);
    } else {
        let output: CliOutput<()> = CliOutput {
            ok: false,
            data: None,
            error: Some(format!("{:#}", e)),
        };
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }
}

// ---------------------------------------------------------------------------
// Config resolution
// ---------------------------------------------------------------------------

const DEFAULT_MAINNET_SERVER: &str = "https://zec.rocks:443";
const DEFAULT_TESTNET_SERVER: &str = "https://testnet.zec.rocks:443";

pub struct Config {
    pub data_dir: String,
    pub server_url: String,
    pub network: Network,
    pub human: bool,
}

fn resolve_config(cli: &Cli) -> Config {
    let network = if cli.testnet {
        Network::TestNetwork
    } else {
        Network::MainNetwork
    };

    let default_server = if cli.testnet {
        DEFAULT_TESTNET_SERVER
    } else {
        DEFAULT_MAINNET_SERVER
    };
    let server_url = cli
        .server
        .clone()
        .unwrap_or_else(|| default_server.to_string());

    let net_suffix = if cli.testnet { "testnet" } else { "mainnet" };
    let data_dir = cli.data_dir.clone().unwrap_or_else(|| {
        let home = dirs::home_dir().expect("Cannot determine home directory");
        home.join(".zumbra")
            .join(net_suffix)
            .to_string_lossy()
            .to_string()
    });

    Config {
        data_dir,
        server_url,
        network,
        human: cli.human,
    }
}

pub fn ensure_data_dir(data_dir: &str) -> Result<()> {
    std::fs::create_dir_all(data_dir)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let cfg = resolve_config(&cli);

    let log_level = if cli.human {
        tracing::Level::INFO
    } else {
        std::env::var("ZUMBRA_LOG")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(tracing::Level::WARN)
    };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    let result: Result<()> = match cli.command {
        Commands::Info => {
            wallet::cmd_info(&cfg).await;
            Ok(())
        }
        Commands::Wallet(sub) => match sub {
            WalletCmd::Init => wallet::cmd_wallet_init(&cfg).await,
            WalletCmd::Restore { seed, birthday } => wallet::cmd_wallet_restore(&cfg, &seed, birthday).await,
            WalletCmd::Delete { confirm } => wallet::cmd_wallet_delete(&cfg, confirm).await,
        },
        Commands::Sync(sub) => match sub {
            SyncCmd::Start => wallet::cmd_sync_start(&cfg).await,
            SyncCmd::Status => wallet::cmd_sync_status(&cfg).await,
            SyncCmd::Benchmark {
                max_seconds,
                poll_ms,
                prefetch_depth,
                multi_server,
            } => {
                benchmark::cmd_sync_benchmark(
                    &cfg,
                    max_seconds,
                    poll_ms,
                    prefetch_depth,
                    multi_server,
                )
                .await
            }
        },
        Commands::Balance => wallet::cmd_balance(&cfg).await,
        Commands::Address => wallet::cmd_address(&cfg).await,
        Commands::Keys => wallet::cmd_keys(&cfg).await,
        Commands::Transactions { limit } => wallet::cmd_transactions(&cfg, limit).await,
        Commands::Send(sub) => match sub {
            SendCmd::Propose {
                to,
                amount,
                max,
                memo,
                context_id,
                priority,
            } => wallet::cmd_send_propose(&cfg, to, amount, max, memo, context_id, priority).await,
            SendCmd::Confirm => wallet::cmd_send_confirm(&cfg).await,
            SendCmd::Max { to } => wallet::cmd_send_max(&cfg, to).await,
            SendCmd::Pczt { to, amount, memo } => {
                market::cmd_send_pczt(&cfg, to, amount, memo).await
            }
            SendCmd::StorePczt { pczt } => wallet::cmd_store_signed_pczt(&cfg, pczt).await,
        },
        Commands::Shield => wallet::cmd_shield(&cfg).await,
        Commands::Consolidate => wallet::cmd_consolidate(&cfg).await,
        Commands::Policy(sub) => match sub {
            PolicyCmd::Show => policy::cmd_policy_show(&cfg).await,
            PolicyCmd::Set { field, value } => policy::cmd_policy_set(&cfg, field, value).await,
            PolicyCmd::AddAllowlist { address } => {
                policy::cmd_policy_add_allowlist(&cfg, address).await
            }
            PolicyCmd::RemoveAllowlist { address } => {
                policy::cmd_policy_remove_allowlist(&cfg, address).await
            }
        },
        Commands::Audit { limit, since } => policy::cmd_audit(&cfg, limit, since).await,
        Commands::Daemon(sub) => match sub {
            DaemonCmd::Start => daemon::cmd_start(&cfg).await,
            DaemonCmd::Status => daemon::cmd_status(&cfg).await,
            DaemonCmd::Stop => daemon::cmd_stop(&cfg).await,
            DaemonCmd::Lock => daemon::cmd_lock(&cfg).await,
            DaemonCmd::Unlock => daemon::cmd_unlock(&cfg).await,
        },
        Commands::X402(sub) => match sub {
            X402Cmd::Propose { body, context_id } => {
                payment::cmd_x402_propose(&cfg, body, context_id).await
            }
            X402Cmd::Pay { body, context_id } => {
                payment::cmd_x402_pay(&cfg, body, context_id).await
            }
        },
        Commands::Ironwood(sub) => match sub {
            IronwoodCmd::Plan => wallet::cmd_ironwood_plan(&cfg).await,
            IronwoodCmd::Confirm { tor } => wallet::cmd_ironwood_confirm(&cfg, tor).await,
            IronwoodCmd::Status => wallet::cmd_ironwood_status(&cfg).await,
            IronwoodCmd::Pause => wallet::cmd_ironwood_pause(&cfg).await,
            IronwoodCmd::Resume => wallet::cmd_ironwood_resume(&cfg).await,
        },

    };

    if let Err(e) = result {
        print_err(&e, cfg.human);
        std::process::exit(1);
    }
}
