use ip_network::IpNetwork;
use std::net::IpAddr;

#[allow(dead_code)]
pub fn range_to_cidrs(start: IpAddr, end: IpAddr) -> Vec<IpNetwork> {
    macro_rules! define_range_to_cidrs_impl {
        ($name:ident, $addr_type:ty, $network_type:ty, $int_type:ty, $max_prefix:expr, $last_addr_method:ident) => {
            fn $name(start: $addr_type, end: $addr_type) -> Vec<IpNetwork> {
                let mut results = vec![];
                let mut current_start = start;

                while current_start <= end {
                    let mut prefix_len = $max_prefix;
                    let mut new_network = <$network_type>::new(current_start, prefix_len).unwrap();

                    while prefix_len > 1 {
                        let next_prefix = prefix_len - 1;
                        let Ok(next_network) =
                            <$network_type>::new_truncate(current_start, next_prefix)
                        else {
                            break;
                        };
                        if next_network.network_address() != current_start {
                            break;
                        }
                        if next_network.$last_addr_method() > end {
                            break;
                        }
                        new_network = next_network;
                        prefix_len = next_prefix;
                    }
                    results.push(IpNetwork::from(new_network));

                    let last_as_int = <$int_type>::from(new_network.$last_addr_method());
                    if let Some(next_ip_as_int) = last_as_int.checked_add(1) {
                        current_start = next_ip_as_int.into();
                    } else {
                        break;
                    }
                }
                results
            }
        };
    }

    use std::net::{Ipv4Addr, Ipv6Addr};

    if start > end || start.is_ipv6() != end.is_ipv6() {
        return vec![];
    }

    match (start, end) {
        (IpAddr::V4(start_v4), IpAddr::V4(end_v4)) => {
            define_range_to_cidrs_impl!(
                calculate_v4,
                Ipv4Addr,
                ip_network::Ipv4Network,
                u32,
                32,
                broadcast_address
            );
            calculate_v4(start_v4, end_v4)
        }
        (IpAddr::V6(start_v6), IpAddr::V6(end_v6)) => {
            define_range_to_cidrs_impl!(
                calculate_v6,
                Ipv6Addr,
                ip_network::Ipv6Network,
                u128,
                128,
                last_address
            );
            calculate_v6(start_v6, end_v6)
        }
        _ => vec![], // Should be unreachable due to the check at the top
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_ipv4_single_address() {
        let start = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 5));
        let end = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 5));
        let expected: Vec<IpNetwork> = vec![IpNetwork::new(start, 32).unwrap()];
        assert_eq!(range_to_cidrs(start, end), expected);
    }

    #[test]
    fn test_ipv4_aligned_range() {
        let start = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 0));
        let end = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 255));
        let expected: Vec<IpNetwork> = vec![IpNetwork::new(start, 24).unwrap()];
        assert_eq!(range_to_cidrs(start, end), expected);
    }

    #[test]
    fn test_ipv4_non_aligned_range() {
        let start = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let end = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 10));
        let expected: Vec<IpNetwork> = vec![
            IpNetwork::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), 32).unwrap(),
            IpNetwork::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)), 31).unwrap(),
            IpNetwork::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 4)), 30).unwrap(),
            IpNetwork::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 8)), 31).unwrap(),
            IpNetwork::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 10)), 32).unwrap(),
        ];
        assert_eq!(range_to_cidrs(start, end), expected);
    }

    #[test]
    fn test_ipv6_single_address() {
        use std::net::Ipv6Addr;
        let start = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1));
        let end = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1));
        let expected: Vec<IpNetwork> = vec![IpNetwork::new(start, 128).unwrap()];
        assert_eq!(range_to_cidrs(start, end), expected);
    }

    #[test]
    fn test_ipv6_aligned_range() {
        use std::net::Ipv6Addr;
        let start = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0));
        let end = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0xffff));
        let expected: Vec<IpNetwork> = vec![IpNetwork::new(start, 112).unwrap()];
        assert_eq!(range_to_cidrs(start, end), expected);
    }

    #[test]
    fn test_ipv6_non_aligned_range() {
        use std::net::Ipv6Addr;
        let start = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1));
        let end = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 10));
        let expected: Vec<IpNetwork> = vec![
            IpNetwork::new(
                IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)),
                128,
            )
            .unwrap(),
            IpNetwork::new(
                IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 2)),
                127,
            )
            .unwrap(),
            IpNetwork::new(
                IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 4)),
                126,
            )
            .unwrap(),
            IpNetwork::new(
                IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 8)),
                127,
            )
            .unwrap(),
            IpNetwork::new(
                IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 10)),
                128,
            )
            .unwrap(),
        ];
        assert_eq!(range_to_cidrs(start, end), expected);
    }

    #[test]
    fn test_ipv4_invalid_range_start_greater_than_end() {
        let start = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 10));
        let end = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let expected: Vec<IpNetwork> = vec![];
        assert_eq!(range_to_cidrs(start, end), expected);
    }

    #[test]
    fn test_ipv6_invalid_range_start_greater_than_end() {
        use std::net::Ipv6Addr;
        let start = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 10));
        let end = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1));
        let expected: Vec<IpNetwork> = vec![];
        assert_eq!(range_to_cidrs(start, end), expected);
    }

    #[test]
    fn test_ipv4_full_range() {
        let start = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
        let end = IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255));
        let expected: Vec<IpNetwork> = vec![
            IpNetwork::new(start, 1).unwrap(),
            IpNetwork::new(IpAddr::V4(Ipv4Addr::new(128, 0, 0, 0)), 1).unwrap(),
        ];
        assert_eq!(range_to_cidrs(start, end), expected);
    }
    #[test]
    fn test_ipv6_full_range() {
        use std::net::Ipv6Addr;
        let start = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0));
        let end = IpAddr::V6(Ipv6Addr::new(
            0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff,
        ));
        let expected: Vec<IpNetwork> = vec![
            IpNetwork::new(start, 1).unwrap(),
            IpNetwork::new(IpAddr::V6(Ipv6Addr::new(0x8000, 0, 0, 0, 0, 0, 0, 0)), 1).unwrap(),
        ];
        assert_eq!(range_to_cidrs(start, end), expected);
    }

    #[test]
    fn test_range_to_cidrs_from_real_data() {
        // From testdata/testdata-small-ip2asn.tsv
        // Sample 1: 38.103.144.0    38.103.149.255
        let start_v4_1 = IpAddr::V4(Ipv4Addr::new(38, 103, 144, 0));
        let end_v4_1 = IpAddr::V4(Ipv4Addr::new(38, 103, 149, 255));
        let expected_v4_1: Vec<IpNetwork> = vec![
            IpNetwork::new(IpAddr::V4(Ipv4Addr::new(38, 103, 144, 0)), 22).unwrap(),
            IpNetwork::new(IpAddr::V4(Ipv4Addr::new(38, 103, 148, 0)), 23).unwrap(),
        ];
        // After running the test, the actual output was:
        // left: [V4(Ipv4Network { network_address: 38.103.144.0, netmask: 22 }), V4(Ipv4Network { network_address: 38.103.148.0, netmask: 23 })]
        // My manual calculation was wrong. The greedy algorithm is better.
        // 38.103.144.0/22 -> 38.103.144.0 - 38.103.147.255
        // 38.103.148.0/23 -> 38.103.148.0 - 38.103.149.255
        // This is correct.
        assert_eq!(range_to_cidrs(start_v4_1, end_v4_1), expected_v4_1);

        // Sample 2: 2804:2f8c::     2804:2f8c:ffff:ffff:ffff:ffff:ffff:ffff
        use std::net::Ipv6Addr;
        let start_v6_1 = IpAddr::V6(Ipv6Addr::new(0x2804, 0x2f8c, 0, 0, 0, 0, 0, 0));
        let end_v6_1 = IpAddr::V6(Ipv6Addr::new(
            0x2804, 0x2f8c, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff,
        ));
        let expected_v6_1: Vec<IpNetwork> = vec![IpNetwork::new(start_v6_1, 32).unwrap()];
        assert_eq!(range_to_cidrs(start_v6_1, end_v6_1), expected_v6_1);
    }
}
