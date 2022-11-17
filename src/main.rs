mod client;
mod schema;
mod server;

use clap::Parser;

/// Simple Async IRC server written in Rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value_t = String::from("localhost"))]
    bind_addr: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 50001)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    server::start_server((args.bind_addr, args.port)).await;
}
