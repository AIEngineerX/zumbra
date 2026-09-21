use std::io::{self, Write as _};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use zcash_protocol::consensus::Network;

use crate::helpers::*;
use crate::{ensure_data_dir, print_ok, Config};

pub async fn cmd_info(cfg: &Config) {
    #[derive(Serialize)]
    struct InfoData {
        version: String,
        engine: String,
        network: String,
        data_dir: String,
        server: String,
    }

    let data = InfoData {
        version: env!("CARGO_PKG_VERSION").to_string(),
        engine: "zumbra-engine".to_string(),
        network: if cfg.network == Network::TestNetwork {
            "testnet"
        } else {
            "mainnet"
        }
        .to_string(),
        data_dir: cfg.data_dir.clone(),
        server: cfg.server_url.clone(),
    };

    print_ok(data, cfg.human, |d| {
        println!("zumbra {}", d.version);
        println!("engine:  {}", d.engine);
        println!("network: {}", d.network);
        println!("data:    {}", d.data_dir);
        println!("server:  {}", d.server);
    });
}

pub async fn cmd_wallet_init(cfg: &Config) -> Result<()> {
    ensure_data_dir(&cfg.data_dir)?;
    ensure_sapling_params(&cfg.data_dir).await?;

    let db_path = std::path::PathBuf::from(&cfg.data_dir).join("zumbra-data.sqlite");
    if db_path.exists() {
        return Err(anyhow::anyhow!(
            "Wallet already exists in {}. Use `wallet delete --confirm` first.",
            cfg.data_dir,
        ));
    }

    let ows_wallet_name = std::env::var("OWS_WALLET").unwrap_or_else(|_| "default".to_string());
    let ows_passphrase = std::env::var("OWS_PASSPHRASE").unwrap_or_default();

    let (seed, created) = open_or_create_vault_seed(&ows_wallet_name, &ows_passphrase, None)?;
    if created {
        eprintln!("Created OWS vault wallet '{}'", ows_wallet_name);
    } else {
        eprintln!("OWS wallet '{}' already exists; reusing it. Its seed is not shown.", ows_wallet_name);
    }
    let seed_phrase = seed.expose_secret().clone();

    let height = zumbra_engine::wallet::fetch_latest_height(&cfg.server_url).await? as u32;

    zumbra_engine::wallet::restore(
        &cfg.data_dir,
        &cfg.server_url,
        cfg.network,
        &seed_phrase,
        height,
        None,
        None, // no Zumbra vault — seed lives in OWS vault
    )
    .await?;

    let default_policy = default_policy();
    zumbra_engine::policy::save_policy(&cfg.data_dir, &default_policy)?;

    // Get the wallet address for the MCP config
    let addresses = zumbra_engine::query::get_addresses()
        .await
        .unwrap_or_default();
    let address = addresses
        .first()
        .map(|a| a.address.clone())
        .unwrap_or_default();

    zumbra_engine::wallet::close().await;

    let mcp_config = serde_json::json!({
        "mcpServers": {
            "zumbra": {
                "command": "zumbra-mcp",
            }
        }
    });

    #[derive(Serialize)]
    struct InitResult {
        /// Present only when init generated the seed just now; a reused vault seed is never printed.
        #[serde(skip_serializing_if = "Option::is_none")]
        seed_phrase: Option<String>,
        birthday: u32,
        address: String,
        data_dir: String,
        ows_wallet: String,
        policy: zumbra_engine::policy::SpendingPolicy,
        mcp_config: serde_json::Value,
    }

    let result = InitResult {
        seed_phrase: created.then(|| seed.expose_secret().clone()),
        birthday: height,
        address: address.clone(),
        data_dir: cfg.data_dir.clone(),
        ows_wallet: ows_wallet_name.clone(),
        policy: default_policy,
        mcp_config: mcp_config.clone(),
    };

    print_ok(result, cfg.human, |r| {
        println!("Wallet initialized.");
        println!();
        match &r.seed_phrase {
            Some(phrase) => {
                println!("  SEED PHRASE (back this up; shown only once):");
                println!("  {}", phrase);
            }
            None => println!("  Seed: reused from OWS wallet '{}'; not shown.", r.ows_wallet),
        }
        println!();
        println!("  Address:    {}", r.address);
        println!("  Birthday:   {}", r.birthday);
        println!("  Data dir:   {}", r.data_dir);
        println!(
            "  OWS wallet: {} (encrypted at ~/.ows/wallets/)",
            r.ows_wallet
        );
        println!();
        println!("  Default policy:");
        println!(
            "    max_per_tx:         {} ZAT ({:.4} ZEC)",
            r.policy.max_per_tx,
            r.policy.max_per_tx as f64 / 1e8
        );
        println!(
            "    daily_limit:        {} ZAT ({:.4} ZEC)",
            r.policy.daily_limit,
            r.policy.daily_limit as f64 / 1e8
        );
        println!(
            "    approval_threshold: {} ZAT ({:.4} ZEC)",
            r.policy.approval_threshold,
            r.policy.approval_threshold as f64 / 1e8
        );
        println!("    Edit: {}/policy.toml", r.data_dir);
        println!();
        println!("  MCP config (add to Claude/Cursor settings):");
        println!("  {}", serde_json::to_string_pretty(&r.mcp_config).unwrap());
        println!();
        println!("  Fund the wallet, then your AI agent can spend shielded ZEC.");
    });
    Ok(())
}

