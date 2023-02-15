use std::{
    convert::Infallible,
    fmt,
    marker::PhantomData,
    task::{Context, Poll},
};

use http::{Request, Response};
use tower_service::Service;

use super::Handler;
use crate::body::BoxBody;

pub struct IntoService<H, B, T> {
    handler: H,
    _marker: PhantomData<fn() -> (B, T)>,
}

impl<H, B, T> IntoService<H, B, T> {
    pub(super) fn new(handler: H) -> Self {
        Self {
            handler,
            _marker: PhantomData,
        }
    }
}

impl<H, B, T> fmt::Debug for IntoService<H, B, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IntoService")
            .field(&format_args!("..."))
            .finish()
    }
}

impl<H, B, T> Clone for IntoService<H, B, T>
where
    H: Clone,
{
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            _marker: PhantomData,
        }
    }
}

impl<H, T, B> Service<Request<B>> for IntoService<H, B, T>
where
    H: Handler<B, T> + Clone + Send + 'static,
    B: Send + 'static,
{
    type Response = Response<BoxBody>;
    type Error = Infallible;
    type Future = super::future::IntoServiceFuture;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        use futures_util::future::FutureExt;

        let handler = self.handler.clone();
        let future = Handler::call(handler, req).map(Ok::<_, Infallible> as _);

        super::future::IntoServiceFuture { future }
    }
}
