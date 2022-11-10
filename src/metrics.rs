use hyper::{
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};
use lazy_static::lazy_static;
use prometheus::{opts, register_gauge};
use prometheus::{Encoder, Gauge, TextEncoder};
use std::net::SocketAddr;

lazy_static! {
    static ref BITCOIN_BLOCK_HEIGHT: Gauge = register_gauge!(opts!(
        "bitcoin_block_height",
        "Bitcoin block height in the longest chain.",
    ))
    .unwrap();
    static ref BITCOIN_CANISTER_BLOCK_HEIGHT: Gauge = register_gauge!(opts!(
        "bitcoin_canister_block_height",
        "Bitcoin canister block height in the longest chain.",
    ))
    .unwrap();
    static ref BLOCK_HEIGHT_DIFFERENCE: Gauge = register_gauge!(opts!(
        "block_height_difference",
        "Block height difference between bitcoin and bitcoin canister.",
    ))
    .unwrap();
}

pub fn set_bitcoin_block_height(height: u32) {
    BITCOIN_BLOCK_HEIGHT.set(height as f64);
}

pub fn set_bitcoin_canister_block_height(height: u32) {
    BITCOIN_CANISTER_BLOCK_HEIGHT.set(height as f64);
}

pub fn set_block_height_difference(height_difference: i32) {
    BLOCK_HEIGHT_DIFFERENCE.set(height_difference as f64);
}

async fn handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/metrics") => {
            let encoder = TextEncoder::new();

            let metric_families = prometheus::gather();
            let mut buffer = vec![];
            encoder.encode(&metric_families, &mut buffer).unwrap();

            let response = Response::builder()
                .status(200)
                .header(CONTENT_TYPE, encoder.format_type())
                .body(Body::from(buffer))
                .unwrap();

            Ok(response)
        }
        _ => {
            let buffer = vec![];
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(buffer))
                .unwrap();
            Ok(response)
        }
    }
}

pub fn run_server(addr: SocketAddr) {
    tokio::spawn(async move {
        println!("Exposing metrics on http://{}/metrics", addr);

        let serve_future = Server::bind(&addr).serve(make_service_fn(|_| async {
            Ok::<_, hyper::Error>(service_fn(handler))
        }));

        if let Err(err) = serve_future.await {
            eprintln!("server error: {}", err);
        }
    });
}