/// The policy every fresh wallet starts with. Init and restore share it, so a restored wallet
/// is never unlimited.
pub fn default_policy() -> zumbra_engine::policy::SpendingPolicy {
    zumbra_engine::policy::SpendingPolicy {
        max_per_tx: 1_000_000,         // 0.01 ZEC
        daily_limit: 10_000_000,       // 0.1 ZEC
        approval_threshold: 5_000_000, // 0.05 ZEC
        require_context_id: false,
        min_spend_interval_ms: 0,
        allowlist: Vec::new(),
    }
}

/// An empty vault passphrase protects nothing. Refused at every point that would store a seed,
/// unless the operator set ZUMBRA_UNSAFE_EMPTY_PASSPHRASE=1 on purpose.
pub fn require_passphrase(passphrase: &str) -> Result<()> {
    if passphrase.is_empty() && std::env::var("ZUMBRA_UNSAFE_EMPTY_PASSPHRASE").as_deref() != Ok("1") {
        return Err(anyhow::anyhow!(
            "OWS_PASSPHRASE is empty. Set a real vault passphrase, or set              ZUMBRA_UNSAFE_EMPTY_PASSPHRASE=1 to accept an unprotected vault."
        ));
    }
    Ok(())
}

/// The seed for `wallet init`: the existing OWS vault wallet if there is one (`created` =
/// false), otherwise a fresh 24-word one (`created` = true). Only a seed created here may be
/// shown to the user; one that already lived in the vault is not init's to print.
pub fn open_or_create_vault_seed(
    ows_wallet: &str,
    passphrase: &str,
    vault_path: Option<&std::path::Path>,
) -> Result<(SecretString, bool)> {
    require_passphrase(passphrase)?;
    if ows_lib::get_wallet(ows_wallet, vault_path).is_ok() {
        let exported = ows_lib::export_wallet(ows_wallet, Some(passphrase), vault_path)
            .map_err(|e| anyhow::anyhow!("Failed to export OWS wallet: {}", e))?;
        if !exported.contains(' ') || exported.starts_with('{') {
            return Err(anyhow::anyhow!(
                "OWS wallet '{}' is a private-key wallet, not mnemonic. \
                 Zcash requires a mnemonic wallet for ZIP-32 key derivation.",
                ows_wallet,
            ));
        }
        return Ok((SecretString::new(exported), false));
    }
    ows_lib::create_wallet(ows_wallet, Some(24), Some(passphrase), vault_path)
        .map_err(|e| anyhow::anyhow!("Failed to create OWS wallet: {}", e))?;
    let exported = ows_lib::export_wallet(ows_wallet, Some(passphrase), vault_path)
        .map_err(|e| anyhow::anyhow!("Failed to export seed from new wallet: {}", e))?;
    Ok((SecretString::new(exported), true))
}

/// What `prepare_restore` put on disk. If the step after it fails, `rollback` removes it all,
/// so a half-done restore cannot be mistaken for a wallet by the next `wallet init`.
#[must_use]
#[derive(Debug)]
pub struct RestoreStaging {
    ows_wallet: String,
    vault_path: Option<std::path::PathBuf>,
    policy_path: std::path::PathBuf,
}

impl RestoreStaging {
    pub fn rollback(self) {
        ows_lib::delete_wallet(&self.ows_wallet, self.vault_path.as_deref()).ok();
        std::fs::remove_file(&self.policy_path).ok();
    }
}

/// Everything restore does before it touches the network: the phrase goes into the encrypted
/// OWS vault under `ows_wallet` and nowhere else, and the data dir gets the default policy.
/// Refuses to replace a vault wallet that already exists, so a typo cannot overwrite a seed.
pub fn prepare_restore(
    data_dir: &str,
    ows_wallet: &str,
    passphrase: &str,
    phrase: &str,
    vault_path: Option<&std::path::Path>,
) -> Result<RestoreStaging> {
    require_passphrase(passphrase)?;
    if ows_lib::get_wallet(ows_wallet, vault_path).is_ok() {
        return Err(anyhow::anyhow!(
            "OWS wallet '{}' already exists. Set OWS_WALLET to a new name for the restored seed, and keep OWS_WALLET set to that name whenever you run zumbra or zumbra-mcp.",
            ows_wallet
        ));
    }
    ows_lib::import_wallet_mnemonic(ows_wallet, phrase, Some(passphrase), None, vault_path)
        .map_err(|e| anyhow::anyhow!("Failed to store the seed in the OWS vault: {}", e))?;
    zumbra_engine::policy::save_policy(data_dir, &default_policy())?;
    Ok(RestoreStaging {
        ows_wallet: ows_wallet.to_string(),
        vault_path: vault_path.map(|p| p.to_path_buf()),
        policy_path: std::path::Path::new(data_dir).join("policy.toml"),
    })
}

