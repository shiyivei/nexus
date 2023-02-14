use core::marker::PhantomData;
use std::convert::Infallible;

use super::*;
use crate::body::BoxBody;

pub(crate) mod sealed {

    #![allow(unreachable_pub, missing_docs, missing_debug_implementations)]

    pub trait HiddenTrait {}
    pub struct Hidden;

    impl HiddenTrait for Hidden {}
}

// 异步 trait 等价于返回 Future

#[async_trait]
pub trait Handler<B, T>: Clone + Send + Sized + 'static {
    #[doc(hidden)]
    type Sealed: sealed::HiddenTrait;

    async fn call(self, req: Request<B>) -> Response<BoxBody>;

    fn into_service(self) -> IntoService<Self, B, T> {
        IntoService::new(self)
    }
}

pub struct OnMethod<H, B, T, F> {
    pub(crate) method: MethodFilter,
    pub(crate) handler: H,
    pub(crate) fallback: F,
    pub(crate) _marker: PhantomData<fn() -> (B, T)>,
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

    pub fn get<H2, T2>(self, handler: H2) -> OnMethod<H2, B, T2, Self>
    where
        H2: Handler<B, T2>,
    {
        self.on(MethodFilter::GET | MethodFilter::HEAD, handler)
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
            let fut = Handler.call(self.handler.clone(), req);
            Either::A { inner: fut }
        } else {
            let fut = self.fallback.clone();
            oneshot(req);
            Either::B { inner: fut }
        };

        future::OnMethodFuture {
            inner: fut,
            req_method,
        }
    }
}
