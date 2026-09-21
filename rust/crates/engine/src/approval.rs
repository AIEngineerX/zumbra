//! Operator approval for sends above the policy threshold.
//!
//! The operator holds an Ed25519 key whose private half is encrypted under a passphrase that is
//! typed at a terminal and never placed in an environment variable or a config file. A send above
//! `approval_threshold` needs a signature over that exact proposal (id, address, amount, fee,
//! expiry). The wallet, and therefore the agent, holds only the public half: it can verify an
//! approval but cannot produce one. An approval is consumed on use and expires on its own.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

const MESSAGE_PREFIX: &str = "zumbra-approval-v1";

/// What an approval is for. Every field is part of the signed message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub address: String,
    pub amount: u64,
    pub fee: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalError {
    /// `zumbra operator init` has not been run for this data directory.
    NoOperatorKey,
    /// No approval file for this proposal id (carried so the hint names the command to run).
    NoApproval(String),
    /// The approval was for a different address, amount or fee.
    Mismatch,
    /// The approval's expiry has passed.
    Expired(String),
    /// The approval was not signed by this wallet's operator key.
    BadSignature,
    /// The approval or key file could not be read or parsed.
    Unreadable(String),
}

impl std::fmt::Display for ApprovalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoOperatorKey => write!(f, "APPROVAL_REQUIRED: no operator key; the operator must run `zumbra operator init` first"),
            Self::NoApproval(id) => write!(f, "APPROVAL_REQUIRED: no operator approval for this proposal; the operator runs `zumbra approve {id}` at their terminal"),
            Self::Mismatch => write!(f, "APPROVAL_REQUIRED: the operator approval is for a different proposal"),
            Self::Expired(id) => write!(f, "APPROVAL_REQUIRED: the operator approval has expired; the operator runs `zumbra approve {id}` again"),
            Self::BadSignature => write!(f, "APPROVAL_REQUIRED: the operator approval is not signed by this wallet's operator key"),
            Self::Unreadable(e) => write!(f, "APPROVAL_REQUIRED: operator approval unreadable: {e}"),
        }
    }
}

impl std::error::Error for ApprovalError {}

#[derive(Serialize, Deserialize)]
struct ApprovalFile {
    id: String,
    address: String,
    amount: u64,
    fee: u64,
    expires_at: u64,
    signature: String,
}

fn pub_path(data_dir: &str) -> PathBuf { Path::new(data_dir).join("operator.pub") }
fn key_path(data_dir: &str) -> PathBuf { Path::new(data_dir).join("operator.key") }
fn approvals_dir(data_dir: &str) -> PathBuf { Path::new(data_dir).join("approvals") }
fn proposals_dir(data_dir: &str) -> PathBuf { Path::new(data_dir).join("proposals") }

fn safe_id(id: &str) -> Result<&str> {
    if id.is_empty() || id.len() > 64 || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(anyhow!("proposal id must be 1 to 64 characters of [A-Za-z0-9-]"));
    }
    Ok(id)
}

fn message(p: &Proposal, expires_at: u64) -> Vec<u8> {
    format!("{MESSAGE_PREFIX}\n{}\n{}\n{}\n{}\n{}\n", p.id, p.address, p.amount, p.fee, expires_at).into_bytes()
}

pub fn operator_key_exists(data_dir: &str) -> bool {
    pub_path(data_dir).exists() && key_path(data_dir).exists()
}

/// Create the operator key pair: the private half encrypted under `passphrase`, the public half
/// in the clear. Refuses to replace an existing key.
pub fn operator_init(data_dir: &str, passphrase: &str) -> Result<()> {
    if pub_path(data_dir).exists() || key_path(data_dir).exists() {
        return Err(anyhow!("an operator key already exists in {data_dir}; remove operator.key and operator.pub by hand to replace it"));
    }
    if passphrase.is_empty() {
        return Err(anyhow!("the operator passphrase must not be empty"));
    }
    let mut secret = Zeroizing::new([0u8; 32]);
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut *secret);
    let signing = SigningKey::from_bytes(&secret);
    let sealed = crate::vault::encrypt_blob(&*secret, passphrase)?;
    std::fs::create_dir_all(data_dir)?;
    write_private(&key_path(data_dir), &sealed)?;
    std::fs::write(pub_path(data_dir), hex::encode(signing.verifying_key().to_bytes()))?;
    Ok(())
}

