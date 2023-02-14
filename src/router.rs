// the mod is about router
// each router has three kind of results: empty , wrong and valid
// empty and wrong are special cases, using a struct to wrap it up

pub mod empty_router;

pub mod future;
pub mod method_filter;
pub mod route;

use std::{
    borrow::Cow,
    convert::Infallible,
    fmt,
    future::ready,
    marker::PhantomData,
    sync::Arc,
    task::{Context, Poll},
};

use http::{Request, Response, StatusCode, Uri};
use tower::{
    util::{BoxService, ServiceExt},
    ServiceBuilder,
};
use tower_http::map_response_body::MapResponseBodyLayer;
use tower_layer::Layer;
use tower_service::Service;

use self::{
    empty_router::{EmptyRouter, FromEmptyRouter},
    future::EmptyRouterFuture,
    route::{PathPattern, Route},
};
use crate::{
    body::{box_body, BoxBody},
    BoxError,
};
pub struct Router<S> {
    // Service
    svc: S,
}

// `EmptyRouter` is one kind of router
// Make `Router` have the method of `EmptyRouter` via generic constraints
impl<E> Router<EmptyRouter<E>> {
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
    pub fn route<T>(self, path: &str, svc: T) -> Router<Route<T, S>> {
        self.map(|fallback| Route {
            pattern: PathPattern::new(path),
            svc,
            fallback,
        })
    }

    fn map<F, S2>(self, f: F) -> Router<S2>
    where
        F: FnOnce(S) -> S2,
    {
        Router { svc: f(self.svc) }
    }
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
        self.svc.poll_ready(cx)
    }
    // Call the underlying service (svc)
    #[inline]
    fn call(&mut self, req: R) -> Self::Future {
        self.svc.call(req)
    }
}