pub async fn cmd_wallet_restore(cfg: &Config, birthday: u32) -> Result<()> {
    ensure_data_dir(&cfg.data_dir)?;

    let db_path = std::path::PathBuf::from(&cfg.data_dir).join("zumbra-data.sqlite");
    if db_path.exists() {
        return Err(anyhow::anyhow!(
            "Wallet already exists in {}. Use `wallet delete --confirm` first.",
            cfg.data_dir,
        ));
    }

    // The phrase comes in on stdin so it never appears in argv or shell history.
    if std::io::IsTerminal::is_terminal(&io::stdin()) {
        eprintln!("Paste the 24-word seed phrase and press Enter. It will be visible in this terminal; clear the scrollback afterwards.");
    }
    let mut line = zeroize::Zeroizing::new(String::new());
    io::stdin().read_line(&mut line)?;
    let phrase = zeroize::Zeroizing::new(line.trim().to_string());
    let words = phrase.split_whitespace().count();
    if words != 24 {
        return Err(anyhow::anyhow!(
            "Expected a 24-word seed phrase on stdin, got {} words. \
             Usage: zumbra wallet restore --birthday <height> < seed.txt",
            words
        ));
    }

    let ows_wallet = std::env::var("OWS_WALLET").unwrap_or_else(|_| "default".to_string());
    let ows_passphrase = std::env::var("OWS_PASSPHRASE").unwrap_or_default();
    let staging = prepare_restore(&cfg.data_dir, &ows_wallet, &ows_passphrase, &phrase, None)?;

    eprintln!("Restoring wallet from seed (birthday={})...", birthday);
    let restored = async {
        ensure_sapling_params(&cfg.data_dir).await?;
        zumbra_engine::wallet::restore(
            &cfg.data_dir,
            &cfg.server_url,
            cfg.network,
            &phrase,
            birthday,
            None,
            None,
        )
        .await
    }
    .await;
    if let Err(e) = restored {
        staging.rollback();
        return Err(anyhow::anyhow!(
            "Restore failed and nothing was kept: {:#}. Fix the cause (usually the server) and run restore again.",
            e
        ));
    }

    let addresses = zumbra_engine::query::get_addresses()
        .await
        .unwrap_or_default();
    let address = addresses
        .first()
        .map(|a| a.address.clone())
        .unwrap_or_default();

    zumbra_engine::wallet::close().await;

    eprintln!("Wallet restored.");
    eprintln!("  Address:    {}", address);
    eprintln!("  Birthday:   {}", birthday);
    eprintln!("  Data dir:   {}", cfg.data_dir);
    eprintln!("  OWS wallet: {} (encrypted at ~/.ows/wallets/)", ows_wallet);
    eprintln!("  Policy:     {}/policy.toml (default caps applied)", cfg.data_dir);

    Ok(())
}

pub async fn cmd_wallet_delete(cfg: &Config, confirm: bool) -> Result<()> {
    if !confirm {
        return Err(anyhow::anyhow!(
            "Pass --confirm to delete wallet data. This action is irreversible."
        ));
    }

    zumbra_engine::wallet::delete(&cfg.data_dir)?;

    print_ok("deleted", cfg.human, |_| {
        println!("Wallet data deleted from {}", cfg.data_dir);
    });
    Ok(())
}

pub async fn cmd_sync_start(cfg: &Config) -> Result<()> {
    auto_open(cfg).await?;
    zumbra_engine::sync::start().await?;

    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_clone = cancel.clone();
    ctrlc::set_handler(move || {
        cancel_clone.store(true, Ordering::SeqCst);
    })
    .ok();

    if cfg.human {
        eprintln!("Syncing... (Ctrl+C to stop)");
    }

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let p = zumbra_engine::sync::get_progress().await;

        if cfg.human {
            if let Some(ref err) = p.connection_error {
                eprintln!("  Connection error: {} (retrying...)", err);
            } else if p.latest_height > 0 {
                let pct = if p.latest_height > 0 {
                    (p.synced_height as f64 / p.latest_height as f64 * 100.0).min(100.0)
                } else {
                    0.0
                };
                eprint!(
                    "\r  {}/{} ({:.1}%)    ",
                    p.synced_height, p.latest_height, pct
                );
                io::stderr().flush().ok();
            }
        }

        if cancel.load(Ordering::SeqCst) {
            zumbra_engine::sync::stop().await;
            if cfg.human {
                eprintln!("\nSync stopped by user.");
            }
            break;
        }

        if !p.is_syncing && p.synced_height > 0 && p.synced_height >= p.latest_height {
            if cfg.human {
                eprintln!("\nFully synced at height {}.", p.synced_height);
            }
            break;
        }

        if !p.is_syncing && !zumbra_engine::sync::is_running() {
            if cfg.human {
                eprintln!("\nSync finished.");
            }
            break;
        }
    }

    let final_progress = zumbra_engine::sync::get_progress().await;
    if !cfg.human {
        print_ok(final_progress, false, |_| {});
    }

    zumbra_engine::wallet::close().await;
    Ok(())
}

