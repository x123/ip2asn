use ip_network::IpNetwork;
use ip2asn::{Builder, Error, IpAsnMap, ParseErrorKind, Warning};
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const TEST_DATA: &str = r#"
1.0.0.0	1.0.0.255	13335	US	CLOUDFLARENET
1.0.1.0	1.0.3.255	38040	AU	GTELECOM
1.0.4.0	1.0.5.255	56203	CN	CNNIC
# Add a duplicate organization to test the interner
8.8.8.0	8.8.8.255	15169	US	GTELECOM
"#;

#[test]
fn test_builder_and_lookup() {
    let map: IpAsnMap = Builder::new()
        .with_source(TEST_DATA.as_bytes())
        .unwrap()
        .build()
        .unwrap();

    // Case 1: IP in the middle of a range
    let result1 = map.lookup(Ipv4Addr::new(1, 0, 0, 100).into()).unwrap();
    assert_eq!(result1.network, "1.0.0.0/24".parse::<IpNetwork>().unwrap());
    assert_eq!(result1.asn, 13335);
    assert_eq!(result1.country_code, "US");
    assert_eq!(result1.organization, "CLOUDFLARENET");

    // Case 2: IP at the start of a range
    let result2 = map.lookup(Ipv4Addr::new(1, 0, 1, 0).into()).unwrap();
    assert_eq!(result2.network, "1.0.1.0/24".parse::<IpNetwork>().unwrap());
    assert_eq!(result2.asn, 38040);
    assert_eq!(result2.country_code, "AU");
    assert_eq!(result2.organization, "GTELECOM");

    // Case 3: IP at the end of a range
    let result3 = map.lookup(Ipv4Addr::new(1, 0, 3, 255).into()).unwrap();
    assert_eq!(result3.network, "1.0.2.0/23".parse::<IpNetwork>().unwrap());
    assert_eq!(result3.asn, 38040);
    assert_eq!(result3.country_code, "AU");
    assert_eq!(result3.organization, "GTELECOM");

    // Case 4: IP not in any range
    let result4 = map.lookup(Ipv4Addr::new(127, 0, 0, 1).into());
    assert!(result4.is_none());

    // Case 5: Check interned string
    let result5 = map.lookup(Ipv4Addr::new(8, 8, 8, 8).into()).unwrap();
    assert_eq!(result5.network, "8.8.8.0/24".parse::<IpNetwork>().unwrap());
    assert_eq!(result5.asn, 15169);
    assert_eq!(result5.country_code, "US");
    assert_eq!(result5.organization, "GTELECOM");
    // Check that the organization string is the same instance as result2
    assert_eq!(result2.organization, result5.organization);
}

#[test]
fn test_builder_from_path() {
    // Test with plain text file
    let map_plain = Builder::new()
        .from_path("testdata/testdata-small-ip2asn.tsv")
        .unwrap()
        .build()
        .unwrap();
    let result_plain = map_plain.lookup("154.16.226.100".parse().unwrap()).unwrap();
    assert_eq!(
        result_plain.network,
        "154.16.226.0/24".parse::<IpNetwork>().unwrap()
    );
    assert_eq!(result_plain.asn, 61317);
    assert_eq!(result_plain.country_code, "US");
    assert_eq!(result_plain.organization, "ASDETUK www.heficed.com");

    // Test with gzipped file
    let map_gz = Builder::new()
        .from_path("testdata/testdata-small-ip2asn.tsv.gz")
        .unwrap()
        .build()
        .unwrap();
    let result_gz = map_gz.lookup("154.16.226.100".parse().unwrap()).unwrap();
    assert_eq!(result_plain.network, result_gz.network);
    assert_eq!(result_plain.asn, result_gz.asn);
    assert_eq!(result_plain.country_code, result_gz.country_code);
    assert_eq!(result_plain.organization, result_gz.organization);

    // Check an IPv6 address from the file
    let result_ipv6 = map_gz.lookup("2001:67c:2309::1".parse().unwrap()).unwrap();
    assert_eq!(
        result_ipv6.network,
        "2001:67c:2309::/48".parse::<IpNetwork>().unwrap()
    );
    assert_eq!(result_ipv6.asn, 0);
    assert_eq!(result_ipv6.country_code, "ZZ");
    assert_eq!(result_ipv6.organization, "Not routed");

    // Check a multi-word organization name
    let result_multi_word = map_plain.lookup("45.234.212.10".parse().unwrap()).unwrap();
    assert_eq!(
        result_multi_word.network,
        "45.234.212.0/22".parse::<IpNetwork>().unwrap()
    );
    assert_eq!(result_multi_word.asn, 267373);
    assert_eq!(result_multi_word.country_code, "BR");
    assert_eq!(result_multi_word.organization, "AGIL TECOMUNICACOES LTDA");
}

#[test]
fn test_builder_from_path_not_found() {
    let result = Builder::new().from_path("testdata/file-does-not-exist.tsv");
    assert!(result.is_err());
    match result {
        Err(Error::Io(e)) => {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        }
        _ => panic!("Expected an I/O error"),
    }
}

const MALFORMED_DATA: &str = r#"
# This line is a comment and should be skipped
1.0.0.0	1.0.0.255	13335	US	CLOUDFLARENET
1.0.1.0	1.0.3.255	38040	AU	GTELECOM
invalid line format
1.0.4.0	1.0.5.255	not-a-number	CN	CNNIC
# Another comment
8.8.8.0	8.8.8.255	15169	US	GTELECOM
"#;

