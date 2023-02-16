use async_trait::async_trait;
use http::{header, Extensions, HeaderMap, Method, Request, Uri, Version};

use self::rejection::{
    BodyAlreadyExtracted, ExtensionAlreadyExtracted, HeadersAlreadyExtracted,
    RequestAlreadyExtracted,
};
use crate::{error::Error, response::IntoResponse};

pub mod builtin;
mod rejection;
mod request_parts;

#[async_trait]

pub trait FromRequest<B = crate::body::Body>: Sized {
    type Rejection: IntoResponse;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection>;
}

#[derive(Debug)]
pub struct RequestParts<B = crate::body::Body> {
    method: Method,
    uri: Uri,
    version: Version,
    headers: Option<HeaderMap>,
    extensions: Option<Extensions>,
    body: Option<B>,
}

impl<B> RequestParts<B> {
    pub fn new(req: Request<B>) -> Self {
        let (
            http::request::Parts {
                method,
                uri,
                version,
                headers,
                extensions,
                ..
            },
            body,
        ) = req.into_parts();

        RequestParts {
            method,
            uri,
            version,
            headers: Some(headers),
            extensions: Some(extensions),
            body: Some(body),
        }
    }

    pub fn try_into_request(self) -> Result<Request<B>, Error> {
        let Self {
            method,
            uri,
            version,
            mut headers,
            mut extensions,
            mut body,
        } = self;

        let mut req = if let Some(body) = body.take() {
            Request::new(body)
        } else {
            return Err(Error::new(RequestAlreadyExtracted::BodyAlreadyExtracted(
                BodyAlreadyExtracted,
            )));
        };

        *req.method_mut() = method;
        *req.uri_mut() = uri;
        *req.version_mut() = version;

        if let Some(headers) = headers.take() {
            *req.headers_mut() = headers;
        } else {
            return Err(Error::new(
                RequestAlreadyExtracted::HeadersAlreadyExtracted(HeadersAlreadyExtracted),
            ));
        }

        if let Some(extensions) = extensions.take() {
            *req.extensions_mut() = extensions;
        } else {
            return Err(Error::new(
                RequestAlreadyExtracted::ExtensionAlreadyExtracted(ExtensionAlreadyExtracted),
            ));
        }

        Ok(req)
    }
}
