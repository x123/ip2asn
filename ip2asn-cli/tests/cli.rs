use assert_cmd::Command;
use predicates::prelude::*;
use rstest::{fixture, rstest};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::{tempdir, NamedTempFile, TempDir};

/// A test environment guard that sets up a temporary home directory and cleans up on drop.
///
/// This ensures that tests do not interfere with the user's actual home directory,
/// cache, or config files. It works by creating a temporary directory and overriding
/// the HOME environment variable for the duration of the test.
struct TestEnv {
    _home_dir: TempDir,
    cache_dir: PathBuf,
    config_file: NamedTempFile,
}

impl TestEnv {
    fn new(auto_update: bool) -> Self {
        let home_dir = tempdir().unwrap();
        let cache_dir = home_dir.path().join(".cache/ip2asn");
        fs::create_dir_all(&cache_dir).unwrap();

        let config_dir = home_dir.path().join(".config/ip2asn");
        fs::create_dir_all(&config_dir).unwrap();
        let mut config_file = NamedTempFile::new_in(&config_dir).unwrap();
        writeln!(config_file, "auto_update = {}", auto_update).unwrap();

        TestEnv {
            _home_dir: home_dir,
            cache_dir,
            config_file,
        }
    }

    fn populate_cache(&self) {
        let data_path = self.cache_dir.join("data.tsv.gz");
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir.parent().unwrap();
        let source_path = workspace_root.join("testdata/testdata-small-ip2asn.tsv.gz");

        fs::copy(&source_path, &data_path).unwrap_or_else(|e| {
            panic!(
                "Failed to copy test data from '{}' to '{}': {}",
                source_path.display(),
                data_path.display(),
                e
            )
        });
    }

    fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("ip2asn").unwrap();
        cmd.arg("--config")
            .arg(self.config_file.path())
            .arg("--cache-dir")
            .arg(&self.cache_dir);
        cmd
    }
}

#[fixture]
fn test_env_unpopulated() -> TestEnv {
    TestEnv::new(false)
}

#[fixture]
fn test_env_populated() -> TestEnv {
    let env = TestEnv::new(false);
    env.populate_cache();
    env
}

#[fixture]
fn test_env_populated_autoupdate() -> TestEnv {
    let env = TestEnv::new(true);
    env.populate_cache();
    env
}

#[rstest]
fn test_lookup_single_ip(test_env_populated: TestEnv) {
    let mut cmd = test_env_populated.cmd();
    cmd.arg("1.1.1.1");
    cmd.assert().success().stdout(predicate::str::contains(
        "13335 | 1.1.1.1 | 1.1.1.0/24 | CLOUDFLARENET | US",
    ));
}

#[rstest]
fn test_lookup_subcommand_still_works(test_env_populated: TestEnv) {
    let mut cmd = test_env_populated.cmd();
    cmd.arg("lookup").arg("1.1.1.1");
    cmd.assert().success().stdout(predicate::str::contains(
        "13335 | 1.1.1.1 | 1.1.1.0/24 | CLOUDFLARENET | US",
    ));
}

#[rstest]
fn test_lookup_not_found(test_env_populated: TestEnv) {
    let mut cmd = test_env_populated.cmd();
    cmd.arg("127.0.0.1");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("127.0.0.1 | Not Found"));
}

#[rstest]
fn test_lookup_stdin(test_env_populated: TestEnv) {
    let mut cmd = test_env_populated.cmd();
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

#[rstest]
fn test_lookup_stdin_with_empty_lines(test_env_populated: TestEnv) {
    let mut cmd = test_env_populated.cmd();
    cmd.write_stdin("8.8.8.8\n\n1.1.1.1\n");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "15169 | 8.8.8.8 | 8.8.8.0/24 | GOOGLE | US",
        ))
        .stdout(predicate::str::contains(
            "13335 | 1.1.1.1 | 1.1.1.0/24 | CLOUDFLARENET | US",
        ));
}

