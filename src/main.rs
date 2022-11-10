use clap::Parser;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Error;
use std::{net::SocketAddr, time::Duration};

mod metrics;

#[derive(Parser)]
#[clap(name = "Prober")]
struct Cli {
    #[clap(long, default_value = "10s")]
    polling_interval: humantime::Duration,

    #[clap(
        long,
        default_value = "https://g4xu7-jiaaa-aaaan-aaaaq-cai.raw.ic0.app/metrics"
    )]
    bitcoin_canister_metrics_endpoint: String,

    #[clap(long, default_value = "0.0.0.0:9090")]
    metrics_addr: SocketAddr,
}

/// Current block height in the longest chain.
async fn get_block_count() -> Result<u32, Error> {
    let url = "https://blockchain.info/q/getblockcount";
    let response = reqwest::get(url).await?;
    let text = response.text().await?;
    let height = text.parse::<u32>().unwrap();
    Ok(height)
}

/*
# HELP main_chain_height Height of the main chain.
# TYPE main_chain_height gauge
main_chain_height 2405670 1668084050769
 */
async fn bitcoin_canister_height(url: &String) -> Result<u32, Error> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\nmain_chain_height (\d+) (\d+)\n").unwrap();
    }

    let response = reqwest::get(url).await?;
    let text = response.text().await?;
    match RE.is_match(&text) {
        false => Ok(0),
        true => {
            let cap = RE.captures(&text).unwrap();
            let height = String::from(&cap[1]).parse::<u32>().unwrap();
            Ok(height)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    metrics::run_server();

    loop {
        let blockchain_height = get_block_count().await?;
        let bitcoin_canister_height =
            bitcoin_canister_height(&cli.bitcoin_canister_metrics_endpoint).await?;

        println!("{blockchain_height:?}");
        println!("{bitcoin_canister_height:?}");

        let delay = Duration::from_secs(5);
        tokio::time::sleep(delay).await;
    }
}
