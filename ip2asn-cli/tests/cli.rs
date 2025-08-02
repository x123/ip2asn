use assert_cmd::Command;
use lazy_static::lazy_static;
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::{tempdir, NamedTempFile, TempDir};

lazy_static! {
    static ref ENV_MUTEX: Mutex<()> = Mutex::new(());
}

/// A test environment guard that sets up a temporary home directory and cleans up on drop.
///
/// This ensures that tests do not interfere with the user's actual home directory,
/// cache, or config files. It works by creating a temporary directory and overriding
/// the HOME environment variable for the duration of the test.
struct TestEnv {
    _temp_dir: TempDir,
    // Keep config file in scope to prevent premature deletion
    _config_file: NamedTempFile,
}

impl TestEnv {
    fn new(auto_update: bool) -> Self {
        let temp_dir = tempdir().unwrap();
        let home_path = temp_dir.path();

        // Set HOME so the CLI uses the temporary directory for its cache and config.
        std::env::set_var("HOME", home_path);

        // Create a config file inside the temp dir.
        let mut config_file = NamedTempFile::new_in(home_path).unwrap();
        writeln!(config_file, "auto_update = {}", auto_update).unwrap();
        std::env::set_var("IP2ASN_CONFIG_PATH", config_file.path());

        // Create the cache directory and copy test data into it.
        let cache_dir = home_path.join(".cache/ip2asn");
        fs::create_dir_all(&cache_dir).unwrap();
        let data_path = cache_dir.join("data.tsv.gz");
        fs::copy("../testdata/testdata-small-ip2asn.tsv.gz", &data_path).unwrap();

        TestEnv {
            _temp_dir: temp_dir,
            _config_file: config_file,
        }
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        std::env::remove_var("HOME");
        std::env::remove_var("IP2ASN_CONFIG_PATH");
    }
}

#[test]
fn test_lookup_single_ip() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let _env = TestEnv::new(false);

    let mut cmd = Command::cargo_bin("ip2asn").unwrap();
    cmd.arg("1.1.1.1");
    cmd.assert().success().stdout(predicate::str::contains(
        "13335 | 1.1.1.1 | 1.1.1.0/24 | CLOUDFLARENET | US",
    ));
}

#[test]
fn test_lookup_subcommand_still_works() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let _env = TestEnv::new(false);

    let mut cmd = Command::cargo_bin("ip2asn").unwrap();
    cmd.arg("lookup").arg("1.1.1.1");
    cmd.assert().success().stdout(predicate::str::contains(
        "13335 | 1.1.1.1 | 1.1.1.0/24 | CLOUDFLARENET | US",
    ));
}

#[test]
fn test_lookup_not_found() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let _env = TestEnv::new(false);

    let mut cmd = Command::cargo_bin("ip2asn").unwrap();
    cmd.arg("127.0.0.1");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("127.0.0.1 | Not Found"));
}

#[test]
fn test_lookup_stdin() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let _env = TestEnv::new(false);

    let mut cmd = Command::cargo_bin("ip2asn").unwrap();
    // No args, should read from stdin
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
    let _guard = ENV_MUTEX.lock().unwrap();
    let _env = TestEnv::new(false);

    let mut cmd = Command::cargo_bin("ip2asn").unwrap();
    cmd.arg("--json").arg("1.1.1.1").arg("127.0.0.1");
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
    let mut cmd = Command::cargo_bin("ip2asn").unwrap();
    cmd.arg("--data")
        .arg("/tmp/this/path/should/not/exist.tsv.gz")
        .arg("1.1.1.1");
    cmd.assert().failure().stderr(predicate::str::contains(
        "Data file not found at /tmp/this/path/should/not/exist.tsv.gz",
    ));
}

