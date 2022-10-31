use linkaddrs::Result;

#[test]
fn test_addresses() -> Result<()> {
    let addrs = linkaddrs::addresses(String::from("lo"))?;
    println!("{:?}", addrs);

    Ok(())
}

#[test]
fn test_all_addresses() -> Result<()> {
    let addrs = linkaddrs::all_addresses()?;
    println!("{:?}", addrs);

    Ok(())
}
