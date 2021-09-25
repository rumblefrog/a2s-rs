#[cfg(not(feature = "async"))]
#[test]
fn test_players() {
    let client = a2s::A2SClient::new().unwrap();

    let result = client.players("play.maxdb.net:27015").unwrap();

    println!("{:?}", result);
}
