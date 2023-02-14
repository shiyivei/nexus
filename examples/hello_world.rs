// Design the APIs
use std::net::SocketAddr;

use nexus::{handler::get, Router};

async fn handler() -> &'static str {
    "<h1> Hello, World! </h>"
}

#[tokio::main]
async fn main() {
    // build application with a route
    let app = Router::new().route("/", get(handler));

    let addr = SocketAddr::from(([127.0.0 .1], 3000));
    println!("Listening on {}", addr);
    // run application
    nexus::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
