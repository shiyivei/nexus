use pin_project_lite::pin_project;

pin_project! {
     #[project = EitherProj]
     pub(crate) enum Either<A,B> {
          A {#[pin] inner:A},
          B {#[pin] inner:B},
     }
}
