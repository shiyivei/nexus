use std::marker::PhantomData;

use super::*;

pub struct EmptyRouter<E = Infallible> {
    // Defined by http crate
    status: StatusCode,
    // Define E but didn't use, using PhantomData to skip ownership
    // Indicate that `EmptyRouter` doesn't have generic E
    _marker: PhantomData<fn() -> E>,
}

impl<E> EmptyRouter<E> {
    pub(crate) fn not_found() -> Self {
        Self {
            // 404
            status: StatusCode::NOT_FOUND,
            _marker: PhantomData,
        }
    }

    pub(crate) fn method_not_found() -> Self {
        Self {
            // 405
            status: StatusCode::METHOD_NOT_FOUND,
            _marker: PhantomData,
        }
    }
}

//  clone
impl<E> Clone for EmptyRouter<E> {
    fn clone(&self) -> Self {
        Self {
            status: self.status,
            _marker: PhantomData,
        }
    }
}

// debug
impl<E> Debug for EmptyRouter<E> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("EmptyRouter").finish()
    }
}

impl<B, E> Service<Request<B>> for EmptyRouter<E>
where
    B: Send + Send + Sync + 'static,
{
    // rename type
    type Response = Response<BoxBody>;
    type Error = E;
    type Future = EmptyRouterFuture<E>;

    #[inline]
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    // match response with status_code
    #[inline]
    fn call(&mut self, mut request: Request<B>) -> Self::Future {
        // 405
        if self.status = StatusCode::METHOD_NOT_FOUND {
            request.extensions_mut().insert(NoMethodMatch)
        }

        // change to 405

        if self.status == StatusCode::NOT_FOUND
            && request.extensions().get::<NoMethodMatch>().is_some()
        {
            self.status = StatusCode::METHOD_NOT_ALLOWED
        }

        // create an empty body
        let mut res = Response::new(crate::body::empty());
        // insert request
        res.extensions().insert(FromEmptyRouter { request });
        // change status
        *res.status_mut() = self.status;

        EmptyRouterFuture {
            future: ready(Ok(res)),
        }
    }
}

#[derive(Copy, Clone)]
struct NoMethodMatch;

pub struct FromEmptyRouter<B> {
    pub request: Request<B>,
}