pub async fn cmd_sync_status(cfg: &Config) -> Result<()> {
    auto_open(cfg).await?;

    let synced = zumbra_engine::query::get_synced_height().await?;
    let birthday = zumbra_engine::query::get_birthday().await?;

    #[derive(Serialize)]
    struct SyncStatus {
        synced_height: u32,
        birthday: u32,
    }

    let status = SyncStatus {
        synced_height: synced,
        birthday,
    };

    print_ok(status, cfg.human, |s| {
        println!("Synced height: {}", s.synced_height);
        println!("Birthday:      {}", s.birthday);
    });

    zumbra_engine::wallet::close().await;
    Ok(())
}

pub async fn cmd_balance(cfg: &Config) -> Result<()> {
    sync_if_needed(cfg).await?;
    let balance = zumbra_engine::query::get_wallet_balance().await?;

    print_ok(&balance, cfg.human, |b| {
        let total = b.sapling + b.orchard + b.transparent;
        let total_zec = total as f64 / 1e8;
        println!("Balance: {:.8} ZEC ({} zat)", total_zec, total);
        println!();
        println!("  Shielded (Orchard):  {} zat", b.orchard);
        println!("  Shielded (Sapling):  {} zat", b.sapling);
        println!("  Transparent:         {} zat", b.transparent);
        let pending = b.unconfirmed_sapling + b.unconfirmed_orchard + b.unconfirmed_transparent;
        if pending > 0 {
            println!();
            println!("  Pending:             {} zat", pending);
        }
    });

    zumbra_engine::wallet::close().await;
    Ok(())
}

pub async fn cmd_address(cfg: &Config) -> Result<()> {
    sync_if_needed(cfg).await?;
    let addresses = zumbra_engine::query::get_addresses().await?;

    print_ok(&addresses, cfg.human, |addrs| {
        if addrs.is_empty() {
            println!("No addresses found.");
        } else {
            for a in addrs.iter() {
                println!("{}", a.address);
                let pools: Vec<&str> = [
                    if a.has_orchard { Some("orchard") } else { None },
                    if a.has_sapling { Some("sapling") } else { None },
                    if a.has_transparent {
                        Some("transparent")
                    } else {
                        None
                    },
                ]
                .iter()
                .filter_map(|p| *p)
                .collect();
                println!("  pools: {}", pools.join(", "));
            }
        }
    });

    zumbra_engine::wallet::close().await;
    Ok(())
}

pub async fn cmd_keys(cfg: &Config) -> Result<()> {
    auto_open(cfg).await?;

    let ufvk = zumbra_engine::query::export_ufvk().await?;
    let uivk = zumbra_engine::query::export_uivk().await?;

    #[derive(Serialize)]
    struct KeysData {
        ufvk: Option<String>,
        uivk: Option<String>,
    }

    let data = KeysData {
        ufvk: ufvk.clone(),
        uivk: uivk.clone(),
    };

    print_ok(data, cfg.human, |d| {
        if let Some(ref k) = d.ufvk {
            println!("UFVK (Unified Full Viewing Key):");
            println!("  {}", k);
            println!();
        }
        if let Some(ref k) = d.uivk {
            println!("UIVK (Unified Incoming Viewing Key):");
            println!("  {}", k);
            println!();
            println!("The UIVK lets a payment processor detect incoming payments without spending rights.");
        }
        if d.ufvk.is_none() && d.uivk.is_none() {
            println!("No viewing keys found. Create a wallet first.");
        }
    });

    zumbra_engine::wallet::close().await;
    Ok(())
}

pub async fn cmd_transactions(cfg: &Config, limit: usize) -> Result<()> {
    auto_open(cfg).await?;
    let mut txs = zumbra_engine::query::get_transactions().await?;
    txs.truncate(limit);

    print_ok(&txs, cfg.human, |txs| {
        if txs.is_empty() {
            println!("No transactions.");
        } else {
            for tx in txs.iter() {
                let zec = tx.value as f64 / 1e8;
                let sign = if tx.value >= 0 { "+" } else { "" };
                println!(
                    "  {} {}{:.8} ZEC  h={}  {}",
                    &tx.txid[..12],
                    sign,
                    zec,
                    tx.height,
                    tx.kind,
                );
                if let Some(ref memo) = tx.memo {
                    println!("    memo: {}", memo);
                }
            }
        }
    });

    zumbra_engine::wallet::close().await;
    Ok(())
}