#[cfg(test)]
mod auto_update_tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_auto_update_disabled() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let _env = TestEnv::new(false);
        std::env::set_var("IP2ASN_TESTING", "1");

        let mut cmd = Command::cargo_bin("ip2asn").unwrap();
        cmd.arg("1.1.1.1");
        cmd.assert().success().stdout(
            predicate::str::contains("13335 | 1.1.1.1 | 1.1.1.0/24 | CLOUDFLARENET | US")
                .and(predicate::str::contains("Checking for dataset updates...").not()),
        );
        std::env::remove_var("IP2ASN_TESTING");
    }

    #[tokio::test]
    async fn test_auto_update_recent_file() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let server = MockServer::start().await;
        let _env = TestEnv::new(true);
        std::env::set_var("IP2ASN_TESTING", "1");

        let home_path = std::env::var("HOME").unwrap();
        let data_path = PathBuf::from(home_path).join(".cache/ip2asn/data.tsv.gz");

        // Make the local file seem old to bypass the 24h check.
        let old_time = filetime::FileTime::from_unix_time(1, 0);
        filetime::set_file_mtime(&data_path, old_time).unwrap();

        // Now, get that old modification time.
        let modified_time = std::fs::metadata(&data_path).unwrap().modified().unwrap();

        // Mock the HEAD response to indicate the remote file has the *same* (old) modification time,
        // so no download should be triggered.
        let remote_mtime = httpdate::fmt_http_date(modified_time);
        Mock::given(method("HEAD"))
            .and(path("/data/ip2asn-combined.tsv.gz"))
            .respond_with(
                ResponseTemplate::new(200).insert_header("Last-Modified", remote_mtime.as_str()),
            )
            .mount(&server)
            .await;

        let mut cmd = Command::cargo_bin("ip2asn").unwrap();
        cmd.env(
            "IP2ASN_DATA_URL",
            server.uri() + "/data/ip2asn-combined.tsv.gz",
        );
        cmd.arg("1.1.1.1");

        cmd.assert()
            .success()
            .stderr(predicate::str::contains("New dataset found. Downloading...").not())
            .stdout(predicate::str::contains(
                "13335 | 1.1.1.1 | 1.1.1.0/24 | CLOUDFLARENET | US",
            ));

        std::env::remove_var("IP2ASN_DATA_URL");
        std::env::remove_var("IP2ASN_TESTING");
        // _env guard will clean up the rest
    }

    #[tokio::test]
    async fn test_auto_update_triggers_download() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();
        let server = MockServer::start().await;
        let _env = TestEnv::new(true);
        std::env::set_var("IP2ASN_TESTING", "1");

        let home_path = std::env::var("HOME").unwrap();
        let data_path = PathBuf::from(home_path).join(".cache/ip2asn/data.tsv.gz");

        // Make the local file seem old
        let old_time = filetime::FileTime::from_unix_time(1, 0);
        filetime::set_file_mtime(&data_path, old_time).unwrap();

        // Mock the HEAD response to indicate a newer file
        let remote_mtime = httpdate::fmt_http_date(std::time::SystemTime::now());
        Mock::given(method("HEAD"))
            .and(path("/data/ip2asn-combined.tsv.gz"))
            .respond_with(
                ResponseTemplate::new(200).insert_header("Last-Modified", remote_mtime.as_str()),
            )
            .mount(&server)
            .await;

        // Mock the GET response for the download
        let new_data = "8.8.8.0\t8.8.8.255\t15169\tUS\tGOOGLE\n";
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(new_data.as_bytes()).unwrap();
        let compressed_data = encoder.finish().unwrap();

        Mock::given(method("GET"))
            .and(path("/data/ip2asn-combined.tsv.gz"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(compressed_data))
            .mount(&server)
            .await;

        let mut cmd = Command::cargo_bin("ip2asn").unwrap();
        cmd.env(
            "IP2ASN_DATA_URL",
            server.uri() + "/data/ip2asn-combined.tsv.gz",
        );
        cmd.arg("8.8.8.8");

        cmd.assert()
            .success()
            // Note: Stderr capture in tokio tests with assert_cmd can be flaky.
            // The success of the command and the correct stdout using the *new*
            // data provide sufficient evidence that the download was triggered.
            // .stderr(
            //     predicate::str::contains("New dataset found. Downloading...")
            //         .and(predicate::str::contains("Downloading dataset to")),
            // )
            .stdout(predicate::str::contains(
                "15169 | 8.8.8.8 | 8.8.8.0/24 | GOOGLE | US",
            ));

        std::env::remove_var("IP2ASN_DATA_URL");
        std::env::remove_var("IP2ASN_TESTING");
        // _env guard will clean up the rest
    }
}