#[test]
fn test_builder_strict_mode() {
    let result = Builder::new()
        .with_source(MALFORMED_DATA.as_bytes())
        .unwrap()
        .strict()
        .build();

    assert!(result.is_err());
    match result {
        Err(Error::Parse {
            line_number,
            line_content,
            kind,
        }) => {
            assert_eq!(line_number, 5);
            assert_eq!(line_content, "invalid line format");
            assert_eq!(
                kind,
                ParseErrorKind::IncorrectColumnCount {
                    expected: 5,
                    found: 1
                }
            );
        }
        _ => panic!("Expected a Parse error"),
    }
}

#[test]
fn test_builder_warning_callback() {
    let warning_count = Arc::new(AtomicUsize::new(0));
    let warning_count_clone = warning_count.clone();

    let callback = move |warning: Warning| {
        println!("Warning received: {:?}", warning);
        warning_count_clone.fetch_add(1, Ordering::SeqCst);
    };

    let map = Builder::new()
        .with_source(MALFORMED_DATA.as_bytes())
        .unwrap()
        .on_warning(callback)
        .build()
        .unwrap();

    // Check that the valid data was still loaded
    let result = map.lookup(Ipv4Addr::new(1, 0, 0, 100).into()).unwrap();
    assert_eq!(result.asn, 13335);

    // Check that the correct number of warnings were issued
    assert_eq!(warning_count.load(Ordering::SeqCst), 2);
}

#[cfg(feature = "fetch")]
mod fetch_tests {
    use super::*;
    use std::io::Read;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_builder_from_url() {
        // Start a mock server.
        let server = MockServer::start().await;

        // Read the gzipped test data.
        let mut file = std::fs::File::open("testdata/testdata-small-ip2asn.tsv.gz").unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        // Set up the mock response.
        Mock::given(method("GET"))
            .and(path("/ip2asn.tsv.gz"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(buffer))
            .mount(&server)
            .await;

        // Construct the URL.
        let url = format!("{}/ip2asn.tsv.gz", server.uri());

        // Build the map from the URL.
        let url_clone = url.clone();
        let map = tokio::task::spawn_blocking(move || {
            Builder::new()
                .from_url(&url_clone)
                .unwrap()
                .build()
                .unwrap()
        })
        .await
        .unwrap();

        // Perform lookups to verify the data was loaded correctly.
        let view = map.lookup("154.16.226.100".parse().unwrap()).unwrap();
        assert_eq!(view.asn, 61317);
        assert_eq!(view.organization, "ASDETUK www.heficed.com");

        let view_ipv6 = map.lookup("2001:67c:2309::1".parse().unwrap()).unwrap();
        assert_eq!(view_ipv6.asn, 0);
        assert_eq!(view_ipv6.organization, "Not routed");
    }

    #[tokio::test]
    async fn test_builder_from_url_not_gzipped() {
        let server = MockServer::start().await;
        let test_data = "1.0.0.0\t1.0.0.255\t13335\tUS\tCLOUDFLARENET";
        Mock::given(method("GET"))
            .and(path("/ip2asn.tsv"))
            .respond_with(ResponseTemplate::new(200).set_body_string(test_data))
            .mount(&server)
            .await;

        let url = format!("{}/ip2asn.tsv", server.uri());
        let url_clone = url.clone();
        let map = tokio::task::spawn_blocking(move || {
            Builder::new()
                .from_url(&url_clone)
                .unwrap()
                .build()
                .unwrap()
        })
        .await
        .unwrap();

        let view = map.lookup("1.0.0.1".parse().unwrap()).unwrap();
        assert_eq!(view.asn, 13335);
        assert_eq!(view.organization, "CLOUDFLARENET");
    }

    #[tokio::test]
    async fn test_builder_from_url_http_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/not-found"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let url = format!("{}/not-found", server.uri());
        let url_clone = url.clone();
        let result = tokio::task::spawn_blocking(move || Builder::new().from_url(&url_clone)).await;

        assert!(result.is_ok());
        let inner_result = result.unwrap();
        assert!(inner_result.is_err());
        match inner_result {
            Err(Error::Http(_)) => {
                // Expected error
            }
            _ => panic!("Expected an HTTP error"),
        }
    }
}

#[test]
fn test_new_empty_map() {
    let map = IpAsnMap::new();

    // Assert that a lookup for any IP address returns None.
    let ip = "192.0.2.1".parse().unwrap();
    assert!(map.lookup(ip).is_none());

    // Verify the Debug representation is reasonable.
    let debug_repr = format!("{:?}", map);
    assert_eq!(debug_repr, "IpAsnMap { organizations: 0, .. }");
}

#[test]
fn test_lookup_owned() {
    let map: IpAsnMap = Builder::new()
        .with_source(TEST_DATA.as_bytes())
        .unwrap()
        .build()
        .unwrap();

    let result = map.lookup_owned("1.0.2.10".parse().unwrap()).unwrap();
    assert_eq!(result.network, "1.0.2.0/23".parse::<IpNetwork>().unwrap());
    assert_eq!(result.asn, 38040);
    assert_eq!(result.country_code, "AU");
    assert_eq!(result.organization, "GTELECOM");

    // Test a miss
    assert!(map.lookup_owned("127.0.0.1".parse().unwrap()).is_none());
}

#[cfg(feature = "serde")]
#[test]
fn test_asn_info_serde() {
    let map: IpAsnMap = Builder::new()
        .with_source(TEST_DATA.as_bytes())
        .unwrap()
        .build()
        .unwrap();

    let info = map.lookup_owned("1.0.0.1".parse().unwrap()).unwrap();

    let serialized = serde_json::to_string(&info).unwrap();
    let deserialized = serde_json::from_str(&serialized).unwrap();

    assert_eq!(info, deserialized);
}