pub async fn cmd_send_propose(
    cfg: &Config,
    to: String,
    amount: u64,
    is_max: bool,
    memo: Option<String>,
    context_id: Option<String>,
    priority: bool,
) -> Result<()> {
    sync_if_needed(cfg).await?;
    let policy = spend_policy(&cfg.data_dir)?;

    // For max sends, we don't know the amount until the engine produces the proposal,
    // so we skip the policy daily-limit check here. (The rate-limit check still runs
    // in cmd_send_confirm.)
    if !is_max {
        let daily_spent = zumbra_engine::audit::daily_spent(&cfg.data_dir).unwrap_or(0);
        if let Err(violation) =
            zumbra_engine::policy::check_proposal(&policy, &to, amount, &context_id, daily_spent)
        {
            zumbra_engine::audit::log_event(
                &cfg.data_dir,
                "propose_send",
                Some(&to),
                Some(amount),
                None,
                context_id.as_deref(),
                None,
                Some(&violation.to_string()),
            )
            .ok();
            return Err(anyhow::anyhow!("{}", violation));
        }
    }

    auto_open(cfg).await?;

    let (send_amount, fee, _) =
        zumbra_engine::send::propose_send(&to, amount, memo.clone(), is_max, priority).await?;

    let pending = PendingProposal {
        address: to.clone(),
        amount: if is_max { send_amount } else { amount },
        memo: memo.clone(),
        is_max,
        context_id: context_id.clone(),
    };
    save_pending(&cfg.data_dir, &pending)?;

    zumbra_engine::audit::log_event(
        &cfg.data_dir,
        "propose_send",
        Some(&to),
        Some(send_amount),
        Some(fee),
        context_id.as_deref(),
        None,
        None,
    )
    .ok();

    #[derive(Serialize)]
    struct ProposalSummary {
        address: String,
        send_amount: u64,
        fee: u64,
        total: u64,
        send_amount_zec: f64,
        fee_zec: f64,
    }

    let summary = ProposalSummary {
        address: to.clone(),
        send_amount,
        fee,
        total: send_amount + fee,
        send_amount_zec: send_amount as f64 / 1e8,
        fee_zec: fee as f64 / 1e8,
    };

    print_ok(summary, cfg.human, |s| {
        println!("Proposal created:");
        println!("  To:     {}", s.address);
        println!(
            "  Amount: {:.8} ZEC ({} zat)",
            s.send_amount_zec, s.send_amount
        );
        println!("  Fee:    {:.8} ZEC ({} zat)", s.fee_zec, s.fee);
        println!("  Total:  {} zat", s.total);
        println!();
        println!("Run `zumbra send confirm` to sign and broadcast.");
    });

    zumbra_engine::wallet::close().await;
    Ok(())
}

/// The policy a spend path may use: the file must exist and parse. A missing or corrupt
/// policy refuses to spend; it never becomes "no limits".
pub fn spend_policy(data_dir: &str) -> Result<zumbra_engine::policy::SpendingPolicy> {
    zumbra_engine::policy::load_policy_checked(data_dir)
        .map_err(|e| anyhow::anyhow!("Spending policy unavailable, refusing to spend: {:#}", e))
}

/// The gate every CLI confirm passes through: the policy judges the amount the engine will
/// actually send (a `--max` proposal has none until now, and the pending file is writable by
/// the agent's OS user), and a refusal is logged. Only after that is the seed read.
pub fn seed_for_confirm(
    data_dir: &str,
    pending: &PendingProposal,
    send_amount: u64,
) -> Result<secrecy::SecretString> {
    let policy = spend_policy(data_dir)?;
    let daily_spent = zumbra_engine::audit::daily_spent(data_dir)?;
    if let Err(violation) = zumbra_engine::policy::check_proposal(
        &policy,
        &pending.address,
        send_amount,
        &pending.context_id,
        daily_spent,
    ) {
        zumbra_engine::audit::log_event(
            data_dir,
            "confirm_send",
            Some(&pending.address),
            Some(send_amount),
            None,
            pending.context_id.as_deref(),
            None,
            Some(&violation.to_string()),
        )
        .ok();
        return Err(anyhow::anyhow!("{}", violation));
    }
    read_seed()
}

pub async fn cmd_send_confirm(cfg: &Config) -> Result<()> {
    ensure_sapling_params(&cfg.data_dir).await?;
    sync_if_needed(cfg).await?;
    let pending = load_pending(&cfg.data_dir)?;

    let policy = spend_policy(&cfg.data_dir)?;
    if let Err(violation) = zumbra_engine::policy::check_rate_limit(&policy) {
        zumbra_engine::audit::log_event(
            &cfg.data_dir,
            "confirm_send",
            Some(&pending.address),
            Some(pending.amount),
            None,
            pending.context_id.as_deref(),
            None,
            Some(&violation.to_string()),
        )
        .ok();
        return Err(anyhow::anyhow!("{}", violation));
    }

    auto_open(cfg).await?;

    let (send_amount, fee, _) = zumbra_engine::send::propose_send(
        &pending.address,
        pending.amount,
        pending.memo.clone(),
        pending.is_max,
        false,
    )
    .await?;

    if cfg.human {
        let zec = send_amount as f64 / 1e8;
        let fee_zec = fee as f64 / 1e8;
        eprintln!(
            "Confirming: {:.8} ZEC + {:.8} fee to {}",
            zec, fee_zec, pending.address
        );
    }

    let seed = match seed_for_confirm(&cfg.data_dir, &pending, send_amount) {
        Ok(seed) => seed,
        Err(e) => {
            zumbra_engine::wallet::close().await;
            return Err(e);
        }
    };
    let txid = match zumbra_engine::send::confirm_send(&seed).await {
        Ok(txid) => {
            zumbra_engine::policy::record_confirm();
            zumbra_engine::audit::log_event(
                &cfg.data_dir,
                "confirm_send",
                Some(&pending.address),
                Some(send_amount),
                Some(fee),
                pending.context_id.as_deref(),
                Some(&txid),
                None,
            )
            .ok();
            txid
        }
        Err(e) => {
            zumbra_engine::audit::log_event(
                &cfg.data_dir,
                "confirm_send",
                Some(&pending.address),
                Some(send_amount),
                Some(fee),
                pending.context_id.as_deref(),
                None,
                Some(&format!("{:#}", e)),
            )
            .ok();
            return Err(e);
        }
    };

    delete_pending(&cfg.data_dir);

    #[derive(Serialize)]
    struct SendResult {
        txid: String,
        amount: u64,
        fee: u64,
        address: String,
    }

    print_ok(
        SendResult {
            txid: txid.clone(),
            amount: send_amount,
            fee,
            address: pending.address.clone(),
        },
        cfg.human,
        |r| {
            println!("Transaction broadcast.");
            println!("  txid: {}", r.txid);
        },
    );

    zumbra_engine::wallet::close().await;
    Ok(())
}

