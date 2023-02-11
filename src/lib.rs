pub use async_trait::async_trait;
pub use http;
pub use hyper::Server;
pub use tower_http::add_extension::{AddExtension, AddExtensionLayer};

#[macro_use]
mod macros;
mod body;
mod error;
mod router;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
