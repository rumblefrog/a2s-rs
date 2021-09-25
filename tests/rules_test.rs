#[cfg(not(feature = "async"))]
#[test]
fn test_rules() {
    let client = a2s::A2SClient::new().unwrap();

    let result = client.rules("play.maxdb.net:27015").unwrap();

    println!("{:?}", result);
}
