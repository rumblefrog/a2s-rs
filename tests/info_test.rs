extern crate a2s_rs;

#[test]
fn test_info() {
    let client = a2s_rs::A2SClient::new().unwrap();

    let result = client.info("play.maxdb.net:27015").unwrap();

    println!("{:?}", result);
}