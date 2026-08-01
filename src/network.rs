use crate::error::SlmpError;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs};

pub(crate) fn normalize_ipv4_host(host: &str) -> Result<String, SlmpError> {
    let normalized = host.trim();
    if normalized.is_empty() {
        return Err(SlmpError::new("host must not be empty"));
    }
    let literal_text = normalized
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(normalized);
    match literal_text.parse::<IpAddr>() {
        Ok(IpAddr::V4(_)) => Ok(literal_text.to_string()),
        Ok(IpAddr::V6(_)) => Err(SlmpError::new(
            "host must be an IPv4 address or a hostname that resolves to IPv4; IPv6 is unsupported",
        )),
        Err(_) => Ok(normalized.to_string()),
    }
}

pub(crate) fn resolve_ipv4_addresses(host: &str, port: u16) -> Result<Vec<SocketAddr>, SlmpError> {
    let normalized = normalize_ipv4_host(host)?;
    if let Ok(address) = normalized.parse::<Ipv4Addr>() {
        return Ok(vec![SocketAddr::from((address, port))]);
    }
    let resolved = (normalized.as_str(), port)
        .to_socket_addrs()
        .map_err(|error| {
            SlmpError::transport(format!(
                "host resolution failed for {normalized}:{port}: {error}"
            ))
        })?;
    let addresses: Vec<_> = resolved.filter(SocketAddr::is_ipv4).collect();
    if addresses.is_empty() {
        return Err(SlmpError::transport(format!(
            "host did not resolve to an IPv4 address: {normalized}"
        )));
    }
    Ok(addresses)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_ipv6_literals_and_normalizes_ipv4() {
        for host in ["::1", "[::1]", "::ffff:127.0.0.1"] {
            assert!(
                normalize_ipv4_host(host)
                    .unwrap_err()
                    .to_string()
                    .contains("IPv6")
            );
        }
        assert_eq!(normalize_ipv4_host(" 127.0.0.1 ").unwrap(), "127.0.0.1");
        assert_eq!(normalize_ipv4_host(" plc.local ").unwrap(), "plc.local");
    }

    #[test]
    fn localhost_resolution_returns_only_ipv4_addresses() {
        let addresses = resolve_ipv4_addresses("localhost", 1025).unwrap();
        assert!(!addresses.is_empty());
        assert!(addresses.iter().all(SocketAddr::is_ipv4));
    }

    #[test]
    fn ipv4_literal_resolution_does_not_need_a_system_resolver() {
        assert_eq!(
            resolve_ipv4_addresses("127.0.0.1", 1025).unwrap(),
            vec!["127.0.0.1:1025".parse().unwrap()]
        );
    }
}
