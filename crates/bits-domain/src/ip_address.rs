//! IP address type with PII-safe Debug output.
//!
//! IP addresses are PII under GDPR and should be treated carefully.
//! While they're needed for security logging (rate limiting, failed logins),
//! they should not appear in general debug output.
//!
//! # Construction
//!
//! - `IpAddress::new(addr)` - Trusted constructor (from request headers, already parsed)
//! - `IpAddress::parse(s)` - Validating constructor (from strings)
//! - `s.parse::<IpAddress>()` - Via FromStr trait

use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IpAddress(IpAddr);

#[derive(Debug, thiserror::Error)]
pub enum IpAddressError {
    #[error("Invalid IP address format: {0}")]
    Invalid(#[from] std::net::AddrParseError),
}

impl IpAddress {
    /// Trusted constructor - wraps without validation.
    ///
    /// Use when you already have a parsed IpAddr (from request headers, etc.).
    #[must_use]
    pub fn new(addr: IpAddr) -> Self {
        IpAddress(addr)
    }

    /// Validating constructor - parses IP address from string.
    ///
    /// Accepts both IPv4 and IPv6 addresses.
    pub fn parse(s: &str) -> Result<Self, IpAddressError> {
        let addr = s.parse::<IpAddr>()?;
        Ok(IpAddress(addr))
    }

    /// Get the IP address.
    ///
    /// This is intentionally explicit to make PII access auditable.
    #[must_use]
    pub fn as_ip_addr(&self) -> IpAddr {
        self.0
    }

    /// Check if this is an IPv4 address.
    #[must_use]
    pub fn is_ipv4(&self) -> bool {
        self.0.is_ipv4()
    }

    /// Check if this is an IPv6 address.
    #[must_use]
    pub fn is_ipv6(&self) -> bool {
        self.0.is_ipv6()
    }
}

impl FromStr for IpAddress {
    type Err = IpAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        IpAddress::parse(s)
    }
}

// PII-safe Debug: redact the IP address
impl fmt::Debug for IpAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IpAddress").field(&"<redacted>").finish()
    }
}

impl fmt::Display for IpAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<IpAddr> for IpAddress {
    fn from(addr: IpAddr) -> Self {
        IpAddress::new(addr)
    }
}

impl From<IpAddress> for IpAddr {
    fn from(addr: IpAddress) -> Self {
        addr.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn parse_valid_ipv4() {
        let ip = IpAddress::parse("192.168.1.1").unwrap();
        assert_eq!(ip.as_ip_addr(), IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
    }

    #[test]
    fn parse_valid_ipv6() {
        let ip = IpAddress::parse("2001:db8::1").unwrap();
        assert_eq!(
            ip.as_ip_addr(),
            IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1))
        );
    }

    #[test]
    fn parse_rejects_invalid() {
        assert!(IpAddress::parse("not an ip").is_err());
        assert!(IpAddress::parse("999.999.999.999").is_err());
    }

    #[test]
    fn from_str_trait() {
        let ip: IpAddress = "127.0.0.1".parse().unwrap();
        assert_eq!(ip.as_ip_addr(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    }

    #[test]
    fn new_trusted_constructor() {
        let addr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let ip = IpAddress::new(addr);
        assert_eq!(ip.as_ip_addr(), addr);
    }

    #[test]
    fn debug_redacted() {
        let ip = IpAddress::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
        let debug = format!("{:?}", ip);
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("192.168"));
    }

    #[test]
    fn display_shows_ip() {
        let ip = IpAddress::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
        assert_eq!(ip.to_string(), "192.168.1.1");
    }

    #[test]
    fn is_ipv4() {
        let ip = IpAddress::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert!(ip.is_ipv4());
        assert!(!ip.is_ipv6());
    }

    #[test]
    fn is_ipv6() {
        let ip = IpAddress::new(IpAddr::V6(Ipv6Addr::LOCALHOST));
        assert!(ip.is_ipv6());
        assert!(!ip.is_ipv4());
    }

    #[test]
    fn from_ip_addr() {
        let addr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let ip: IpAddress = addr.into();
        assert_eq!(ip.as_ip_addr(), addr);
    }

    #[test]
    fn into_ip_addr() {
        let ip = IpAddress::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
        let addr: IpAddr = ip.into();
        assert_eq!(addr, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
    }
}
