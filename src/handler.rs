use core::marker::PhantomData;
use std::{
    convert::Infallible,
    fmt,
    future::Future,
    task::{Context, Poll},
};

use bytes::Bytes;
use headers::UserAgent;
use http::header::USER_AGENT;
use tower::ServiceExt;

use self::into_service::IntoService;
use crate::{
    body::{box_body, BoxBody},
    extract::{
        builtin::{query::Query, typed_header::TypedHeader},
        FromRequest,
    },
    response::IntoResponse,
    service::HandleError,
    util::Either,
    BoxError,
};

mod future;
mod into_service;

pub(crate) mod sealed {

    #![allow(unreachable_pub, missing_docs, missing_debug_implementations)]

    pub trait HiddenTrait {}
    pub struct Hidden;

    impl HiddenTrait for Hidden {}
}
use async_trait::async_trait;
use http::{Request, Response};
use tower_service::Service;

use crate::router::{empty_router::EmptyRouter, method_filter::MethodFilter};

pub struct OnMethod<H, B, T, F> {
    pub(crate) method: MethodFilter,
    pub(crate) handler: H,
    pub(crate) fallback: F,
    pub(crate) _marker: PhantomData<fn() -> (B, T)>,
}

impl<H, B, T, F> Clone for OnMethod<H, B, T, F>
where
    H: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            method: self.method,
            handler: self.handler.clone(),
            fallback: self.fallback.clone(),
            _marker: PhantomData,
        }
    }
}
impl<H, B, T, F> Copy for OnMethod<H, B, T, F>
where
    H: Copy,
    F: Copy,
{
}
pub fn on<H, B, T>(method: MethodFilter, handler: H) -> OnMethod<H, B, T, EmptyRouter>
where
    H: Handler<B, T>,
{
    OnMethod {
        method,
        handler,
        fallback: EmptyRouter::method_not_allowed(),
        _marker: PhantomData,
    }
}

pub fn get<H, B, T>(handler: H) -> OnMethod<H, B, T, EmptyRouter>
where
    H: Handler<B, T>,
{
    on(MethodFilter::GET | MethodFilter::HEAD, handler)
}

pub fn post<H, B, T>(handler: H) -> OnMethod<H, B, T, EmptyRouter>
where
    H: Handler<B, T>,
{
    on(MethodFilter::POST, handler)
}

// 链式调用的方法使用关联函数来处理
impl<H, B, T, F> OnMethod<H, B, T, F> {
    pub fn any<H2, T2>(self, handler: H2) -> OnMethod<H2, B, T2, Self>
    where
        H2: Handler<B, T2>,
    {
        self.on(MethodFilter::all(), handler)
    }

    pub fn connect<H2, T2>(self, handler: H2) -> OnMethod<H2, B, T2, Self>
    where
        H2: Handler<B, T2>,
    {
        self.on(MethodFilter::CONNECT, handler)
    }

    pub fn delete<H2, T2>(self, handler: H2) -> OnMethod<H2, B, T2, Self>
    where
        H2: Handler<B, T2>,
    {
        self.on(MethodFilter::DELETE, handler)
    }

    pub fn get<H2, T2>(self, handler: H2) -> OnMethod<H2, B, T2, Self>
    where
        H2: Handler<B, T2>,
    {
        self.on(MethodFilter::GET | MethodFilter::HEAD, handler)
    }

    pub fn head<H2, T2>(self, handler: H2) -> OnMethod<H2, B, T2, Self>
    where
        H2: Handler<B, T2>,
    {
        self.on(MethodFilter::HEAD, handler)
    }

    pub fn options<H2, T2>(self, handler: H2) -> OnMethod<H2, B, T2, Self>
    where
        H2: Handler<B, T2>,
    {
        self.on(MethodFilter::OPTIONS, handler)
    }

    pub fn patch<H2, T2>(self, handler: H2) -> OnMethod<H2, B, T2, Self>
    where
        H2: Handler<B, T2>,
    {
        self.on(MethodFilter::PATCH, handler)
    }

    pub fn post<H2, T2>(self, handler: H2) -> OnMethod<H2, B, T2, Self>
    where
        H2: Handler<B, T2>,
    {
        self.on(MethodFilter::POST, handler)
    }

    pub fn put<H2, T2>(self, handler: H2) -> OnMethod<H2, B, T2, Self>
    where
        H2: Handler<B, T2>,
    {
        self.on(MethodFilter::PUT, handler)
    }

    pub fn trace<H2, T2>(self, handler: H2) -> OnMethod<H2, B, T2, Self>
    where
        H2: Handler<B, T2>,
    {
        self.on(MethodFilter::TRACE, handler)
    }

    pub fn on<H2, T2>(self, method: MethodFilter, handler: H2) -> OnMethod<H2, B, T2, Self>
    where
        H2: Handler<B, T2>,
    {
        OnMethod {
            method,
            handler,
            fallback: self,
            _marker: PhantomData,
        }
    }
}

