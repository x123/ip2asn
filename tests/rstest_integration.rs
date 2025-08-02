use ip2asn::Builder;
use ip2asn::IpAsnMap;
use rstest::fixture;
use rstest::rstest;
use std::net::IpAddr;

#[fixture]
fn small_map() -> IpAsnMap {
    let data = include_bytes!("../testdata/testdata-small-ip2asn.tsv.gz");
    Builder::new()
        .with_source(data.as_ref())
        .unwrap()
        .on_warning(|w| println!("Warning: {}", w))
        .build()
        .unwrap()
}

#[rstest]
fn test_lookup_with_rstest_fixture(small_map: IpAsnMap) {
    let ip: IpAddr = "1.1.1.1".parse().unwrap();
    let info = small_map.lookup(ip).unwrap();
    assert_eq!(info.asn, 13335);
    assert_eq!(info.organization, "CLOUDFLARENET");
}
