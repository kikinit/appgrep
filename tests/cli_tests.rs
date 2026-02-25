use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_exits_0_and_contains_list() {
    Command::cargo_bin("appgrep")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("list"));
}

#[test]
fn test_version_exits_0() {
    Command::cargo_bin("appgrep")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("appgrep"));
}

#[test]
fn test_list_exits_0() {
    Command::cargo_bin("appgrep")
        .unwrap()
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_list_json_outputs_valid_json_array() {
    let output = Command::cargo_bin("appgrep")
        .unwrap()
        .args(["--format", "json", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn test_has_nonexistent_exits_1() {
    Command::cargo_bin("appgrep")
        .unwrap()
        .args(["has", "nonexistent_app_xyz_12345"])
        .assert()
        .code(1);
}

#[test]
fn test_info_nonexistent_exits_1() {
    Command::cargo_bin("appgrep")
        .unwrap()
        .args(["info", "nonexistent_app_xyz_12345"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_path_nonexistent_exits_1() {
    Command::cargo_bin("appgrep")
        .unwrap()
        .args(["path", "nonexistent_app_xyz_12345"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_search_exits_0() {
    Command::cargo_bin("appgrep")
        .unwrap()
        .args(["search", "nonexistent_app_xyz_12345"])
        .assert()
        .success();
}

#[test]
fn test_list_with_source_filter() {
    Command::cargo_bin("appgrep")
        .unwrap()
        .args(["--source", "desktop", "list"])
        .assert()
        .success();
}

#[test]
fn test_list_tsv_format() {
    let output = Command::cargo_bin("appgrep")
        .unwrap()
        .args(["--format", "tsv", "list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // TSV always has a header line
    assert!(stdout.starts_with("name\texec\tsource\tdescription"));
}

#[test]
fn test_list_names_format() {
    Command::cargo_bin("appgrep")
        .unwrap()
        .args(["--format", "names", "list"])
        .assert()
        .success();
}

#[test]
fn test_list_exec_format() {
    Command::cargo_bin("appgrep")
        .unwrap()
        .args(["--format", "exec", "list"])
        .assert()
        .success();
}

#[test]
fn test_has_json_nonexistent() {
    let output = Command::cargo_bin("appgrep")
        .unwrap()
        .args(["--format", "json", "has", "nonexistent_app_xyz_12345"])
        .output()
        .unwrap();

    // Exit code 1 for not found
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(parsed["found"], false);
}

#[test]
fn test_no_color_flag() {
    Command::cargo_bin("appgrep")
        .unwrap()
        .args(["--no-color", "list"])
        .assert()
        .success();
}
