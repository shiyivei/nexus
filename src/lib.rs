pub use async_trait::async_trait;
pub use http;
pub use hyper::Server;
pub use tower_http::add_extension::{AddExtension, AddExtensionLayer};

#[macro_use]
mod macros;
pub mod body;
mod error;
pub mod extract;
pub mod handler;
pub mod response;
pub mod router;
mod service;

mod util;

pub use self::router::Router;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
