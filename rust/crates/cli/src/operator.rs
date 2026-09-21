//! The operator's side of the approval channel. Both commands read the operator passphrase from
//! stdin, never from an argument or the environment, so it is typed by a person at a terminal.

use std::io::{self, BufRead as _, IsTerminal as _};

use anyhow::Result;

use crate::{print_ok, Config};

fn read_passphrase(prompt: &str) -> Result<zeroize::Zeroizing<String>> {
    if io::stdin().is_terminal() {
        eprintln!("{prompt}");
    }
    let mut line = zeroize::Zeroizing::new(String::new());
    io::stdin().lock().read_line(&mut line)?;
    Ok(zeroize::Zeroizing::new(line.trim_end_matches(['\r', '\n']).to_string()))
}

pub async fn cmd_operator_init(cfg: &Config) -> Result<()> {
    crate::ensure_data_dir(&cfg.data_dir)?;
    let pass = read_passphrase("Choose the operator passphrase (typed here, never stored, never put in an env var):")?;
    let again = read_passphrase("Type it again:")?;
    if *pass != *again {
        return Err(anyhow::anyhow!("the two passphrases differ; nothing was created"));
    }
    zumbra_engine::approval::operator_init(&cfg.data_dir, &pass)?;
    print_ok("operator key created", cfg.human, |_| {
        println!("Operator key created in {}.", cfg.data_dir);
        println!("Sends above the approval threshold now need `zumbra approve <proposal_id>` from this terminal.");
    });
    Ok(())
}

pub async fn cmd_approve(cfg: &Config, proposal_id: String, ttl_minutes: u64) -> Result<()> {
    let p = zumbra_engine::approval::load_proposal(&cfg.data_dir, &proposal_id)?;
    eprintln!("Approve this send?");
    eprintln!("  Proposal: {}", p.id);
    eprintln!("  To:       {}", p.address);
    eprintln!("  Amount:   {:.8} ZEC ({} zat)", p.amount as f64 / 1e8, p.amount);
    eprintln!("  Fee:      {:.8} ZEC ({} zat)", p.fee as f64 / 1e8, p.fee);
    eprintln!("  Valid:    {} minutes", ttl_minutes);
    let pass = read_passphrase("Operator passphrase to sign it (Enter with nothing to abort):")?;
    if pass.is_empty() {
        return Err(anyhow::anyhow!("aborted; nothing was approved"));
    }
    let now = zumbra_engine::approval::now_unix();
    zumbra_engine::approval::approve(&cfg.data_dir, &pass, &p, ttl_minutes * 60, now)?;
    #[derive(serde::Serialize)]
    struct Approved { proposal_id: String, expires_at: u64 }
    print_ok(Approved { proposal_id: p.id.clone(), expires_at: now + ttl_minutes * 60 }, cfg.human, |a| {
        println!("Approved {} for {} minutes. Confirm it now with `zumbra send confirm` or confirm_send.", a.proposal_id, ttl_minutes);
    });
    Ok(())
}
