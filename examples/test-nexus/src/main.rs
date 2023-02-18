// Design the APIs
use std::net::SocketAddr;

use color_eyre::Report;
use http::header::USER_AGENT;
use hyper::Body;
use nexus::{
    self,
    handler::{get, post},
    Router,
};
use tower_http::set_header::SetRequestHeaderLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod handlers;
use headers::HeaderValue;

use crate::handlers::{handler, page_handler, type_handler};

#[tokio::main]
async fn main() -> Result<(), Report> {
    setup()?;
    info!("nexus init...");
    // build application with a route
    let app = Router::new()
        // .route("/", post(handler))
        .route("/", get(type_handler))
        .route("/page", get(page_handler));
    // .layer(SetRequestHeaderLayer::<_, Body>::overriding(
    //     USER_AGENT,
    //     HeaderValue::from_static("nexus-http demo"),
    // ));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!(%addr,"Listening on: {}",addr);

    nexus::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

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