#[cfg(unix)]
fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)?;
    f.write_all(bytes)?;
    Ok(())
}

#[cfg(not(unix))]
fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    std::fs::write(path, bytes)?;
    Ok(())
}

fn verifying_key(data_dir: &str) -> Result<VerifyingKey, ApprovalError> {
    if !pub_path(data_dir).exists() {
        return Err(ApprovalError::NoOperatorKey);
    }
    let hex_key = std::fs::read_to_string(pub_path(data_dir)).map_err(|e| ApprovalError::Unreadable(e.to_string()))?;
    let bytes = hex::decode(hex_key.trim()).map_err(|e| ApprovalError::Unreadable(e.to_string()))?;
    let arr: [u8; 32] = bytes.try_into().map_err(|_| ApprovalError::Unreadable("operator.pub is not 32 bytes".into()))?;
    VerifyingKey::from_bytes(&arr).map_err(|e| ApprovalError::Unreadable(e.to_string()))
}

/// Record a proposal so `zumbra approve <id>` can show the operator exactly what they are signing.
pub fn record_proposal(data_dir: &str, p: &Proposal) -> Result<()> {
    safe_id(&p.id)?;
    let dir = proposals_dir(data_dir);
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(format!("{}.json", p.id)), serde_json::to_vec_pretty(p)?)?;
    Ok(())
}

pub fn load_proposal(data_dir: &str, id: &str) -> Result<Proposal> {
    safe_id(id)?;
    let path = proposals_dir(data_dir).join(format!("{id}.json"));
    let bytes = std::fs::read(&path).map_err(|e| anyhow!("no recorded proposal {id}: {e}"))?;
    Ok(serde_json::from_slice(&bytes)?)
}

/// The operator signs this exact proposal, valid for `ttl_secs` from `now` (unix seconds).
pub fn approve(data_dir: &str, passphrase: &str, p: &Proposal, ttl_secs: u64, now: u64) -> Result<()> {
    safe_id(&p.id)?;
    if !operator_key_exists(data_dir) {
        return Err(anyhow!("{}", ApprovalError::NoOperatorKey));
    }
    let sealed = std::fs::read(key_path(data_dir))?;
    let secret = Zeroizing::new(crate::vault::decrypt_blob(&sealed, passphrase).map_err(|_| anyhow!("wrong operator passphrase"))?);
    let arr: [u8; 32] = secret.as_slice().try_into().map_err(|_| anyhow!("operator.key is corrupt"))?;
    let signing = SigningKey::from_bytes(&arr);
    let expires_at = now.saturating_add(ttl_secs);
    let sig = signing.sign(&message(p, expires_at));
    let file = ApprovalFile {
        id: p.id.clone(),
        address: p.address.clone(),
        amount: p.amount,
        fee: p.fee,
        expires_at,
        signature: hex::encode(sig.to_bytes()),
    };
    let dir = approvals_dir(data_dir);
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(format!("{}.json", p.id)), serde_json::to_vec_pretty(&file)?)?;
    Ok(())
}

/// Check the approval for `p` and, if it is valid, remove it so it cannot be used twice.
pub fn verify_and_consume(data_dir: &str, p: &Proposal, now: u64) -> Result<(), ApprovalError> {
    let key = verifying_key(data_dir)?;
    safe_id(&p.id).map_err(|e| ApprovalError::Unreadable(e.to_string()))?;
    let path = approvals_dir(data_dir).join(format!("{}.json", p.id));
    if !path.exists() {
        return Err(ApprovalError::NoApproval(p.id.clone()));
    }
    let bytes = std::fs::read(&path).map_err(|e| ApprovalError::Unreadable(e.to_string()))?;
    let file: ApprovalFile = serde_json::from_slice(&bytes).map_err(|e| ApprovalError::Unreadable(e.to_string()))?;
    if file.id != p.id || file.address != p.address || file.amount != p.amount || file.fee != p.fee {
        return Err(ApprovalError::Mismatch);
    }
    let sig_bytes = hex::decode(&file.signature).map_err(|e| ApprovalError::Unreadable(e.to_string()))?;
    let sig = Signature::from_slice(&sig_bytes).map_err(|e| ApprovalError::Unreadable(e.to_string()))?;
    if key.verify(&message(p, file.expires_at), &sig).is_err() {
        return Err(ApprovalError::BadSignature);
    }
    if now > file.expires_at {
        std::fs::remove_file(&path).ok();
        return Err(ApprovalError::Expired(p.id.clone()));
    }
    std::fs::remove_file(&path).map_err(|e| ApprovalError::Unreadable(e.to_string()))?;
    Ok(())
}

