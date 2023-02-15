// Design the APIs
use std::net::SocketAddr;

use color_eyre::Report;
use nexus::{handler::get, Router};
use tracing::info;
use tracing_subscriber::EnvFilter;

async fn handler() -> &'static str {
    "<h1> Hello, World! </h>"
}

#[tokio::main]
async fn main() -> Result<(), Report> {
    setup()?;
    // build application with a route
    let app = Router::new().route("/", get(handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!(%addr,"Listening on: {}",addr);

    nexus::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

fn setup() -> Result<(), Report> {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }
    color_eyre::install()?;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }

    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    Ok(())
}
