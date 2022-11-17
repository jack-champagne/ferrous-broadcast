mod client;
mod schema;
mod server;

#[tokio::main]
async fn main() {
    server::start_server(("localhost", 50001)).await;
}
