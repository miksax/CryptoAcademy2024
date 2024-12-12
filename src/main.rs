use pallas::{
    ledger::traverse::{header, MultiEraBlock, MultiEraHeader},
    network::miniprotocols::Point,
};
use tracing_subscriber::prelude::*;

pub fn process_block(block: &MultiEraBlock) {
    tracing::debug!("Received block: {:?}", block);
}

#[tokio::main]
async fn main() {
    let fmt_layer = tracing_subscriber::fmt::layer().with_line_number(true);
    let filter = tracing_subscriber::filter::Targets::new()
        .with_target(env!("CARGO_PKG_NAME"), tracing::Level::DEBUG)
        .with_target(env!("CARGO_BIN_NAME"), tracing::Level::DEBUG);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter)
        .init();

    let ip: std::net::IpAddr = std::net::IpAddr::V4("51.195.7.40".parse().unwrap());
    let port: u16 = 10002;
    let magic: u64 = 1;

    let mut peer =
        pallas::network::facades::PeerClient::connect(std::net::SocketAddr::new(ip, port), magic)
            .await
            .unwrap();

    tracing::debug!("Fetch know blocks");

    for block_cbor in peer
        .blockfetch()
        .fetch_range((
            Point::Specific(
                78318568,
                hex::decode("10aad4a0d5e9f01661c7c706738bdefa3541f59a5ea871e1d22a3865e4076aed")
                    .unwrap(),
            ),
            Point::Specific(
                78318655,
                hex::decode("cc61baa19a797912f1737c137982e5348fa397e05e15b5ce1cb79b1d4b4150c2")
                    .unwrap(),
            ),
        ))
        .await
        .unwrap()
    {
        if let Ok(block) = pallas::ledger::traverse::MultiEraBlock::decode(&block_cbor) {
            process_block(&block);
        }
    }

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    tracing::debug!("Fetch new blocks");
    loop {
        match peer.chainsync().request_or_await_next().await {
            Ok(pallas::network::miniprotocols::chainsync::NextResponse::RollForward(
                header_cbor,
                tip,
            )) => {
                if let Ok(header) =
                    MultiEraHeader::decode(header_cbor.variant, None, &header_cbor.cbor)
                {
                    if let Ok(block_cbor) = peer
                        .blockfetch()
                        .fetch_single(Point::Specific(header.slot(), header.hash().to_vec()))
                        .await
                    {
                        if let Ok(block) =
                            pallas::ledger::traverse::MultiEraBlock::decode(&block_cbor)
                        {
                            process_block(&block);
                        }
                    }
                }
            }
            Ok(pallas::network::miniprotocols::chainsync::NextResponse::RollBackward(
                point,
                tip,
            )) => {}
            Ok(pallas::network::miniprotocols::chainsync::NextResponse::Await) => {}
            Err(err) => {
                tracing::error!("{}", err);
            }
        }
    }
}
