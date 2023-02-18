use std::convert::Infallible;

use async_trait::async_trait;
use http::{header, Extensions, HeaderMap, Method, Request, Uri, Version};

use self::rejection::{
    BodyAlreadyExtracted, ExtensionAlreadyExtracted, HeadersAlreadyExtracted,
    RequestAlreadyExtracted,
};
use crate::{error::Error, response::IntoResponse};

pub mod builtin;
pub mod rejection;
pub mod request_parts;

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

    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.method
    }

    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    pub fn uri_mut(&mut self) -> &mut Uri {
        &mut self.uri
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.version
    }

    pub fn headers(&self) -> Option<&HeaderMap> {
        self.headers.as_ref()
    }

    pub fn headers_mut(&mut self) -> Option<&mut HeaderMap> {
        self.headers_mut()
    }

    pub fn take_headers(&mut self) -> Option<HeaderMap> {
        self.headers.take()
    }

    pub fn extensions(&self) -> Option<&Extensions> {
        self.extensions.as_ref()
    }

    pub fn extensions_mut(&mut self) -> Option<&mut Extensions> {
        self.extensions.as_mut()
    }

    pub fn take_extensions(&mut self) -> Option<Extensions> {
        self.extensions.take()
    }

    pub fn body(&self) -> Option<&B> {
        self.body.as_ref()
    }

    pub fn body_mut(&mut self) -> Option<&mut B> {
        self.body.as_mut()
    }

    pub fn take_body(&mut self) -> Option<B> {
        self.body.take()
    }
}

#[async_trait]
impl<T, B> FromRequest<B> for Result<T, T::Rejection>
where
    T: FromRequest<B>,
    B: Send,
{
    type Rejection = Infallible;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        Ok(T::from_request(req).await)
    }
}

pub(crate) fn has_content_type<B>(
    req: &RequestParts<B>,
    expected_content_type: &str,
) -> Result<bool, HeadersAlreadyExtracted> {
    let content_type = if let Some(content_type) = req
        .headers()
        .ok_or(HeadersAlreadyExtracted)?
        .get(header::CONTENT_TYPE)
    {
        content_type
    } else {
        return Ok(false);
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return Ok(false);
    };

    Ok(content_type.starts_with(expected_content_type))
}

pub(crate) fn take_body<B>(req: &mut RequestParts<B>) -> Result<B, BodyAlreadyExtracted> {
    req.take_body().ok_or(BodyAlreadyExtracted)
}
