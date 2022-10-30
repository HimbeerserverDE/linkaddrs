use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use futures::future;
use futures::stream::{StreamExt, TryStreamExt};
use netlink_packet_route::address::Nla::Address;
use netlink_packet_route::rtnl::constants::{AF_INET, AF_INET6};
use rtnetlink::new_connection;
use tokio::runtime::Runtime;

#[derive(Debug)]
pub enum Error {
    RtNetlink(rtnetlink::Error),
    IoError(std::io::Error),
    LinkNotFound(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RtNetlink(e) => write!(fmt, "rtnetlink error: {}", e),
            Self::IoError(e) => write!(fmt, "rtnetlink connection failed: {}", e),
            Self::LinkNotFound(link) => write!(fmt, "link not found: {}", link),
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

type Result<T> = std::result::Result<T, Error>;

pub fn addresses(link: String) -> Result<Vec<IpAddr>> {
    let rt = Runtime::new()?;

    rt.block_on(internal_addresses(link))
}

async fn internal_addresses(link: String) -> Result<Vec<IpAddr>> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut links = handle
        .link()
        .get()
        .match_name(link.clone())
        .execute();

    if let Some(link) = links.try_next().await? {
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
                            let octets: [u8; 4] = (*bytes)
                                .clone()
                                .try_into()
                                .unwrap();

                            let ip = IpAddr::from(
                                Ipv4Addr::from(octets)
                            );

                            Some(ip)
                        }
                        AF_INET6 => {
                            let octets: [u8; 16] = (*bytes)
                                .clone()
                                .try_into()
                                .unwrap();

                            let ip = IpAddr::from(
                                Ipv6Addr::from(octets)
                            );

                            Some(ip)
                        }
                        _ => None
                    }
                } else {
                    None
                }
            })
            .try_filter(|v| future::ready(v.is_some()))
            .filter_map(|v| future::ready(v.unwrap()));

        Ok(addrs.collect::<Vec<IpAddr>>().await)
    } else {
        Err(Error::LinkNotFound(link))
    }
}
