extern crate a2s;

#[test]
fn test_info() {
    let client = a2s::A2SClient::new().unwrap();

    let result = client.info("play.maxdb.net:27015").unwrap();

    println!("{:?}", result);
}