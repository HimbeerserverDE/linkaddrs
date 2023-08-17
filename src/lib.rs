use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ops::AddAssign;

use futures::future;
use futures::stream::{StreamExt, TryStreamExt};
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use netlink_packet_route::address::Nla::Address;
use netlink_packet_route::rtnl::constants::{AF_INET, AF_INET6};
use rtnetlink::new_connection;
use tokio::runtime::Runtime;

/// The errors that can occur when interacting with rtnetlink.
#[derive(Debug)]
pub enum Error {
    RtNetlink(rtnetlink::Error),
    IoError(std::io::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RtNetlink(e) => write!(fmt, "rtnetlink error: {}", e),
            Self::IoError(e) => write!(fmt, "rtnetlink connection failed: {}", e),
        }
    }
}

impl From<rtnetlink::Error> for Error {
    fn from(e: rtnetlink::Error) -> Self {
        Self::RtNetlink(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

/// An alias for `std::result::Result` that uses `Error`
/// as its error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Get all IP addresses of an interface.
pub fn addresses(link: String) -> Result<Vec<IpNet>> {
    let rt = Runtime::new()?;

    rt.block_on(internal_addresses(Some(link)))
}

/// Get the IPv4 addresses of an interface.
pub fn ipv4_addresses(link: String) -> Result<Vec<Ipv4Net>> {
    let addrs = addresses(link)?
        .iter()
        .filter_map(|addr| match addr {
            IpNet::V4(addr) => Some(*addr),
            IpNet::V6(_) => None,
        })
        .collect();

    Ok(addrs)
}

/// Get the IPv6 addresses of an interface.
pub fn ipv6_addresses(link: String) -> Result<Vec<Ipv6Net>> {
    let addrs = addresses(link)?
        .iter()
        .filter_map(|addr| match addr {
            IpNet::V4(_) => None,
            IpNet::V6(addr) => Some(*addr),
        })
        .collect();

    Ok(addrs)
}

/// Get all IP addresses of this host.
pub fn all_addresses() -> Result<Vec<IpNet>> {
    let rt = Runtime::new()?;

    rt.block_on(internal_addresses(None))
}

/// Get the IPv4 addresses of this host.
pub fn all_ipv4_addresses() -> Result<Vec<Ipv4Net>> {
    let addrs = all_addresses()?
        .iter()
        .filter_map(|addr| match addr {
            IpNet::V4(addr) => Some(*addr),
            IpNet::V6(_) => None,
        })
        .collect();

    Ok(addrs)
}

/// Get the IPv6 addresses of this host.
pub fn all_ipv6_addresses() -> Result<Vec<Ipv6Net>> {
    let addrs = all_addresses()?
        .iter()
        .filter_map(|addr| match addr {
            IpNet::V4(_) => None,
            IpNet::V6(addr) => Some(*addr),
        })
        .collect();

    Ok(addrs)
}

/// Get the IP addresses. If filter is Some, limit the search
/// to that interface.
async fn internal_addresses(filter: Option<String>) -> Result<Vec<IpNet>> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut links = handle.link().get();

    if let Some(link) = filter.clone() {
        links = links.match_name(link);
    }

    let mut links = links.execute();

    let mut num_links = 0_i32;
    let mut link_addrs = Vec::new();

    while let Some(link) = links.try_next().await? {
        let addrs = handle
            .address()
            .get()
            .set_link_index_filter(link.header.index)
            .execute();

        let addrs = addrs
            .map_ok(|v| {
                if let Some(Address(bytes)) = v.nlas.first() {
                    match v.header.family as u16 {
                        AF_INET => {
                            let octets: [u8; 4] = (*bytes).clone().try_into().unwrap();
                            let ip = IpAddr::from(Ipv4Addr::from(octets));
                            let net = IpNet::new(ip, v.header.prefix_len).unwrap();

                            Some(net)
                        }
                        AF_INET6 => {
                            let octets: [u8; 16] = (*bytes).clone().try_into().unwrap();
                            let ip = IpAddr::from(Ipv6Addr::from(octets));
                            let net = IpNet::new(ip, v.header.prefix_len).unwrap();

                            Some(net)
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .try_filter(|v| future::ready(v.is_some()))
            .filter_map(|v| future::ready(v.unwrap()));

        link_addrs.append(&mut addrs.collect::<Vec<IpNet>>().await);

        num_links.add_assign(1);
    }

    Ok(link_addrs)
}
