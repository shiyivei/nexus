use std::ops::Deref;

use bytes::Bytes;
use pin_project_lite::pin_project;

pin_project! {
     #[project = EitherProj]
     pub(crate) enum Either<A,B> {
          A {#[pin] inner:A},
          B {#[pin] inner:B},
     }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ByteStr(Bytes);

impl Deref for ByteStr {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}
impl ByteStr {
    pub(crate) fn new<S>(s: S) -> Self
    where
        S: AsRef<str>,
    {
        Self(Bytes::copy_from_slice(s.as_ref().as_bytes()))
    }
    pub(crate) fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap()
    }
}