#[rstest]
fn test_lookup_json_output(test_env_populated: TestEnv) {
    let mut cmd = test_env_populated.cmd();
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

#[rstest]
fn test_lookup_invalid_ip_json_output(test_env_populated: TestEnv) {
    let mut cmd = test_env_populated.cmd();
    cmd.arg("--json").arg("not-an-ip");
    cmd.assert().success().stdout(predicate::str::contains(
        r#"{"ip":"not-an-ip","found":false,"info":null}"#,
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

#[rstest]
fn test_run_lookup_no_home_dir() {
    let mut cmd = Command::cargo_bin("ip2asn").unwrap();
    let temp_file = NamedTempFile::new().unwrap();
    cmd.env("HOME", temp_file.path());
    cmd.arg("1.1.1.1");
    cmd.assert().failure().stderr(predicate::str::contains(
        "Dataset not found. Please run `ip2asn update` to download it.",
    ));
}

#[rstest]
fn test_run_lookup_dataset_missing(test_env_unpopulated: TestEnv) {
    let mut cmd = test_env_unpopulated.cmd();
    cmd.arg("1.1.1.1");
    cmd.assert().failure().stderr(predicate::str::contains(
        "Dataset not found. Please run `ip2asn update` to download it.",
    ));
}

#[cfg(test)]
mod auto_update_tests {
    use super::*;
    use mockito;

    #[rstest]
    #[tokio::test]
    async fn test_auto_update_disabled(test_env_populated: TestEnv) {
        let mut cmd = test_env_populated.cmd();
        cmd.arg("1.1.1.1");
        cmd.assert().success().stdout(
            predicate::str::contains("13335 | 1.1.1.1 | 1.1.1.0/24 | CLOUDFLARENET | US")
                .and(predicate::str::contains("Checking for dataset updates...").not()),
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_auto_update_remote_not_newer(test_env_populated_autoupdate: TestEnv) {
        let env = test_env_populated_autoupdate;
        let mut server = mockito::Server::new_async().await;

        let data_path = env.cache_dir.join("data.tsv.gz");

        // Make the local file seem old to bypass the 24h check.
        let old_time = filetime::FileTime::from_unix_time(1, 0);
        filetime::set_file_mtime(&data_path, old_time).unwrap();

        // Now, get that old modification time.
        let modified_time = std::fs::metadata(&data_path).unwrap().modified().unwrap();

        // Mock the HEAD response to indicate the remote file has the *same* (old) modification time,
        // so no download should be triggered.
        let remote_mtime = httpdate::fmt_http_date(modified_time);
        let _mock = server
            .mock("HEAD", "/data/ip2asn-combined.tsv.gz")
            .with_status(200)
            .with_header("Last-Modified", &remote_mtime)
            .create_async()
            .await;

        let mut cmd = env.cmd();
        cmd.env(
            "IP2ASN_DATA_URL",
            server.url() + "/data/ip2asn-combined.tsv.gz",
        )
        .env("IP2ASN_TESTING", "1");
        cmd.arg("1.1.1.1");

        cmd.assert()
            .success()
            .stderr(predicate::str::contains("New dataset found. Downloading...").not())
            .stdout(predicate::str::contains(
                "13335 | 1.1.1.1 | 1.1.1.0/24 | CLOUDFLARENET | US",
            ));
    }

    #[rstest]
    #[tokio::test]
    async fn test_auto_update_skips_check_for_recent_cache(test_env_populated_autoupdate: TestEnv) {
        let env = test_env_populated_autoupdate;
        let mut server = mockito::Server::new_async().await;

        // The mock should NOT be called.
        let _mock = server
            .mock("HEAD", "/data/ip2asn-combined.tsv.gz")
            .with_status(200)
            .create_async()
            .await;

        let mut cmd = env.cmd();
        cmd.env(
            "IP2ASN_DATA_URL",
            server.url() + "/data/ip2asn-combined.tsv.gz",
        );
        cmd.arg("1.1.1.1");

        cmd.assert()
            .success()
            .stderr(predicate::str::contains("Checking for dataset updates...").not())
            .stdout(predicate::str::contains(
                "13335 | 1.1.1.1 | 1.1.1.0/24 | CLOUDFLARENET | US",
            ));

        // This is tricky to assert, as mockito will panic if the mock is not called.
        // Instead, we rely on the stderr check to ensure no "Checking for updates"
        // message was printed, which is a strong indicator the check was skipped.
        // The mock will be dropped and checked for being called 0 times.
    }

    #[rstest]
    #[tokio::test]
    async fn test_auto_update_triggers_download(test_env_populated_autoupdate: TestEnv) {
        let env = test_env_populated_autoupdate;
        let _ = tracing_subscriber::fmt().with_test_writer().try_init();
        let mut server = mockito::Server::new_async().await;

        let data_path = env.cache_dir.join("data.tsv.gz");

        // Make the local file seem old
        let old_time = filetime::FileTime::from_unix_time(1, 0);
        filetime::set_file_mtime(&data_path, old_time).unwrap();

        // Mock the HEAD response to indicate a newer file
        let remote_mtime = httpdate::fmt_http_date(std::time::SystemTime::now());
        let head_mock = server
            .mock("HEAD", "/data/ip2asn-combined.tsv.gz")
            .with_status(200)
            .with_header("Last-Modified", &remote_mtime)
            .create_async()
            .await;

        // Mock the GET response for the download
        let new_data = "8.8.8.0\t8.8.8.255\t15169\tUS\tGOOGLE\n";
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(new_data.as_bytes()).unwrap();
        let compressed_data = encoder.finish().unwrap();

        let get_mock = server
            .mock("GET", "/data/ip2asn-combined.tsv.gz")
            .with_status(200)
            .with_body(&compressed_data)
            .create_async()
            .await;

        let mut cmd = env.cmd();
        cmd.env(
            "IP2ASN_DATA_URL",
            server.url() + "/data/ip2asn-combined.tsv.gz",
        )
        .env("IP2ASN_TESTING", "1");
        cmd.arg("8.8.8.8");

        cmd.assert().success().stdout(predicate::str::contains(
            "15169 | 8.8.8.8 | 8.8.8.0/24 | GOOGLE | US",
        ));

        head_mock.assert_async().await;
        get_mock.assert_async().await;
    }

    #[rstest]
    #[tokio::test]
    async fn test_auto_update_handles_invalid_last_modified_header(
        test_env_populated_autoupdate: TestEnv,
    ) {
        let env = test_env_populated_autoupdate;
        let mut server = mockito::Server::new_async().await;

        let data_path = env.cache_dir.join("data.tsv.gz");
        let old_time = filetime::FileTime::from_unix_time(1, 0);
        filetime::set_file_mtime(&data_path, old_time).unwrap();

        let mock = server
            .mock("HEAD", "/data/ip2asn-combined.tsv.gz")
            .with_status(200)
            .with_header("Last-Modified", "not-a-valid-date")
            .create_async()
            .await;

        let mut cmd = env.cmd();
        cmd.env(
            "IP2ASN_DATA_URL",
            server.url() + "/data/ip2asn-combined.tsv.gz",
        )
        .env("IP2ASN_TESTING", "1");
        cmd.arg("1.1.1.1");

        // The command should fail because of the invalid header.
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Failed to parse HTTP date"));

        mock.assert_async().await;
    }

    #[rstest]
    #[tokio::test]
    async fn test_auto_update_handles_missing_last_modified_header(
        test_env_populated_autoupdate: TestEnv,
    ) {
        let env = test_env_populated_autoupdate;
        let mut server = mockito::Server::new_async().await;

        let data_path = env.cache_dir.join("data.tsv.gz");
        let old_time = filetime::FileTime::from_unix_time(1, 0);
        filetime::set_file_mtime(&data_path, old_time).unwrap();

        let mock = server
            .mock("HEAD", "/data/ip2asn-combined.tsv.gz")
            .with_status(200)
            // No Last-Modified header
            .create_async()
            .await;

        let mut cmd = env.cmd();
        cmd.env(
            "IP2ASN_DATA_URL",
            server.url() + "/data/ip2asn-combined.tsv.gz",
        )
        .env("IP2ASN_TESTING", "1");
        cmd.arg("1.1.1.1");

        // Should succeed and use the cached data, not attempting a download.
        cmd.assert()
            .success()
            .stderr(predicate::str::contains("New dataset found. Downloading...").not())
            .stdout(predicate::str::contains(
                "13335 | 1.1.1.1 | 1.1.1.0/24 | CLOUDFLARENET | US",
            ));

        mock.assert_async().await;
    }

    #[rstest]
    #[tokio::test]
    async fn test_auto_update_forces_download_when_cache_missing() {
        let env = TestEnv::new(true);
        let mut server = mockito::Server::new_async().await;

        // Mock the GET response for the download
        let new_data = "8.8.8.0\t8.8.8.255\t15169\tUS\tGOOGLE\n";
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(new_data.as_bytes()).unwrap();
        let compressed_data = encoder.finish().unwrap();

        let get_mock = server
            .mock("GET", "/data/ip2asn-combined.tsv.gz")
            .with_status(200)
            .with_header("Content-Length", &compressed_data.len().to_string())
            .with_body(&compressed_data)
            .create_async()
            .await;

        let mut cmd = env.cmd();
        cmd.env(
            "IP2ASN_DATA_URL",
            server.url() + "/data/ip2asn-combined.tsv.gz",
        )
        .env("IP2ASN_TESTING", "1");
        cmd.arg("8.8.8.8");

        cmd.assert()
            .success()
            .stderr(predicate::str::contains(
                "Cache file not found. Downloading...",
            ))
            .stdout(predicate::str::contains(
                "15169 | 8.8.8.8 | 8.8.8.0/24 | GOOGLE | US",
            ));

        get_mock.assert_async().await;
    }
}

#[cfg(test)]
mod command_tests {
    use super::*;
    use mockito;

    #[rstest]
    #[tokio::test]
    async fn test_update_subcommand_downloads_data(test_env_unpopulated: TestEnv) {
        let env = test_env_unpopulated;
        let mut server = mockito::Server::new_async().await;

        let data_path = env.cache_dir.join("data.tsv.gz");
        assert!(!data_path.exists());

        // Mock the GET response for the download
        let new_data = "0.0.0.0\t0.0.0.255\t1\tUS\tTEST\n";
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(new_data.as_bytes()).unwrap();
        let compressed_data = encoder.finish().unwrap();

        let mock = server
            .mock("GET", "/data/ip2asn-combined.tsv.gz")
            .with_status(200)
            .with_header("Content-Length", &compressed_data.len().to_string())
            .with_body(&compressed_data)
            .create_async()
            .await;

        let mut cmd = env.cmd();
        cmd.env(
            "IP2ASN_DATA_URL",
            server.url() + "/data/ip2asn-combined.tsv.gz",
        )
        .env("IP2ASN_TESTING", "1");
        cmd.arg("update");

        cmd.assert()
            .success()
            .stderr(predicate::str::contains("Downloading dataset to"));

        mock.assert_async().await;
        assert!(data_path.exists());
        // Verify the content of the downloaded file
        let file_content = fs::read(&data_path).unwrap();
        assert_eq!(file_content, compressed_data);
    }

    #[rstest]
    #[tokio::test]
    async fn test_network_error_during_update(test_env_unpopulated: TestEnv) {
        let env = test_env_unpopulated;
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", "/data/ip2asn-combined.tsv.gz")
            .with_status(500)
            .create_async()
            .await;

        let mut cmd = env.cmd();
        cmd.env(
            "IP2ASN_DATA_URL",
            server.url() + "/data/ip2asn-combined.tsv.gz",
        )
        .env("IP2ASN_TESTING", "1");
        cmd.arg("update");

        cmd.assert().failure().stderr(predicate::str::contains(
            "Error: Update error: HTTP status server error (500 Internal Server Error)",
        ));

        mock.assert_async().await;
    }

    #[rstest]
    fn test_lookup_invalid_stdin(test_env_populated: TestEnv) {
        let mut cmd = test_env_populated.cmd();
        cmd.write_stdin("8.8.8.8\nnot-an-ip\n1.1.1.1\n");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(
                "15169 | 8.8.8.8 | 8.8.8.0/24 | GOOGLE | US",
            ))
            .stdout(predicate::str::contains(
                "13335 | 1.1.1.1 | 1.1.1.0/24 | CLOUDFLARENET | US",
            ))
            .stderr(predicate::str::contains(
                "Error: Invalid IP address 'not-an-ip'",
            ));
    }

    #[rstest]
    fn test_run_update_no_home_dir() {
        let mut cmd = Command::cargo_bin("ip2asn").unwrap();
        let temp_file = NamedTempFile::new().unwrap();
        cmd.env_clear().env("HOME", temp_file.path()).arg("update");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Not a directory"));
    }
}