pub async fn cmd_send_max(cfg: &Config, to: String) -> Result<()> {
    auto_open(cfg).await?;
    let max = zumbra_engine::send::get_max_sendable(&to).await?;

    #[derive(Serialize)]
    struct MaxSendable {
        max_amount: u64,
        max_amount_zec: f64,
        address: String,
    }

    print_ok(
        MaxSendable {
            max_amount: max,
            max_amount_zec: max as f64 / 1e8,
            address: to.clone(),
        },
        cfg.human,
        |m| {
            println!(
                "Max sendable to {}: {:.8} ZEC ({} zat)",
                m.address, m.max_amount_zec, m.max_amount
            );
        },
    );

    zumbra_engine::wallet::close().await;
    Ok(())
}

pub async fn cmd_shield(cfg: &Config) -> Result<()> {
    ensure_sapling_params(&cfg.data_dir).await?;
    auto_open(cfg).await?;

    let seed = read_seed()?;
    let txid = zumbra_engine::send::shield_funds(&seed).await?;

    #[derive(Serialize)]
    struct ShieldResult {
        txid: String,
    }

    print_ok(ShieldResult { txid: txid.clone() }, cfg.human, |r| {
        println!("Shielding transaction broadcast.");
        println!("  txid: {}", r.txid);
    });

    zumbra_engine::wallet::close().await;
    Ok(())
}

pub async fn cmd_consolidate(cfg: &Config) -> Result<()> {
    ensure_sapling_params(&cfg.data_dir).await?;
    force_sync(cfg).await?;
    auto_open(cfg).await?;

    let addresses = zumbra_engine::query::get_addresses().await?;
    let own_addr = addresses
        .first()
        .map(|a| a.address.clone())
        .ok_or_else(|| anyhow::anyhow!("No address found — wallet may not be initialized"))?;

    if cfg.human {
        eprintln!("Consolidating shielded notes (send-to-self)...");
        eprintln!(
            "  Destination: {}...{}",
            &own_addr[..12],
            &own_addr[own_addr.len() - 8..]
        );
    }

    let (send_amount, fee, _) = zumbra_engine::send::propose_send(&own_addr, 0, None, true, false).await?;

    if cfg.human {
        eprintln!(
            "  Amount: {:.8} ZEC (max minus fee)",
            send_amount as f64 / 1e8
        );
        eprintln!("  Fee:    {} zat", fee);
    }

    let seed = read_seed()?;
    let txid = zumbra_engine::send::confirm_send(&seed).await?;

    #[derive(Serialize)]
    struct ConsolidateResult {
        txid: String,
        amount: u64,
        fee: u64,
    }

    print_ok(
        ConsolidateResult {
            txid: txid.clone(),
            amount: send_amount,
            fee,
        },
        cfg.human,
        |r| {
            println!("Notes consolidated successfully.");
            println!("  txid:   {}", r.txid);
            println!("  Amount: {:.8} ZEC", r.amount as f64 / 1e8);
            println!("  Fee:    {} zat", r.fee);
        },
    );

    zumbra_engine::wallet::close().await;
    Ok(())
}

// ---------------------------------------------------------------------------
// Ironwood pool transfer (ZIP 318)
// ---------------------------------------------------------------------------

pub async fn cmd_ironwood_plan(cfg: &Config) -> Result<()> {
    sync_if_needed(cfg).await?;
    let result = async {
        let seed = read_seed()?;
        let plan = zumbra_engine::ironwood_v2::plan(&seed).await?;
        print_ok(&plan, cfg.human, |p| {
            println!("Ironwood SDK transfer plan");
            println!("Amount: {} zat", p.total_migrating_zat);
            println!("Estimated fees: {} zat", p.estimated_total_fee_zat);
            println!("Transactions: {} preparation + {} transfer", p.prep_tx_count, p.transfer_tx_count);
            println!("Use the wallet app to review and execute this transfer.");
        });
        Ok(())
    }.await;
    zumbra_engine::wallet::close().await;
    result
}

