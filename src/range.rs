use ip_network::IpNetwork;
use std::net::IpAddr;

#[allow(dead_code)]
pub fn range_to_cidrs(start: IpAddr, end: IpAddr) -> Vec<IpNetwork> {
    if start > end {
        return vec![];
    }
    if start == end {
        let prefix = if start.is_ipv4() { 32 } else { 128 };
        return vec![IpNetwork::new(start, prefix).unwrap()];
    }

    if let (IpAddr::V4(start_v4), IpAddr::V4(end_v4)) = (start, end) {
        for prefix_len in 0..=32 {
            if let Ok(network) = ip_network::Ipv4Network::new_truncate(start_v4, prefix_len) {
                if network.network_address() == start_v4 && network.broadcast_address() == end_v4 {
                    return vec![IpNetwork::from(network)];
                }
            }
        }
    }

    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_single_ipv4_address() {
        let start = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 5));
        let end = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 5));
        let expected: Vec<IpNetwork> = vec![IpNetwork::new(start, 32).unwrap()];
        assert_eq!(range_to_cidrs(start, end), expected);
    }

    #[test]
    fn test_aligned_ipv4_block() {
        let start = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 0));
        let end = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 255));
        let expected: Vec<IpNetwork> = vec![IpNetwork::new(start, 24).unwrap()];
        assert_eq!(range_to_cidrs(start, end), expected);
    }
}
