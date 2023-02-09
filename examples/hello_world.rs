// Design the APIs

use std::net::SocketAddr;

use nexus::{handler::get, Router};

#[tokio::main]
async fn main() {
    // build application with a route
    let app = Router::new().route("/", get(handler));

    // run application

    let addr = SocketAddr::from(([126, 0, 0, 1], 3000));
    println!("Listening on {}", addr);
    nexus::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> &'static str {
    "<h1> Hello, World! </h>"
}
