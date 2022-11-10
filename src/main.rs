use clap::Parser;
use metrics::{
    set_bitcoin_block_height, set_bitcoin_canister_block_height, set_block_height_difference,
};
use regex::Regex;
use std::net::SocketAddr;
use thiserror::Error;

mod metrics;

#[derive(Parser)]
#[clap(name = "Prober")]
struct Cli {
    #[clap(long, default_value = "10s")]
    polling_interval: humantime::Duration,

    #[clap(long, default_value = "https://blockchain.info/q/getblockcount")]
    bitcoin_block_height_endpoint: String,

    #[clap(
        long,
        default_value = "https://g4xu7-jiaaa-aaaan-aaaaq-cai.raw.ic0.app/metrics"
    )]
    bitcoin_canister_metrics_endpoint: String,

    #[clap(long, default_value = r"\nmain_chain_height (\d+) \d+\n")]
    bitcoin_canister_metric_regex: String,

    #[clap(long, default_value = "0.0.0.0:9090")]
    metrics_addr: SocketAddr,
}

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error(transparent)]
    ApiError(#[from] reqwest::Error),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    RegexError(#[from] regex::Error),

    #[error("Specified metric was not found")]
    NoMetricError,

    #[error("Incorrect regex: {message}")]
    IncorrectRegex { message: String },
}

/// Get the full response text from a GET request.
async fn text_response_from_get_request(url: &str) -> Result<String, ServiceError> {
    let response = reqwest::get(url).await?;
    let text = response.text().await?;
    Ok(text)
}

/// Fetch current bitcoin block height in the longest chain.
async fn fetch_bitcoin_block_height(url: &str) -> Result<u32, ServiceError> {
    let text = text_response_from_get_request(url).await?;
    let height = text.parse::<u32>()?;
    Ok(height)
}

/// Apply regex rule to a given text.
fn apply(re: &Regex, text: &str) -> Result<String, ServiceError> {
    match re.captures(text) {
        None => Err(ServiceError::NoMetricError),
        Some(cap) => match cap.len() {
            2 => Ok(String::from(&cap[1])),
            x => Err(ServiceError::IncorrectRegex {
                message: format!("expected 1 group exactly, provided {}", x),
            }),
        },
    }
}

/// Fetch current block height from a bitcoin canister.
async fn fetch_bitcoin_canister_height(re: &Regex, url: &str) -> Result<u32, ServiceError> {
    let text = text_response_from_get_request(url).await?;
    let matched = apply(re, &text)?;
    let height = matched.parse::<u32>()?;
    Ok(height)
}

#[tokio::main]
async fn main() -> Result<(), ServiceError> {
    let cli = Cli::parse();
    let metric_regex = Regex::new(&cli.bitcoin_canister_metric_regex)?;

    metrics::run_server(cli.metrics_addr);

    loop {
        let target_height = fetch_bitcoin_block_height(&cli.bitcoin_block_height_endpoint).await?;
        let observed_height =
            fetch_bitcoin_canister_height(&metric_regex, &cli.bitcoin_canister_metrics_endpoint)
                .await?;
        let height_diff = target_height as i32 - observed_height as i32;

        set_bitcoin_block_height(target_height);
        set_bitcoin_canister_block_height(observed_height);
        set_block_height_difference(height_diff);

        tokio::time::sleep(cli.polling_interval.into()).await;
    }
}