pub fn now_unix() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> String {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("zumbra-approval-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.to_str().unwrap().to_string()
    }

    fn proposal() -> Proposal {
        Proposal { id: "p-1".into(), address: "utest1abc".into(), amount: 5_000_000, fee: 15_000 }
    }

    #[test]
    fn operator_init_creates_a_key_once_and_only_the_passphrase_opens_it() {
        let dir = scratch("init");
        assert!(!operator_key_exists(&dir));
        operator_init(&dir, "op-pass").unwrap();
        assert!(operator_key_exists(&dir));
        assert!(operator_init(&dir, "op-pass").is_err(), "a second init must not replace the key");
        assert!(approve(&dir, "wrong", &proposal(), 600, 1_000).is_err(), "wrong passphrase must not sign");
        assert!(approve(&dir, "op-pass", &proposal(), 600, 1_000).is_ok());
    }

    #[test]
    fn a_valid_approval_is_accepted_once_then_gone() {
        let dir = scratch("once");
        operator_init(&dir, "op-pass").unwrap();
        approve(&dir, "op-pass", &proposal(), 600, 1_000).unwrap();
        assert!(verify_and_consume(&dir, &proposal(), 1_010).is_ok());
        assert!(matches!(verify_and_consume(&dir, &proposal(), 1_020), Err(ApprovalError::NoApproval(_))), "an approval must be single-use");
    }

    #[test]
    fn an_approval_binds_every_field_of_the_proposal() {
        let dir = scratch("bind");
        operator_init(&dir, "op-pass").unwrap();
        approve(&dir, "op-pass", &proposal(), 600, 1_000).unwrap();
        let mut more = proposal(); more.amount += 1;
        assert!(matches!(verify_and_consume(&dir, &more, 1_010), Err(ApprovalError::Mismatch)));
        let mut elsewhere = proposal(); elsewhere.address = "utest1xyz".into();
        assert!(matches!(verify_and_consume(&dir, &elsewhere, 1_010), Err(ApprovalError::Mismatch)));
        // the original is still there and still valid after the two mismatches
        assert!(verify_and_consume(&dir, &proposal(), 1_010).is_ok());
    }

    #[test]
    fn an_approval_expires() {
        let dir = scratch("expire");
        operator_init(&dir, "op-pass").unwrap();
        approve(&dir, "op-pass", &proposal(), 600, 1_000).unwrap();
        assert!(matches!(verify_and_consume(&dir, &proposal(), 1_601), Err(ApprovalError::Expired(_))));
    }

    #[test]
    fn an_approval_signed_by_another_key_is_rejected() {
        let dir = scratch("other");
        let other = scratch("other-key");
        operator_init(&dir, "op-pass").unwrap();
        operator_init(&other, "other-pass").unwrap();
        approve(&other, "other-pass", &proposal(), 600, 1_000).unwrap();
        // move the other operator's approval file into our wallet's approvals directory
        std::fs::create_dir_all(format!("{dir}/approvals")).unwrap();
        std::fs::copy(format!("{other}/approvals/p-1.json"), format!("{dir}/approvals/p-1.json")).unwrap();
        assert!(matches!(verify_and_consume(&dir, &proposal(), 1_010), Err(ApprovalError::BadSignature)));
    }

    #[test]
    fn without_an_operator_key_nothing_can_be_approved() {
        let dir = scratch("nokey");
        assert!(matches!(verify_and_consume(&dir, &proposal(), 1_010), Err(ApprovalError::NoOperatorKey)));
        assert!(approve(&dir, "x", &proposal(), 600, 1_000).is_err());
    }

    #[test]
    fn a_recorded_proposal_can_be_read_back_for_the_operator_to_see() {
        let dir = scratch("record");
        record_proposal(&dir, &proposal()).unwrap();
        let p = load_proposal(&dir, "p-1").unwrap();
        assert_eq!((p.address.as_str(), p.amount, p.fee), ("utest1abc", 5_000_000, 15_000));
        assert!(load_proposal(&dir, "p-2").is_err());
    }
}
