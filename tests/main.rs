use linkaddrs::Result;

#[test]
fn test_addresses() -> Result<()> {
    let addrs = linkaddrs::addresses(String::from("lo"))?;
    println!("{:?}", addrs);

    Ok(())
}
