use super::*;

pub struct Route<S, F> {
    pub(crate) pattern: PathPattern, // parse url and match route
    pub(crate) svc: S,               // service
    pub(crate) fallback: F,          // back to 404 or SPA application
}

impl<S, F, B> Service<Request<B>> for Route<S, F>
where
    S: Service<Request<B>, Response = Response<BoxBody>> + Clone,
    F: Service<Request<B>, Response = Response<BoxBody>, Error = S::Error> + Clone,
    B: Send + Sync + 'static,
{
    type Response = Response<BoxBody>;
    type Error = S::Error;
    type Future = RouteFuture<S, F, B>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::ready(Ok(()))
    }

    // call and return, choose the handle
    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        todo!()
    }
}
