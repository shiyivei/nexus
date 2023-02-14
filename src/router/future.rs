use std::{future::Future, pin::Pin, task::ready};

use pin_project_lite::pin_project;
use tower::util::Oneshot;

use super::*;

opaque_future! {
     pub type EmptyRouterFuture<E> = std::future::Ready<Result<Response<BoxBody>,E>>;
}

opaque_future! {
    pub type MakeRouteServiceFuture<S> =
     std::future::Ready<Result<S,Infallible>>;
}

pin_project! {
     #[derive(Debug)]
     pub struct RouteFuture<S,F,B>
     where
     S:Service<Request<B>>,
     F:Service<Request<B>>
     {
          #[pin]
          state:RouteFutureInner<S,F,B>
     }
}

impl<S, F, B> RouteFuture<S, F, B>
where
    S: Service<Request<B>>,
    F: Service<Request<B>>,
{
    pub(crate) fn a(a: Oneshot<S, Request<B>>, fallback: F) -> Self {
        RouteFuture {
            state: RouteFutureInner::A {
                a,
                fallback: Some(fallback),
            },
        }
    }

    pub(crate) fn b(b: Oneshot<F, Request<B>>) -> Self {
        RouteFuture {
            state: RouteFutureInner::B { b },
        }
    }
}

pin_project! {
     #[project = RouteFutureInnerProject]
     #[derive(Debug)]

     enum RouteFutureInner<S,F,B>
     where S:Service<Request<B>>, F:Service<Request<B>> {


          A {
               #[pin]
               a: Oneshot<S, Request<B>>,
               fallback:Option<F>
          },
          B {
               #[pin]
               b:Oneshot<F, Request<B>>
          }

     }

}

impl<S, F, B> Future for RouteFuture<S, F, B>
where
    S: Service<Request<B>, Response = Response<BoxBody>>,
    F: Service<Request<B>, Response = Response<BoxBody>, Error = S::Error>,
    B: Send + Sync + 'static,
{
    type Output = Result<Response<BoxBody>, S::Error>;

    #[allow(warning)]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();
            let new_state = match this.state.as_mut().project() {
                RouteFutureInnerProject::A { a, fallback } => {
                    let mut response = ready!(a.poll(cx))?;
                    let req = if let Some(ext) =
                        response.extensions_mut().remove::<FromEmptyRouter<B>>()
                    {
                        ext.request
                    } else {
                        return Poll::Ready(Ok(response));
                    };

                    RouteFutureInner::B {
                        b: fallback
                            .take()
                            .expect("future polled after completion")
                            .oneshot(req),
                    }
                }
                RouteFutureInnerProject::B { b } => return b.poll(cx),
            };
            this.state.set(new_state)
        }
    }
}
