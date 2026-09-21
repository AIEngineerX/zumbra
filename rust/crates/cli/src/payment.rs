use std::io::{self, Read as _};

use anyhow::Result;
use serde::Serialize;
use zcash_protocol::consensus::Network;

use crate::helpers::*;
use crate::{print_ok, Config};

fn read_402_body(body: &Option<String>) -> Result<String> {
    if let Some(b) = body {
        return Ok(b.clone());
    }
    let mut buf = String::new();
    io::stdin().lock().read_to_string(&mut buf)?;
    let trimmed = buf.trim().to_string();
    if trimmed.is_empty() {
        return Err(anyhow::anyhow!(
            "No 402 body provided. Pass --body '<JSON>' or pipe via stdin."
        ));
    }
    Ok(trimmed)
}

fn expected_network(cfg: &Config) -> &'static str {
    if cfg.network == Network::TestNetwork {
        "zcash:testnet"
    } else {
        "zcash:mainnet"
    }
}

pub async fn cmd_x402_propose(
    cfg: &Config,
    body: Option<String>,
    context_id: Option<String>,
) -> Result<()> {
    let raw = read_402_body(&body)?;
    let req = zumbra_engine::x402::parse_402_response(&raw, expected_network(cfg))?;
    let amount = zumbra_engine::x402::amount_zatoshis(&req)?;

    crate::wallet::cmd_send_propose(cfg, req.pay_to, amount, false, None, context_id, false).await
}

pub async fn cmd_x402_pay(
    cfg: &Config,
    body: Option<String>,
    context_id: Option<String>,
) -> Result<()> {
    ensure_sapling_params(&cfg.data_dir).await?;
    sync_if_needed(cfg).await?;
    let raw = read_402_body(&body)?;
    let req = zumbra_engine::x402::parse_402_response(&raw, expected_network(cfg))?;
    let amount = zumbra_engine::x402::amount_zatoshis(&req)?;
    let address = req.pay_to.clone();

    let policy = crate::wallet::spend_policy(&cfg.data_dir)?;

    let daily_spent = zumbra_engine::audit::daily_spent(&cfg.data_dir).unwrap_or(0);
    if let Err(violation) =
        zumbra_engine::policy::check_proposal(&policy, &address, amount, &context_id, daily_spent)
    {
        zumbra_engine::audit::log_event(
            &cfg.data_dir,
            "x402_pay",
            Some(&address),
            Some(amount),
            None,
            context_id.as_deref(),
            None,
            Some(&violation.to_string()),
        )
        .ok();
        return Err(anyhow::anyhow!("{}", violation));
    }

    if let Err(violation) = zumbra_engine::policy::check_rate_limit(&policy) {
        zumbra_engine::audit::log_event(
            &cfg.data_dir,
            "x402_pay",
            Some(&address),
            Some(amount),
            None,
            context_id.as_deref(),
            None,
            Some(&violation.to_string()),
        )
        .ok();
        return Err(anyhow::anyhow!("{}", violation));
    }

    auto_open(cfg).await?;

    let (send_amount, fee, _) =
        zumbra_engine::send::propose_send(&address, amount, None, false, false).await?;

    if cfg.human {
        let zec = send_amount as f64 / 1e8;
        let fee_zec = fee as f64 / 1e8;
        eprintln!(
            "x402 payment: {:.8} ZEC + {:.8} fee to {}",
            zec, fee_zec, address
        );
    }

    let seed = read_seed(&cfg.data_dir)?;
    let txid = match zumbra_engine::send::confirm_send(&seed).await {
        Ok(txid) => {
            zumbra_engine::policy::record_confirm();
            zumbra_engine::audit::log_event(
                &cfg.data_dir,
                "x402_pay",
                Some(&address),
                Some(send_amount),
                Some(fee),
                context_id.as_deref(),
                Some(&txid),
                None,
            )
            .ok();
            txid
        }
        Err(e) => {
            zumbra_engine::audit::log_event(
                &cfg.data_dir,
                "x402_pay",
                Some(&address),
                Some(send_amount),
                Some(fee),
                context_id.as_deref(),
                None,
                Some(&format!("{:#}", e)),
            )
            .ok();
            return Err(e);
        }
    };

    delete_pending(&cfg.data_dir);

    let payment_signature = zumbra_engine::x402::build_payment_signature(&txid, &req);

    #[derive(Serialize)]
    struct X402PayResult {
        txid: String,
        payment_signature: String,
        amount: u64,
        fee: u64,
        address: String,
    }

    print_ok(
        X402PayResult {
            txid: txid.clone(),
            payment_signature: payment_signature.clone(),
            amount: send_amount,
            fee,
            address: address.clone(),
        },
        cfg.human,
        |r| {
            println!("x402 payment broadcast.");
            println!("  txid: {}", r.txid);
            println!(
                "  amount: {:.8} ZEC ({} zat)",
                r.amount as f64 / 1e8,
                r.amount
            );
            println!("  fee:    {:.8} ZEC ({} zat)", r.fee as f64 / 1e8, r.fee);
            println!();
            println!("PAYMENT-SIGNATURE header:");
            println!("  {}", r.payment_signature);
        },
    );

    zumbra_engine::wallet::close().await;
    Ok(())
}
