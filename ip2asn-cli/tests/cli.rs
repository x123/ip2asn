use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

fn setup_test_data() -> PathBuf {
    let dirs = directories::ProjectDirs::from("io", "github", "x123").unwrap();
    let cache_dir = dirs.cache_dir().join("ip2asn");
    fs::create_dir_all(&cache_dir).unwrap();
    let data_path = cache_dir.join("data.tsv.gz");
    fs::copy("../testdata/testdata-small-ip2asn.tsv.gz", &data_path).unwrap();
    data_path
}

#[test]
fn test_lookup_single_ip() {
    setup_test_data();
    let mut cmd = Command::cargo_bin("ip2asn-cli").unwrap();
    cmd.arg("lookup").arg("1.1.1.1");
    cmd.assert().success().stdout(predicate::str::contains(
        "13335 | 1.1.1.1 | 1.1.1.0/24 | CLOUDFLARENET | US",
    ));
}

#[test]
fn test_lookup_not_found() {
    setup_test_data();
    let mut cmd = Command::cargo_bin("ip2asn-cli").unwrap();
    cmd.arg("lookup").arg("127.0.0.1");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("127.0.0.1 | Not Found"));
}

#[test]
fn test_lookup_stdin() {
    setup_test_data();
    let mut cmd = Command::cargo_bin("ip2asn-cli").unwrap();
    cmd.arg("lookup");
    cmd.write_stdin("8.8.8.8\n1.1.1.1\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "15169 | 8.8.8.8 | 8.8.8.0/24 | GOOGLE | US",
        ))
        .stdout(predicate::str::contains(
            "13335 | 1.1.1.1 | 1.1.1.0/24 | CLOUDFLARENET | US",
        ));
}

#[test]
fn test_lookup_json_output() {
    setup_test_data();
    let mut cmd = Command::cargo_bin("ip2asn-cli").unwrap();
    cmd.arg("lookup")
        .arg("--json")
        .arg("1.1.1.1")
        .arg("127.0.0.1");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            r#"{"ip":"1.1.1.1","found":true,"info":{"network":"1.1.1.0/24","asn":13335,"country_code":"US","organization":"CLOUDFLARENET"}}"#,
        ))
        .stdout(predicate::str::contains(
            r#"{"ip":"127.0.0.1","found":false,"info":null}"#,
        ));
}

#[test]
fn test_lookup_data_file_not_found() {
    let mut cmd = Command::cargo_bin("ip2asn-cli").unwrap();
    cmd.arg("lookup")
        .arg("--data")
        .arg("/tmp/this/path/should/not/exist.tsv.gz")
        .arg("1.1.1.1");
    cmd.assert().failure().stderr(predicate::str::contains(
        "Data file not found at /tmp/this/path/should/not/exist.tsv.gz",
    ));
}
