//! The restore command must never take the seed phrase on the command line: argv is visible
//! to every process on the machine and lands in shell history.

use std::process::Command;

#[test]
fn restore_takes_the_seed_from_stdin_never_from_argv() {
    let out = Command::new(env!("CARGO_BIN_EXE_zumbra"))
        .args(["wallet", "restore", "--help"])
        .output()
        .expect("run zumbra");
    let help = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "help exited non-zero: {help}");
    assert!(!help.contains("--seed"), "restore still offers --seed on argv:\n{help}");
    assert!(
        help.to_lowercase().contains("stdin"),
        "restore help does not say the seed is read from stdin:\n{help}"
    );
}
