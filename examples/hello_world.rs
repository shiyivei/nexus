// Design the APIs
use std::net::SocketAddr;

use nexus;

async fn hello() -> &'static str {
    "<h1> Hello, World! </h>"
}

#[tokio::main]
async fn main() {
    // build application with a route
    let app = nexus::app().route("/", get(hello));

    // run application
    nexus::start(([126, 0, 0, 1], 3000))
        .serve(app.into_make_service())
        .await
        .unwrap();
}
