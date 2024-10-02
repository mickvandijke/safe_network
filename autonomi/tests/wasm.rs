use std::time::Duration;

use crate::common::evm_wallet_from_env_or_default;
use autonomi::Client;
use evmlib::utils::evm_network_from_env_compile_time;
use tokio::time::sleep;
use wasm_bindgen_test::*;

mod common;

wasm_bindgen_test_configure!(run_in_browser);

#[allow(clippy::unwrap_used)]
#[wasm_bindgen_test]
async fn file() -> Result<(), Box<dyn std::error::Error>> {
    common::enable_logging();

    let peers = vec![
        "/ip4/127.0.0.1/tcp/35499/ws/p2p/12D3KooWGN5RqREZ4RYtsUc3DNCkrNSVXEzTYEbMb1AZx2rNddoW"
            .try_into()
            .expect("str to be valid multiaddr"),
    ];

    let mut client = Client::connect(&peers).await.unwrap();
    let network = evm_network_from_env_compile_time();
    let wallet = evm_wallet_from_env_or_default(network);

    let data = common::gen_random_data(1024 * 1024 * 10);

    let addr = client.put(data.clone(), &wallet).await.unwrap();

    sleep(Duration::from_secs(2)).await;

    let data_fetched = client.get(addr).await.unwrap();
    assert_eq!(data, data_fetched, "data fetched should match data put");

    Ok(())
}
