use linkaddrs::Result;

#[test]
fn test_addresses() -> Result<()> {
    let addrs = linkaddrs::addresses(String::from("lo"))?;
    println!("{:?}", addrs);

    Ok(())
}

#[test]
fn test_ipv4_addresses() -> Result<()> {
    let addrs = linkaddrs::ipv4_addresses(String::from("lo"))?;
    println!("{:?}", addrs);

    Ok(())
}

#[test]
fn test_ipv6_addresses() -> Result<()> {
    let addrs = linkaddrs::ipv6_addresses(String::from("lo"))?;
    println!("{:?}", addrs);

    Ok(())
}

#[test]
fn test_all_addresses() -> Result<()> {
    let addrs = linkaddrs::all_addresses()?;
    println!("{:?}", addrs);

    Ok(())
}
