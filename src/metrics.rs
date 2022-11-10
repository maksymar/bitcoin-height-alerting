use hyper::{
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};
//use lazy_static::lazy_static;
use prometheus::{labels, opts, register_counter};
use prometheus::{Counter, Encoder, TextEncoder};

// lazy_static! {
//     static ref CURRENT_BLOCK_HEIGHT: Counter = register_counter!(opts!(
//         "bot_requests_total",
//         "Number of bot requests received.",
//         labels! {"handler" => "all",}
//     ))
//     .unwrap();
// }
// pub fn bot_requests_counter_inc() {
//     CURRENT_BLOCK_HEIGHT.inc();
// }

async fn handler(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    // println!(
    //     "ABC serve_req method={}, uri={}",
    //     req.method(),
    //     req.uri().path()
    // );

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

pub fn run_server() {
    tokio::spawn(async move {
        // https://stackoverflow.com/questions/39525820/docker-port-forwarding-not-working
        // Replace 127.0.0.1 with 0.0.0.0.
        let addr = ([0, 0, 0, 0], 8008).into();
        println!("Listening on http://{}", addr);

        let serve_future = Server::bind(&addr).serve(make_service_fn(|_| async {
            Ok::<_, hyper::Error>(service_fn(handler))
        }));

        if let Err(err) = serve_future.await {
            eprintln!("server error: {}", err);
        }
    });
}
