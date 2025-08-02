//! Integration tests for the line parser.

use ip2asn::parser::{parse_line, ParsedLine};
use std::net::{Ipv4Addr, Ipv6Addr};

#[test]
fn test_parse_line_happy_path() {
    let line = "1.0.0.0\t1.0.0.255\t13335\tUS\tCLOUDFLARENET";
    let result = parse_line(line).unwrap();

    assert_eq!(
        result,
        ParsedLine {
            start_ip: Ipv4Addr::new(1, 0, 0, 0).into(),
            end_ip: Ipv4Addr::new(1, 0, 0, 255).into(),
            asn: 13335,
            country_code: [b'U', b'S'],
            organization: "CLOUDFLARENET",
        }
    );
}

#[test]
fn test_parse_line_incorrect_column_count() {
    let line = "1.0.0.0\t1.0.0.255\t13335"; // Missing columns
    let result = parse_line(line);
    assert!(result.is_err());
    // We'll assert the specific error kind later
}

#[test]
fn test_parse_line_invalid_start_ip() {
    let line = "not-an-ip\t1.0.0.255\t13335\tUS\tCLOUDFLARENET";
    let result = parse_line(line);
    assert!(matches!(
        result,
        Err(ip2asn::ParseErrorKind::InvalidIpAddress { .. })
    ));
}

#[test]
fn test_parse_line_invalid_end_ip() {
    let line = "1.0.0.0\tnot-an-ip\t13335\tUS\tCLOUDFLARENET";
    let result = parse_line(line);
    assert!(matches!(
        result,
        Err(ip2asn::ParseErrorKind::InvalidIpAddress { .. })
    ));
}

#[test]
fn test_parse_line_invalid_asn() {
    let line = "1.0.0.0\t1.0.0.255\tnot-a-number\tUS\tCLOUDFLARENET";
    let result = parse_line(line);
    assert!(matches!(
        result,
        Err(ip2asn::ParseErrorKind::InvalidAsnNumber { .. })
    ));
}

#[test]
fn test_parse_line_ip_family_mismatch() {
    let line = "1.0.0.0\t::1\t13335\tUS\tCLOUDFLARENET";
    let result = parse_line(line);
    assert!(matches!(
        result,
        Err(ip2asn::ParseErrorKind::IpFamilyMismatch)
    ));
}

#[test]
fn test_parse_line_invalid_range() {
    let line = "1.0.0.255\t1.0.0.0\t13335\tUS\tCLOUDFLARENET";
    let result = parse_line(line);
    assert!(matches!(
        result,
        Err(ip2asn::ParseErrorKind::InvalidRange { .. })
    ));
}

#[test]
fn test_country_code_normalization() {
    let line = "1.0.0.0\t1.0.0.255\t13335\tNone\tCLOUDFLARENET";
    let result = parse_line(line).unwrap();
    assert_eq!(result.country_code, [b'Z', b'Z']);
}

#[test]
fn test_parse_line_from_real_data() {
    // Case 1: Standard IPv4
    let line1 = "185.237.4.0\t185.237.4.255\t14618\tUS\tAMAZON-AES";
    let result1 = parse_line(line1).unwrap();
    assert_eq!(result1.start_ip, Ipv4Addr::new(185, 237, 4, 0));
    assert_eq!(result1.end_ip, Ipv4Addr::new(185, 237, 4, 255));
    assert_eq!(result1.asn, 14618);
    assert_eq!(result1.country_code, [b'U', b'S']);
    assert_eq!(result1.organization, "AMAZON-AES");

    // Case 2: Standard IPv6
    let line2 = "2803:c280::\t2803:c280:2:ffff:ffff:ffff:ffff:ffff\t265775\tEC\tAUSTRONET";
    let result2 = parse_line(line2).unwrap();
    assert_eq!(
        result2.start_ip,
        Ipv6Addr::new(0x2803, 0xc280, 0, 0, 0, 0, 0, 0)
    );
    assert_eq!(
        result2.end_ip,
        Ipv6Addr::new(0x2803, 0xc280, 0x0002, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff)
    );
    assert_eq!(result2.asn, 265775);
    assert_eq!(result2.country_code, [b'E', b'C']);
    assert_eq!(result2.organization, "AUSTRONET");

    // Case 3: IPv6 with "None" country code
    let line3 = "2a02:fe80:22::\t2a02:fe80:100f:ffff:ffff:ffff:ffff:ffff\t0\tNone\tNot routed";
    let result3 = parse_line(line3).unwrap();
    assert_eq!(result3.country_code, [b'Z', b'Z']);
    assert_eq!(result3.asn, 0);
    assert_eq!(result3.organization, "Not routed");

    // Case 4: Multi-word organization
    let line4 = "213.230.0.0\t213.230.0.255\t28938\tSA\tMEDUNET-AS Program for Medical and Educational Telecommunications Riyadh, Saudi Arabia";
    let result4 = parse_line(line4).unwrap();
    assert_eq!(result4.asn, 28938);
    assert_eq!(result4.country_code, [b'S', b'A']);
    assert_eq!(
        result4.organization,
        "MEDUNET-AS Program for Medical and Educational Telecommunications Riyadh, Saudi Arabia"
    );
}

#[test]
fn test_parse_line_empty() {
    let line = "";
    let result = parse_line(line);
    assert!(matches!(
        result,
        Err(ip2asn::ParseErrorKind::IncorrectColumnCount { .. })
    ));
}

#[test]
fn test_parse_line_malformed_country_code() {
    let line = "1.0.0.0\t1.0.0.255\t13335\tUSA\tCLOUDFLARENET";
    let result = parse_line(line);
    assert!(matches!(
        result,
        Err(ip2asn::ParseErrorKind::InvalidCountryCode { .. })
    ));
}
