use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_lookup_single_ip() {
    let mut cmd = Command::cargo_bin("ip2asn-cli").unwrap();
    cmd.arg("--data")
        .arg("../testdata/testdata-small-ip2asn.tsv.gz")
        .arg("1.1.1.1");
    cmd.assert().success().stdout(predicate::str::contains(
        "AS13335 | 1.1.1.1 | 1.1.1.0/24 | CLOUDFLARENET | US",
    ));
}

#[test]
fn test_lookup_not_found() {
    let mut cmd = Command::cargo_bin("ip2asn-cli").unwrap();
    cmd.arg("--data")
        .arg("../testdata/testdata-small-ip2asn.tsv.gz")
        .arg("127.0.0.1");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("127.0.0.1 | Not Found"));
}

#[test]
fn test_lookup_stdin() {
    let mut cmd = Command::cargo_bin("ip2asn-cli").unwrap();
    cmd.arg("--data")
        .arg("../testdata/testdata-small-ip2asn.tsv.gz");
    cmd.write_stdin("8.8.8.8\n1.1.1.1\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "AS15169 | 8.8.8.8 | 8.8.8.0/24 | GOOGLE | US",
        ))
        .stdout(predicate::str::contains(
            "AS13335 | 1.1.1.1 | 1.1.1.0/24 | CLOUDFLARENET | US",
        ));
}

#[test]
fn test_lookup_json_output() {
    let mut cmd = Command::cargo_bin("ip2asn-cli").unwrap();
    cmd.arg("--data")
        .arg("../testdata/testdata-small-ip2asn.tsv.gz")
        .arg("--json")
        .arg("1.1.1.1")
        .arg("127.0.0.1");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            r#"{"ip":"1.1.1.1","found":true,"asn":13335,"network":"1.1.1.0/24","organization":"CLOUDFLARENET","country_code":"US"}"#,
        ))
        .stdout(predicate::str::contains(
            r#"{"ip":"127.0.0.1","found":false}"#,
        ));
}