pub async fn cmd_ironwood_status(cfg: &Config) -> Result<()> {
    auto_open(cfg).await?;
    let result = zumbra_engine::ironwood_v2::status().await;
    zumbra_engine::wallet::close().await;
    let report = result?;
    print_ok(&report, cfg.human, |r| {
        println!("Ironwood SDK transfer: {}", r.status);
        println!("Confirmed: {}/{} transactions", r.confirmed_count, r.total_tx_count);
        println!("Confirmed amount: {} zat", r.total_confirmed_zat);
        println!("Next due height: {}", r.next_due_height);
    });
    Ok(())
}

// The removed scheduler only wrote a JSON file. It did not drive the current
// SDK's commit/prove/broadcast loop. Do not report a transfer as running or
// paused while the wallet's database-backed SDK runner is unaffected.
pub async fn cmd_ironwood_confirm(_cfg: &Config, _tor: bool) -> Result<()> {
    Err(anyhow::anyhow!("Headless Ironwood execution is not connected to the SDK runner. Use the wallet app to review and execute the transfer; no legacy JSON schedule was created."))
}

pub async fn cmd_ironwood_pause(_cfg: &Config) -> Result<()> {
    Err(anyhow::anyhow!("Headless pause cannot control the SDK migration runner. Use the wallet app managing this transfer. Nothing was paused."))
}

pub async fn cmd_ironwood_resume(_cfg: &Config) -> Result<()> {
    Err(anyhow::anyhow!("Headless resume is not connected to the SDK migration runner. Use the wallet app managing this transfer. Nothing was resumed."))
}

#[cfg(test)]
mod confirm_gate_tests {
    use super::*;

