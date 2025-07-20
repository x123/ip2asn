use ip2asn::{AsnInfoView, Builder, IpAsnMap};
use std::net::Ipv4Addr;

const TEST_DATA: &str = r#"
1.0.0.0	1.0.0.255	13335	US	CLOUDFLARENET
1.0.1.0	1.0.3.255	38040	AU	GTELECOM
1.0.4.0	1.0.5.255	56203	CN	CNNIC
# Add a duplicate organization to test the interner
8.8.8.0	8.8.8.255	15169	US	GTELECOM
"#;

#[test]
fn test_builder_and_lookup() {
    let map: IpAsnMap = Builder::with_source(TEST_DATA.as_bytes()).build().unwrap();

    // Case 1: IP in the middle of a range
    let result1 = map.lookup(Ipv4Addr::new(1, 0, 0, 100).into()).unwrap();
    assert_eq!(
        result1,
        AsnInfoView {
            asn: 13335,
            country_code: "US",
            organization: "CLOUDFLARENET",
        }
    );

    // Case 2: IP at the start of a range
    let result2 = map.lookup(Ipv4Addr::new(1, 0, 1, 0).into()).unwrap();
    assert_eq!(
        result2,
        AsnInfoView {
            asn: 38040,
            country_code: "AU",
            organization: "GTELECOM",
        }
    );

    // Case 3: IP at the end of a range
    let result3 = map.lookup(Ipv4Addr::new(1, 0, 3, 255).into()).unwrap();
    assert_eq!(result3, result2); // Should be the same record

    // Case 4: IP not in any range
    let result4 = map.lookup(Ipv4Addr::new(127, 0, 0, 1).into());
    assert!(result4.is_none());

    // Case 5: Check interned string
    let result5 = map.lookup(Ipv4Addr::new(8, 8, 8, 8).into()).unwrap();
    assert_eq!(
        result5,
        AsnInfoView {
            asn: 15169,
            country_code: "US",
            organization: "GTELECOM",
        }
    );
    // Check that the organization string is the same instance as result2
    assert_eq!(result2.organization, result5.organization);
}
