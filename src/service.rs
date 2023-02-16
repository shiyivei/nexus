use std::marker::PhantomData;
pub struct HandleError<S, F, B> {
    inner: S,
    f: F,
    _marker: PhantomData<fn() -> B>,
}

impl<S, F, B> HandleError<S, F, B> {
    pub(crate) fn new(inner: S, f: F) -> Self {
        Self {
            inner: inner,
            f,
            _marker: PhantomData,
        }
    }
}
