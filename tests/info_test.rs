#[cfg(not(feature = "async"))]
#[test]
fn test_info() {
    let client = a2s::A2SClient::new().unwrap();

    let result = client.info("play.maxdb.net:27015").unwrap();

    println!("{:?}", result);
}

#[cfg(not(feature = "async"))]
#[test]
fn test_info_theship() {
    let client = a2s::A2SClient::new().unwrap();

    let result = client.info("46.4.48.226:27017").unwrap();

    println!("{:?}", result);
    assert!(result.the_ship.is_some());
}
