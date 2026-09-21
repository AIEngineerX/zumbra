//! The CLI honours the ZUMBRA_* environment the docs, the Dockerfile and CI already set.

use std::process::Command;

fn zumbra() -> Command {
    Command::new(env!("CARGO_BIN_EXE_zumbra"))
}

#[test]
fn global_flags_advertise_their_environment_variables() {
    let out = zumbra().arg("--help").output().expect("run zumbra");
    let help = String::from_utf8_lossy(&out.stdout);
    for var in ["ZUMBRA_DATA_DIR", "ZUMBRA_TESTNET", "ZUMBRA_SERVER"] {
        assert!(help.contains(&format!("[env: {var}")), "--help does not mention {var}:\n{help}");
    }
}

#[test]
fn zumbra_testnet_env_selects_testnet() {
    let out = zumbra()
        .args(["info"])
        .env("ZUMBRA_TESTNET", "1")
        .env("ZUMBRA_DATA_DIR", std::env::temp_dir().join("zumbra-env-test"))
        .output()
        .expect("run zumbra info");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "info failed: {}", String::from_utf8_lossy(&out.stderr));
    assert!(text.to_lowercase().contains("testnet"), "ZUMBRA_TESTNET=1 did not select testnet:\n{text}");
}
