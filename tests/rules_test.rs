#[cfg(not(feature = "async"))]
#[test]
fn test_rules() {
    let client = a2s::A2SClient::new().unwrap();

    let result = client.rules("play.maxdb.net:27015").unwrap();

    println!("{:?}", result);
    assert!(result.into_iter().find(|rule| rule.name.chars().next().unwrap().is_numeric()).is_none());
}

#[cfg(not(feature = "async"))]
#[test]
fn test_rules_multipacket() {
    let client = a2s::A2SClient::new().unwrap();

    let result = client.rules("74.91.118.209:27015").unwrap();

    println!("{:?}", result);
    // In case of multi-packet responses, it might happen that the response is parsed incorrectly
    // so the names and values swap place (e.g. Rule { name: "10", value: "sv_airaccelerate" }).
    // We can usually catch this by checking for names starting with digits.
    assert!(result.into_iter().find(|rule| rule.name.chars().next().unwrap().is_numeric()).is_none());
}

#[cfg(not(feature = "async"))]
#[test]
fn test_rules_multipacket2() {
    let client = a2s::A2SClient::new().unwrap();

    let result = client.rules("188.165.244.220:27175").unwrap();

    println!("{:?}", result);
    assert!(result.into_iter().find(|rule| rule.name.chars().next().unwrap().is_numeric()).is_none());
}

#[cfg(not(feature = "async"))]
#[test]
fn test_rules_goldsource() {
    let client = a2s::A2SClient::new().unwrap();

    // Only servers providing multipacket responses are relevant for GoldSource tests
    let result = client.rules("45.83.244.193:27015").unwrap();

    println!("{:?}", result);
}
