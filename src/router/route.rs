use bytes::Bytes;
use regex::Regex;

use self::future::RouteFuture;
use super::*;
use crate::util::ByteStr;

#[derive(Debug, Clone)]
pub struct Route<S, F> {
    pub(crate) pattern: PathPattern, // parse url and match route
    pub(crate) svc: S,               // service
    pub(crate) fallback: F,          // back to 404 or SPA application
}

impl<S, F, B> Service<Request<B>> for Route<S, F>
where
    S: Service<Request<B>, Response = Response<BoxBody>> + Clone,
    F: Service<Request<B>, Response = Response<BoxBody>, Error = S::Error> + Clone,
    B: Send + Sync + 'static,
{
    type Response = Response<BoxBody>;
    type Error = S::Error;
    type Future = RouteFuture<S, F, B>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    // call and return, choose the handle
    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        if let Some(captures) = self.pattern.full_match(&req) {
            insert_url_params(&mut req, captures);
            let fut = self.svc.clone().oneshot(req);

            RouteFuture::a(fut, self.fallback.clone())
        } else {
            let fut = self.fallback.clone().oneshot(req);
            RouteFuture::b(fut)
        }
    }
}

#[derive(Debug)]
pub(crate) struct UrlParams(pub(crate) Vec<(ByteStr, ByteStr)>);

fn insert_url_params<B>(req: &mut Request<B>, params: Vec<(String, String)>) {
    let params = params
        .into_iter()
        .map(|(k, v)| (ByteStr::new(k), ByteStr::new(v)));

    if let Some(current) = req.extensions_mut().get_mut::<Option<UrlParams>>() {
        let mut current = current.take().unwrap();

        current.0.extend(params);
        req.extensions_mut().insert(Some(current));
    } else {
        req.extensions_mut()
            .insert(Some(UrlParams(params.collect())));
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PathPattern(Arc<Inner>);
impl PathPattern {
    pub(crate) fn new(pattern: &str) -> Self {
        assert!(pattern.starts_with('/'), "Route path must start with a `/`");

        let mut capture_group_names = Vec::new();

        let pattern: String = pattern
            .split('/')
            .map(|part| {
                if let Some(key) = part.strip_prefix(':') {
                    capture_group_names.push(Bytes::copy_from_slice(key.as_bytes()));
                    Cow::Owned(format!("(?P<{}>[^/]+)", key))
                } else {
                    Cow::Borrowed(part)
                }
            })
            .collect::<Vec<_>>()
            .join("/");
        let full_path_regex =
            Regex::new(&format!("^{}", pattern)).expect("invalid regex generate from route");
        Self(Arc::new(Inner {
            full_path_regex,
            capture_group_names: capture_group_names.into(),
        }))
    }

    pub(crate) fn full_match<B>(&self, req: &Request<B>) -> Option<Captures> {
        self.do_match(req).and_then(|match_| {
            if match_.full_match {
                Some(match_.captures)
            } else {
                None
            }
        })
    }

    pub(crate) fn prefix_match<'a, B>(&self, req: &'a Request<B>) -> Option<(&'a str, Captures)> {
        self.do_match(req)
            .and_then(|match_| Some((match_.matched, match_.captures)))
    }

    fn do_match<'a, B>(&self, req: &'a Request<B>) -> Option<Match<'a>> {
        let path = req.uri().path();

        self.0.full_path_regex.captures(path).map(|captures| {
            let matched = captures.get(0).unwrap();

            let full_match = matched.as_str() == path;

            let captures = self
                .0
                .capture_group_names
                .iter()
                .map(|bytes| {
                    std::str::from_utf8(bytes).expect("bytes were created from str is valid utf-8")
                })
                .filter_map(|name| captures.name(name).map(|value| (name, value.as_str())))
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect::<Vec<_>>();

            Match {
                captures,
                full_match,
                matched: matched.as_str(),
            }
        })
    }
}

struct Match<'a> {
    captures: Captures,
    full_match: bool,
    matched: &'a str,
}

type Captures = Vec<(String, String)>;

#[derive(Debug)]
struct Inner {
    full_path_regex: Regex,
    // dynamic route
    // Box<[Bytes]> is smaller than Vec
    capture_group_names: Box<[Bytes]>,
}