    fn scratch(tag: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("zumbra-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// The amount the engine will actually send (not the one the agent typed, and not the one in
    /// a pending file the agent can edit) is what the policy judges, and it is judged before the
    /// seed is touched. A --max proposal has no amount until the engine produces one.
    #[test]
    fn confirm_checks_the_real_amount_against_policy_before_reading_the_seed() {
        let dir = scratch("confirm-gate");
        let data_dir = dir.to_str().unwrap();
        let mut policy = default_policy();
        policy.max_per_tx = 1_000;
        zumbra_engine::policy::save_policy(data_dir, &policy).unwrap();
        // If the seed were read first, this is the error we would see instead of the policy one.
        std::env::set_var("OWS_WALLET", format!("zumbra-no-such-wallet-{}", std::process::id()));
        let pending = PendingProposal {
            address: "utest1placeholder".into(),
            amount: 0,
            memo: None,
            is_max: true,
            context_id: None,
        };

        let err = seed_for_confirm(data_dir, &pending, 5_000).unwrap_err().to_string();

        assert!(err.contains("POLICY_EXCEEDED"), "expected the policy violation, got: {err}");
        assert!(!err.contains("OWS"), "the seed was read before the policy ran: {err}");
        let log = zumbra_engine::audit::query_log(data_dir, 10, None).unwrap();
        assert!(
            log.iter().any(|e| e.action == "confirm_send"
                && e.amount == Some(5_000)
                && e.error.as_deref().map_or(false, |m| m.contains("POLICY_EXCEEDED"))),
            "the refusal was not written to the audit log: {log:?}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn confirm_refuses_when_the_policy_file_is_corrupt_or_missing() {
        std::env::set_var("OWS_WALLET", format!("zumbra-no-such-wallet-{}", std::process::id()));
        let pending = PendingProposal { address: "utest1x".into(), amount: 1, memo: None, is_max: false, context_id: None };

        let missing = scratch("confirm-missing");
        let err = seed_for_confirm(missing.to_str().unwrap(), &pending, 1).unwrap_err().to_string();
        assert!(err.to_lowercase().contains("policy"), "missing policy should refuse, got: {err}");
        assert!(!err.contains("OWS"), "seed was read before the policy was checked: {err}");

        let corrupt = scratch("confirm-corrupt");
        std::fs::write(corrupt.join("policy.toml"), "max_per_tx = \"x\"\n").unwrap();
        let err = seed_for_confirm(corrupt.to_str().unwrap(), &pending, 1).unwrap_err().to_string();
        assert!(err.to_lowercase().contains("policy"), "corrupt policy should refuse, got: {err}");
        assert!(!err.contains("OWS"), "seed was read before the policy was checked: {err}");
        std::fs::remove_dir_all(&missing).ok();
        std::fs::remove_dir_all(&corrupt).ok();
    }
}

#[cfg(test)]
mod restore_tests {
    use super::*;

    fn scratch(tag: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("zumbra-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn prepare_restore_puts_the_seed_in_the_vault_and_writes_the_default_policy() {
        let dir = scratch("restore");
        let data_dir = dir.join("data");
        std::fs::create_dir_all(&data_dir).unwrap();
        let vault = dir.join("vault");
        let phrase = ows_lib::generate_mnemonic(24).unwrap();

        let _staging = prepare_restore(data_dir.to_str().unwrap(), "restore-test", "pass", &phrase, Some(&vault))
            .expect("prepare_restore");

        let exported = ows_lib::export_wallet("restore-test", Some("pass"), Some(&vault)).unwrap();
        assert_eq!(exported, phrase, "the vault must hold exactly the restored phrase");
        assert!(!data_dir.join(".seed").exists(), "a plaintext .seed file was written");
        assert!(
            std::fs::read_dir(&data_dir).unwrap().all(|e| {
                let name = e.unwrap().file_name();
                name == "policy.toml"
            }),
            "restore wrote something other than policy.toml into the data dir"
        );

        let policy = zumbra_engine::policy::load_policy_checked(data_dir.to_str().unwrap()).unwrap();
        let expected = default_policy();
        assert_eq!(policy.max_per_tx, expected.max_per_tx);
        assert_eq!(policy.daily_limit, expected.daily_limit);
        assert_eq!(policy.approval_threshold, expected.approval_threshold);
        assert!(expected.max_per_tx > 0 && expected.daily_limit > 0, "default policy must cap");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn prepare_restore_refuses_to_overwrite_an_existing_vault_wallet() {
        let dir = scratch("restore-dup");
        let data_dir = dir.join("data");
        std::fs::create_dir_all(&data_dir).unwrap();
        let vault = dir.join("vault");
        let first = ows_lib::generate_mnemonic(24).unwrap();
        let second = ows_lib::generate_mnemonic(24).unwrap();
        let dd = data_dir.to_str().unwrap();

        let _staging = prepare_restore(dd, "dup", "pass", &first, Some(&vault)).unwrap();
        let err = prepare_restore(dd, "dup", "pass", &second, Some(&vault)).unwrap_err();
        assert!(err.to_string().contains("dup"), "error should name the wallet: {err}");
        let kept = ows_lib::export_wallet("dup", Some("pass"), Some(&vault)).unwrap();
        assert_eq!(kept, first, "the existing vault wallet was replaced");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// An empty vault passphrase protects nothing. Both entry points refuse it unless the
    /// operator has set ZUMBRA_UNSAFE_EMPTY_PASSPHRASE=1 on purpose.
    #[test]
    fn init_and_restore_refuse_an_empty_passphrase() {
        let dir = scratch("empty-pass");
        let vault = dir.join("vault");
        let phrase = ows_lib::generate_mnemonic(24).unwrap();
        std::env::remove_var("ZUMBRA_UNSAFE_EMPTY_PASSPHRASE");

        let err = prepare_restore(dir.to_str().unwrap(), "empty", "", &phrase, Some(&vault)).unwrap_err().to_string();
        assert!(err.to_lowercase().contains("passphrase"), "restore accepted an empty passphrase: {err}");
        assert!(ows_lib::export_wallet("empty", Some(""), Some(&vault)).is_err(), "restore stored the seed anyway");

        let err = open_or_create_vault_seed("empty2", "", Some(&vault)).unwrap_err().to_string();
        assert!(err.to_lowercase().contains("passphrase"), "init accepted an empty passphrase: {err}");
        assert!(ows_lib::get_wallet("empty2", Some(&vault)).is_err(), "init created a vault wallet anyway");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// If the network step after staging fails, nothing may be left behind: otherwise the next
    /// `wallet init` finds the vault entry, "reuses" it, and a stale birthday hides the funds.
    #[test]
    fn a_failed_restore_leaves_no_vault_entry_and_no_policy_behind() {
        let dir = scratch("restore-rollback");
        let data_dir = dir.join("data");
        std::fs::create_dir_all(&data_dir).unwrap();
        let vault = dir.join("vault");
        let phrase = ows_lib::generate_mnemonic(24).unwrap();
        let dd = data_dir.to_str().unwrap();

        let staging = prepare_restore(dd, "rb", "pass", &phrase, Some(&vault)).unwrap();
        staging.rollback();

        assert!(ows_lib::export_wallet("rb", Some("pass"), Some(&vault)).is_err(), "vault entry survived the rollback");
        assert!(!data_dir.join("policy.toml").exists(), "policy.toml survived the rollback");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Init may print a seed it just generated, once. A seed that already lived in the vault is
    /// not init's to print.
    #[test]
    fn init_reports_whether_it_created_the_vault_seed() {
        let dir = scratch("init-reuse");
        let vault = dir.join("vault");

        let (first, created_first) = open_or_create_vault_seed("reuse", "pass", Some(&vault)).unwrap();
        let (second, created_second) = open_or_create_vault_seed("reuse", "pass", Some(&vault)).unwrap();

        assert!(created_first, "first call must create the vault seed");
        assert!(!created_second, "second call must report reuse");
        assert_eq!(first.expose_secret(), second.expose_secret());
        assert_eq!(first.expose_secret().split_whitespace().count(), 24);
        std::fs::remove_dir_all(&dir).ok();
    }
}
