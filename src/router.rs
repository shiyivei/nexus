// the mod is about router
// each router has three kind of results: empty , wrong and valid
// empty and wrong are special cases, using a struct to wrap it up

mod empty_router;
mod route;

pub struct Router<S> {
    // Service
    svc: S,
}

// `EmptyRouter` is one kind of router
// Make `Router` have the method of `EmptyRouter` via generic constraints
impl<S> Router<EmptyRouter<E>> {
    // create a new router, default is not found
    pub fn new() -> Self {
        Self {
            svc: EmptyRouter::not_found(),
        }
    }

    //
}

impl<S> Router<S> {
    // add one more constraint T
    pub fn route<T>(self, description: &str, svc: T) -> Router<Route<T, S>> {
        todo!()
    }

    //     fn map<F, S2>(self, f: F) -> Router<S2>
    //     where
    //         F: Fnonce(S) -> S2,
    //     {
    //         Router { svc: f(self.svc) }
    //     }
}

impl<E> Default for Router<EmptyRouter<E>> {
    fn default() -> Self {
        Self::new()
    }
}

// Make `Router` have the methods of `Service` trait
impl<S, R> Service<R> for Router<S>
where
    S: Service<R>,
{
    // rename type
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    // Call the underlying service (svc)
    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.svc.call(cx)
    }
    // Call the underlying service (svc)
    #[inline]
    fn call(&mut self, req: R) -> Self::Future {
        self.svc.call(req)
    }
}
