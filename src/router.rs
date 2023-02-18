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
    sync::Arc,
    task::{Context, Poll},
};

use http::{Request, Response, StatusCode};
use tower::util::ServiceExt;
use tower_service::Service;

use self::{
    empty_router::{EmptyRouter, FromEmptyRouter},
    future::EmptyRouterFuture,
    route::{PathPattern, Route},
};
use crate::{body::BoxBody, service::HandleError};

#[derive(Debug, Clone)]
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

    pub fn into_make_service(self) -> IntoMakeService<S>
    where
        S: Clone,
    {
        IntoMakeService::new(self.svc)
    }

    pub fn layer<L>(self, layer: L) -> Router<Layered<L::Service>>
    where
        L: tower_layer::Layer<S>,
    {
        self.map(|svc| Layered::new(layer.layer(svc)))
    }

    pub fn handle_error<ReqBody, F>(self, f: F) -> Router<HandleError<S, F, ReqBody>> {
        self.map(|svc| HandleError::new(svc, f))
    }

    pub fn check_infallible(self) -> Router<CheckInfallible<S>> {
        self.map(CheckInfallible)
    }
}

pub struct Layered<S> {
    inner: S,
}
impl<S> Layered<S> {
    fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S> Clone for Layered<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self::new(self.inner.clone())
    }
}

impl<S> fmt::Debug for Layered<S>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Layered")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<S, R> Service<R> for Layered<S>
where
    S: Service<R>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, req: R) -> Self::Future {
        self.inner.call(req)
    }
}
#[derive(Debug, Clone)]
pub struct IntoMakeService<S> {
    service: S,
}

impl<S> IntoMakeService<S> {
    fn new(svc: S) -> Self {
        Self { service: svc }
    }
}
impl<S, T> Service<T> for IntoMakeService<S>
where
    S: Clone,
{
    type Response = S;
    type Error = Infallible;
    type Future = future::MakeRouteServiceFuture<S>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _target: T) -> Self::Future {
        future::MakeRouteServiceFuture {
            future: ready(Ok(self.service.clone())),
        }
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

pub struct CheckInfallible<S>(S);

impl<R, S> Service<R> for CheckInfallible<S>
where
    S: Service<R, Error = Infallible>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    #[inline]
    fn call(&mut self, req: R) -> Self::Future {
        self.0.call(req)
    }
}
