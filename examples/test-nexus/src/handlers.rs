use http::header::USER_AGENT;
use nexus::{
    extract::builtin::{query::Query, typed_header::TypedHeader},
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::info;
pub async fn type_handler(user_agent: TypedHeader<headers::UserAgent>) -> impl IntoResponse {
    let url = "localhost";

    //     if let TypedHeader(u_g) = user_agent {}

    tracing::info!(%url,user_agent =?user_agent.as_str(),"Got a connection");

    let res = "<h1>hello,world<h1>".into_response();

    //     info!(%url,content_type = ?res.headers().get(USER_AGENT),"Got a
    //     response");
    res
}

pub async fn handler() -> &'static str {
    "<h1>hello,world<h1>"
}

#[derive(Debug, Deserialize)]
pub struct Pagination {
    page: usize,
    per_page: usize,
}

pub async fn page_handler(pagination: Query<Pagination>) -> &'static str {
    let pagination = pagination.0;

    info!(?pagination, "Got a connection!");

    "<h1> Hello, World!<h1>"
}
