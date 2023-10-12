use clap::Parser;
use futures::StreamExt;
use hyper::body::Bytes;
use hyper::{Body, Client, Request, Uri};
use rand::Rng;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URI to send requests to
    uri: Uri,

    #[arg(short, long, default_value_t = 100 * 1024)]
    payload_size: usize,

    /// Number of roundtrips
    #[arg(long, default_value_t = 100_000)]
    count: usize,

    /// Maximum concurrency
    #[arg(long, default_value_t = 10)]
    concurrency: usize,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!("Connecting to {}", args.uri);

    let mut rng = rand::thread_rng();
    let client = Client::new();
    let data: Bytes = (0..args.payload_size).map(|_| rng.gen()).collect();

    let mut stream = futures::stream::iter(0..args.count)
        .map(|_| async {
            let mut request = Request::new(Body::from(data.clone()));
            *request.uri_mut() = args.uri.clone();

            let start = Instant::now();
            let response = client.request(request).await?;
            hyper::body::to_bytes(response).await?;
            anyhow::Ok(start.elapsed())
        })
        .buffered(args.concurrency);

    while let Some(result) = stream.next().await {
        match result {
            Ok(duration) => println!("{}", duration.as_secs_f64()),
            Err(e) => eprintln!("{e}"),
        }
    }
}
