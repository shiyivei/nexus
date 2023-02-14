pub trait IntoResponse {
    type Err;
    fn into_response(&self) -> Result<(), Self::Err>;
}
