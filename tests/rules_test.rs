#[cfg(not(feature = "async"))]
#[test]
fn test_rules() {
    let client = a2s::A2SClient::new().unwrap();

    let result = client.rules("play.maxdb.net:27015").unwrap();

    println!("{:?}", result);
}

#[cfg(not(feature = "async"))]
#[test]
fn test_rules_multipacket() {
    let client = a2s::A2SClient::new().unwrap();

    let result = client.rules("74.91.118.209:27015").unwrap();

    println!("{:?}", result);
}

#[cfg(not(feature = "async"))]
#[test]
fn test_rules_multipacket2() {
    let client = a2s::A2SClient::new().unwrap();

    let result = client.rules("188.165.244.220:27175").unwrap();

    println!("{:?}", result);
}
