use std::{borrow::Cow, convert::Infallible, fmt};

use bytes::Bytes;
use http_body::{Empty, Full};

use super::IntoResponse;
use crate::{
    body::{box_body, BoxBody},
    error::Error,
    BoxError,
};

define_rejection! {
     #[status = INTERNAL_SERVER_ERROR]
     #[body = "Extensions taken by other extractor"]

     pub struct ExtensionAlreadyExtracted;
}

define_rejection! {
     #[status = INTERNAL_SERVER_ERROR]
     #[body = "Headers taken by other extractor"]

     pub struct HeadersAlreadyExtracted;
}

define_rejection! {
     #[status = BAD_REQUEST]
     #[body="Filed to parse the request body as JSON"]

     pub struct InvalidJsonBody(Error);
}

define_rejection! {
     #[status = BAD_REQUEST]
     #[body = "Expected request with `Content-Type:application/json`"]

     pub struct MissingJsonContentType;
}

define_rejection! {
     #[status = INTERNAL_SERVER_ERROR]
     #[body = "Missing request extension"]

     pub struct MissingExtension(Error);
}

define_rejection! {
     #[status = BAD_REQUEST]
     #[body = "Failed to buffer the request body"]
     pub struct FailedToBufferBody(Error);
}

define_rejection! {

     #[status = BAD_REQUEST]
     #[body = "Request body didn't contain valid UTF-8"]

     pub struct InvalidUtf8(Error);
}

define_rejection! {

     #[status = PAYLOAD_TOO_LARGE]
     #[body = "Request payload is too large"]

     pub struct PayloadTooLarge;

}

define_rejection! {
     #[status = LENGTH_REQUIRED]
     #[body = "Content length header is required"]

     pub struct LengthRequired;
}

define_rejection! {
     #[status = INTERNAL_SERVER_ERROR]
     #[body = "No url params found or matched the route. This is a bug in nexus,please open the issue"]

     pub struct MissingRouteParams;
}

define_rejection! {
     #[status = INTERNAL_SERVER_ERROR]
     #[body = "Cannot have two request body extractors for a single handler"]

     pub struct BodyAlreadyExtracted;
}

define_rejection! {
     #[status = BAD_REQUEST]
     #[body = "From requests must have `Content-Type:x-www-from-urlencoded`"]

     pub struct InvalidFormContentType;
}

#[derive(Debug)]
pub struct InvalidPathParam(String);

impl InvalidPathParam {
    pub(super) fn new(err: impl Into<String>) -> Self {
        InvalidPathParam(err.into())
    }
}

impl IntoResponse for InvalidPathParam {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> http::Response<Self::Body> {
        let mut res = http::Response::new(Full::from(self.to_string()));
        *res.status_mut() = http::StatusCode::BAD_REQUEST;
        res
    }
}

impl std::fmt::Display for InvalidPathParam {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "Invalid url param. {}", self.0)
    }
}

impl std::error::Error for InvalidPathParam {}

#[derive(Debug)]
pub struct FailedToDeserializeQueryString {
    error: Error,
    type_name: &'static str,
}

impl FailedToDeserializeQueryString {
    pub(super) fn new<T, E>(error: E) -> Self
    where
        E: Into<BoxError>,
    {
        FailedToDeserializeQueryString {
            error: Error::new(error),
            type_name: std::any::type_name::<T>(),
        }
    }
}

impl IntoResponse for FailedToDeserializeQueryString {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> http::Response<Self::Body> {
        let mut res = http::Response::new(Full::from(self.to_string()));
        *res.status_mut() = http::StatusCode::BAD_REQUEST;
        res
    }
}

impl std::fmt::Display for FailedToDeserializeQueryString {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            fmt,
            "Failed to deserialize query string, Expected something of type `{}`, Error: {}",
            self.type_name, self.error
        )
    }
}

impl std::error::Error for FailedToDeserializeQueryString {}

composite_rejection! {
     pub enum QueryRejection {
          FailedToDeserializeQueryString
     }
}

composite_rejection! {
     pub enum FromRejection {
          InvalidFormContentType,
          FailedToDeserializeQueryString,
          FailedToBufferBody,
          BodyAlreadyExtracted,
          HeadersAlreadyExtracted
     }
}

composite_rejection! {
     pub enum JsonRejection {
          MissingExtension,
          ExtensionAlreadyExtracted
     }
}

composite_rejection! {
     pub enum  PathParamsRejection {
          InvalidPathParam,
          MissingRouteParams,
     }
}

composite_rejection! {
     pub enum BytesRejection {
          BodyAlreadyExtracted,
          FailedToBufferBody
     }
}

composite_rejection! {
     pub enum StringRejection {
          BodyAlreadyExtracted,
          FailedToBufferBody,
          InvalidUtf8,
     }
}

composite_rejection! {
     pub enum RequestAlreadyExtracted {
          BodyAlreadyExtracted,
          HeadersAlreadyExtracted,
          ExtensionAlreadyExtracted
     }
}

#[derive(Debug)]
#[non_exhaustive]

pub enum ContentLengthLimitRejection<T> {
    #[allow(missing_docs)]
    PayloadTooLarge(PayloadTooLarge),

    #[allow(missing_docs)]
    LengthRequired(LengthRequired),

    #[allow(missing_docs)]
    HeadersAlreadyExtracted(HeadersAlreadyExtracted),

    #[allow(missing_docs)]
    Inner(T),
}

impl<T> IntoResponse for ContentLengthLimitRejection<T>
where
    T: IntoResponse,
{
    type Body = BoxBody;
    type BodyError = Error;

    fn into_response(self) -> http::Response<Self::Body> {
        match self {
            Self::PayloadTooLarge(inner) => inner.into_response().map(box_body),
            Self::LengthRequired(inner) => inner.into_response().map(box_body),
            Self::HeadersAlreadyExtracted(inner) => inner.into_response().map(box_body),
            Self::Inner(inner) => inner.into_response().map(box_body),
        }
    }
}

impl<T> std::fmt::Display for ContentLengthLimitRejection<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PayloadTooLarge(inner) => inner.fmt(f),
            Self::LengthRequired(inner) => inner.fmt(f),
            Self::HeadersAlreadyExtracted(inner) => inner.fmt(f),
            Self::Inner(inner) => inner.fmt(f),
        }
    }
}

impl<T> std::error::Error for ContentLengthLimitRejection<T>
where
    T: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PayloadTooLarge(inner) => Some(inner),
            Self::LengthRequired(inner) => Some(inner),
            Self::HeadersAlreadyExtracted(inner) => Some(inner),
            Self::Inner(inner) => Some(inner),
        }
    }
}

#[cfg(feature = "headers")]
#[cfg_attr(docsrs, doc(cfg(feature = "headers")))]
pub use super::builtin::typed_header::TypeHeaderRejection;
