use http_body::Body;

pub type BoxBody = http_body::combinators::BoxBody<Bytes, Error>;

// convert `http_body::Body` to `BoxBoxy`
pub fn box_body<B>(body: B) -> BoxBody
where
    B: http_body::Body<Data = Bytes> + Send + Sync + 'static,
    B::Error: Into<BoxError>,
{
    body.map_err(Error::new).boxed()
}

pub(crate) fn empty() -> BoxBody {
    box_body(http_body::Empty::new())
}