impl<H, B, T, F> Service<Request<B>> for OnMethod<H, B, T, F>
where
    H: Handler<B, T>,
    F: Service<Request<B>, Response = Response<BoxBody>, Error = Infallible> + Clone,
    B: Send + 'static,
{
    type Response = Response<BoxBody>;
    type Error = Infallible;
    type Future = future::OnMethodFuture<F, B>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let req_method = req.method().clone();

        let fut = if self.method.matches(req.method()) {
            let fut = Handler::call(self.handler.clone(), req);
            Either::A { inner: fut }
        } else {
            let fut = self.fallback.clone().oneshot(req);
            Either::B { inner: fut }
        };

        future::OnMethodFuture {
            inner: fut,
            req_method,
        }
    }
}

// 异步 trait 等价于返回 Future
// Handler 系统
// B代表Body,T代表 handler类型
#[async_trait]
pub trait Handler<B, T>: Clone + Send + Sized + 'static {
    #[doc(hidden)]
    type Sealed: sealed::HiddenTrait;

    async fn call(self, req: Request<B>) -> Response<BoxBody>;

    fn into_service(self) -> IntoService<Self, B, T> {
        IntoService::new(self)
    }
}

// #[async_trait]
// impl<F, Fut, Res, B, T> Handler<B, (T,)> for F
// where
//     F: FnOnce(T) -> Fut + Clone + Send + Sync + 'static,
//     Fut: Future<Output = Res> + Send,

//     Res: IntoResponse,
//     T: FromRequest<B> + Send,
//     B: Send + 'static,
// {
//     type Sealed = sealed::Hidden;
//     async fn call(self, _req: Request<B>) -> Response<BoxBody> {
//         let mut req = crate::extract::RequestParts::new(_req);
//         let value = match T::from_request(&mut req).await {
//             Ok(value) => value,
//             Err(rejection) => return rejection.into_response().map(box_body),
//         };
//         self(value).await.into_response().map(box_body)
//     }
// }

#[async_trait]
impl<F, Fut, Res, B> Handler<B, ()> for F
where
    F: FnOnce() -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Res> + Send,
    Res: IntoResponse,
    B: Send + 'static,
    // T: crate::extract::FromRequest<B> + Send,
{
    type Sealed = sealed::Hidden;
    async fn call(self, req: Request<B>) -> Response<BoxBody> {
        self().await.into_response().map(box_body)
    }
}

macro_rules! impl_handler {
    () => {

    };

    ($head:ident,$($tail:ident),* $(,)?) => {

        #[async_trait]
        #[allow(non_snake_case)]
        impl<F,Fut,B,Res:IntoResponse,$head,$($tail,)*>
Handler<B,($head,$($tail,)*)>         for F
        where
        F:FnOnce($head,$($tail,)*) -> Fut + Clone + Send + Sync + 'static ,
        Fut:Future<Output = Res> + Send,
        B:Send + 'static,
        $head:FromRequest<B> + Send,
        $($tail:FromRequest<B> + Send,)*
        {
            type Sealed = sealed::Hidden;

            async fn call(self,req:Request<B>) -> Response<BoxBody> {
                let mut req = crate::extract::RequestParts::new(req);

                let $head = match $head::from_request(&mut req).await {
                    Ok(value) => value,
                    Err(rejection) => return
rejection.into_response().map(box_body)                 };

                $(
                    let $tail = match $tail::from_request(&mut req).await {
                    Ok(value) => value,
                    Err(rejection) => return
rejection.into_response().map(box_body),                     };
                )*

                let res = self($head,$($tail,)*).await;

                res.into_response().map(crate::body::box_body)
            }
        }

        impl_handler!($($tail,)*);

    };
}

impl_handler!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);

pub struct Layered<S, T> {
    svc: S,
    _input: PhantomData<fn() -> T>,
}

impl<S, T> fmt::Debug for Layered<S, T>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Layered").field("svc", &self.svc).finish()
    }
}

impl<S, T> Clone for Layered<S, T>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self::new(self.svc.clone())
    }
}

#[async_trait]
impl<S, T, ReqBody, ResBody> Handler<ReqBody, T> for Layered<S, T>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Error: IntoResponse,
    S::Future: Send,
    T: 'static,
    ReqBody: Send + 'static,
    ResBody: http_body::Body<Data = Bytes> + Send + Sync + 'static,
    ResBody::Error: Into<BoxError> + Send + Sync + 'static,
{
    type Sealed = sealed::Hidden;

    async fn call(self, req: Request<ReqBody>) -> Response<BoxBody> {
        match self
            .svc
            .oneshot(req)
            .await
            .map_err(IntoResponse::into_response)
        {
            Ok(res) => res.map(box_body),
            Err(res) => res.map(box_body),
        }
    }
}

impl<S, T> Layered<S, T> {
    pub(crate) fn new(svc: S) -> Self {
        Self {
            svc,
            _input: PhantomData,
        }
    }

    pub fn handle_error<F, ReqBody, ResBody, Res, E>(
        self,
        f: F,
    ) -> Layered<HandleError<S, F, ReqBody>, E>
    where
        S: Service<Request<ReqBody>, Response = Response<ResBody>>,
        F: FnOnce(S::Error) -> Result<Res, E>,
        Res: IntoResponse,
    {
        let svc = HandleError::new(self.svc, f);
        Layered::new(svc)
    }
}
