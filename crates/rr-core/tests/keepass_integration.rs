use std::io::Write;
use std::process::{Command, Stdio};

const TEST_PASSWORD: &str = "rr-integration-test-password-2026";
const TEST_ENTRY_NAME: &str = "rr-integration-identity";
const TEST_NSEC: &str = "nsec1p0zr6ued0prcss88z36q7kznjnlny6v3q5ykj4wx2e7wkym6uykq4jhpc";

fn skip_unless_env() -> bool {
    if std::env::var("RR_KDBX_TEST").is_err() {
        eprintln!("SKIP: set RR_KDBX_TEST=1 to run");
        return true;
    }
    false
}

fn keepassxc_available() -> bool {
    Command::new("which")
        .arg("keepassxc-cli")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn skip_unless_keepassxc() -> bool {
    if !keepassxc_available() {
        eprintln!("SKIP: keepassxc-cli not found in PATH");
        return true;
    }
    false
}

fn setup_kdbx(db_path: &str) {
    let mut child = Command::new("keepassxc-cli")
        .args(["db-create", "-p"])
        .arg(db_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("keepassxc-cli db-create failed");
    writeln!(
        child.stdin.take().unwrap(),
        "{}\n{}",
        TEST_PASSWORD,
        TEST_PASSWORD
    )
    .ok();
    assert!(child.wait().unwrap().success(), "db-create failed");

    let mut child = Command::new("keepassxc-cli")
        .args(["add", "-q", "-p", db_path, TEST_ENTRY_NAME])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("keepassxc-cli add failed");
    writeln!(
        child.stdin.take().unwrap(),
        "{}\n{}\n{}",
        TEST_PASSWORD,
        TEST_NSEC,
        TEST_NSEC
    )
    .ok();
    assert!(child.wait().unwrap().success(), "add entry failed");
}

fn db_path_1() -> String {
    format!("/tmp/rr-integration-{}.kdbx", std::process::id())
}
fn db_path_2() -> String {
    format!("/tmp/rr-integration-{}-2.kdbx", std::process::id())
}
fn db_path_3() -> String {
    format!("/tmp/rr-integration-{}-3.kdbx", std::process::id())
}

#[test]
fn test_keepassxc_cli_read_entry() {
    if skip_unless_env() || skip_unless_keepassxc() {
        return;
    }

    let db_path = db_path_1();
    setup_kdbx(&db_path);

    let out = Command::new("keepassxc-cli")
        .args([
            "show",
            "-q",
            "-s",
            "-a",
            "Password",
            &db_path,
            TEST_ENTRY_NAME,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("keepassxc-cli show failed");
    writeln!(out.stdin.as_ref().unwrap(), "{}", TEST_PASSWORD).ok();
    let output = out.wait_with_output().unwrap();
    assert!(output.status.success(), "show failed");
    let nsec = String::from_utf8(output.stdout).unwrap().trim().to_string();
    assert_eq!(nsec, TEST_NSEC, "keepassxc-cli returned wrong nsec");

    std::fs::remove_file(&db_path).ok();
}

#[test]
fn test_keepass_rs_read_entry() {
    if skip_unless_env() || skip_unless_keepassxc() {
        return;
    }

    let db_path = db_path_2();
    setup_kdbx(&db_path);

    let mut file = std::fs::File::open(&db_path).expect("open kdbx");
    let key = keepass::DatabaseKey::new().with_password(TEST_PASSWORD);
    let database = keepass::Database::open(&mut file, key).expect("open database");
    let mut found = false;
    for entry_ref in database.root().entries() {
        let title = entry_ref.get_title().unwrap_or("");
        if title == TEST_ENTRY_NAME {
            if let Some(pwd) = entry_ref.get_password() {
                assert_eq!(pwd, TEST_NSEC, "keepass-rs returned wrong nsec");
                found = true;
            }
        }
    }
    assert!(
        found,
        "entry '{}' not found via keepass-rs",
        TEST_ENTRY_NAME
    );

    std::fs::remove_file(&db_path).ok();
}

#[test]
fn test_both_backends_match() {
    if skip_unless_env() || skip_unless_keepassxc() {
        return;
    }

    let db_path = db_path_3();
    setup_kdbx(&db_path);

    let out = Command::new("keepassxc-cli")
        .args([
            "show",
            "-q",
            "-s",
            "-a",
            "Password",
            &db_path,
            TEST_ENTRY_NAME,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("keepassxc-cli show");
    writeln!(out.stdin.as_ref().unwrap(), "{}", TEST_PASSWORD).ok();
    let cli_output = out.wait_with_output().unwrap();
    let cli_nsec = String::from_utf8(cli_output.stdout)
        .unwrap()
        .trim()
        .to_string();

    let mut file = std::fs::File::open(&db_path).expect("open kdbx");
    let key = keepass::DatabaseKey::new().with_password(TEST_PASSWORD);
    let database = keepass::Database::open(&mut file, key).expect("open database");
    let rs_nsec = database
        .root()
        .entries()
        .find_map(|e| {
            let title = e.get_title().unwrap_or("");
            if title == TEST_ENTRY_NAME {
                e.get_password().map(|p| p.to_string())
            } else {
                None
            }
        })
        .expect("entry not found via keepass-rs");

    assert_eq!(cli_nsec, rs_nsec, "backends returned different nsec values");
    assert_eq!(cli_nsec, TEST_NSEC);

    std::fs::remove_file(&db_path).ok();
}
