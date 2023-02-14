use bitflags::bitflags;
use http::Method;

bitflags! {
     pub struct MethodFilter: u16 {
          const CONNECT = 0b000000001;
          const DELETE =  0b000000010;
          const GET =     0b000000100;
          const HEAD =    0b000001000;
          const OPTIONS = 0b000010000;
          const PATCH =   0b000100000;
          const POST =    0b001000000;
          const PUT =     0b010000000;
          const TRACE =   0b100000000;
     }
}

impl MethodFilter {
    #[allow(clippy::match_like_matches_macro)]
    pub(crate) fn matches(self, method: &Method) -> bool {
        let method = match *method {
            Method::CONNECT => Self::CONNECT,
            Method::DELETE => Self::DELETE,
            Method::GET => Self::GET,
            Method::HEAD => Self::HEAD,
            Method::OPTIONS => Self::OPTIONS,
            Method::PATCH => Self::PATCH,
            Method::POST => Self::POST,
            Method::PUT => Self::PUT,
            Method::TRACE => Self::TRACE,
            _ => return false,
        };

        self.contains(method)
    }
}
