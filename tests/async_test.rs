#[cfg(feature = "async")]
use a2s::A2SClient;
#[cfg(feature = "async")]
use futures::future;
#[cfg(feature = "async")]
use std::net::SocketAddr;
#[cfg(feature = "async")]
use tokio::net::lookup_host;
#[cfg(feature = "async")]
use tokio::try_join;

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_multiplequeries() {
    let address = "74.91.118.209:27015";
    let client = A2SClient::new().await.unwrap();
    let info = client.info(&address);
    let rules = client.rules(&address);
    let players = client.players(&address);
    let (info, rules, players) = try_join!(info, rules, players).unwrap();
    println!("{:?}\n{:?}\n{:?}", info, rules, players);
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_multipleservers() {
    let client = A2SClient::new().await.unwrap();
    let addresses = vec![
        "coralie.megabrutal.com:27015",
        "play.lifeisabug.com:27015",
        "play.maxdb.net:27015",
        "ebateam.eu:27019",
        "92.80.103.133:27021",
        "186.19.99.1:27024",
    ]
    .into_iter()
    .map(lookup_host);
    let addresses = future::join_all(addresses).await.into_iter().flat_map(|a| {
        a.unwrap().into_iter().flat_map(|sa| match sa {
            SocketAddr::V4(sa4) => Some(sa4),
            _ => None,
        })
    });
    let fut = addresses.map(|a| {
        println!("Addr: {a}");
        client.info(a)
    });
    let mut fut: Vec<_> = fut.map(Box::pin).collect();
    while !fut.is_empty() {
        let (query_result, _index, remaining) = future::select_all(fut).await;
        println!("Result: {:?}", query_result.unwrap());
        fut = remaining;
    }
}
