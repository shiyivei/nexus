use async_trait::async_trait;
use bytes::Bytes;
use futures_util::stream::Stream;
use http::Request;

use super::{FromRequest, RequestParts, *};
use crate::{extract::rejection::RequestAlreadyExtracted, BoxError};

pub struct Body<B = crate::body::Body>(pub B);

#[async_trait]
impl<B> FromRequest<B> for Request<B>
where
    B: Send,
{
    type Rejection = RequestAlreadyExtracted;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let req = std::mem::replace(
            req,
            RequestParts {
                method: req.method.clone(),
                version: req.version,
                uri: req.uri.clone(),
                headers: None,
                extensions: None,
                body: None,
            },
        );

        let err = match req.try_into_request() {
            Ok(req) => return Ok(req),
            Err(err) => err,
        };

        match err.downcast::<RequestAlreadyExtracted>() {
          Ok(err) => return Err(err),
          Err(err) => unreachable! (

               "Unexpected error type from  `try_into_request`: `{:?}`. This is bug in nexus, please file  the issue",
               err
          )
        }
    }
}
