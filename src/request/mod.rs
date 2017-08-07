use std::collections::HashMap;
use state::Container;
use std::collections::BTreeMap;
use http::{Uri, HeaderMap, Method};

mod params;

pub use self::params::Params;


pub struct Request<'r> {
    state: StateHolder<'r>,
    uri: ::http::Uri,
    method: ::http::Method,
    params: Params,
    headers: HeaderMap<String>,
    query: HashMap<String, Vec<String>>,
    remote_addr: Option<::std::net::SocketAddr>,
}

enum StateHolder<'r> {
    None,
    Some(&'r Container)
}

impl<'r> ::std::fmt::Debug for StateHolder<'r> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            &StateHolder::None => write!(f, "no state"),
            _ => write!(f, "has state"),
        }
    }
}

impl<'r> Default for Request<'r> {
    fn default() -> Self {
        Request {
            state: StateHolder::None,
            uri: ::http::Uri::default(),
            method: ::http::Method::default(),
            params: Params::default(),
            query: HashMap::default(),
            headers: HeaderMap::default(),
            remote_addr: None,
        }
    }
}


impl<'r> Request<'r> {
    pub fn from_hyper(hyper_req: ::hyper::Request, state: &'r Container, params: Params) -> Self {
        let mut headers = HeaderMap::new();
        for item in hyper_req.headers().iter() {
            headers.insert(item.name(), item.value_string());
        }
        let remote_addr = hyper_req.remote_addr();

        let query = Request::parse_query(hyper_req.query());
        let uri = hyper_req.uri().as_ref().parse::<::http::Uri>().unwrap();// we trust hyper
        use ::hyper::Method::*;
        let method = match hyper_req.method() {
            &Get => ::http::method::GET,
            &Put => ::http::method::PUT,
            &Post => ::http::method::POST,
            &Head => ::http::method::HEAD,
            &Patch => ::http::method::PATCH,
            &Connect => ::http::method::CONNECT,
            &Delete => ::http::method::DELETE,
            &Options => ::http::method::OPTIONS,
            &Trace => ::http::method::TRACE,
            &Extension(_) => unimplemented!(),
        };

        Request { uri, method, params, state: StateHolder::Some(state), query, headers, remote_addr }
    }

    pub fn param(&self, name: &str) -> Option<&str> {
        self.params.get(name)
    }

    pub fn params(&self) -> &Params {
        &self.params
    }

    pub fn query_first(&self, name: &str) -> Option<&str> {
        let o = self.query_all().get(name);
        if let Some(vec) = o {
            if !vec.is_empty() {
                Some(vec[0].as_str())
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn query(&self, name: &str) -> Option<&Vec<String>> {
        self.query_all().get(name)
    }

    pub fn query_all(&self) -> &HashMap<String, Vec<String>> {
        &self.query
    }

    fn parse_query(query: Option<&str>) -> HashMap<String, Vec<String>> {
        use std::borrow::Borrow;

        if let Some(query) = query {
            let parsed = ::url::form_urlencoded::parse(query.as_bytes());
            let mut map = HashMap::new();
            for parse in parsed {
                let key = parse.0.borrow();
                let value = parse.1.borrow();

                map.entry(String::from(key)).or_insert(Vec::new()).push(String::from(value));
            }
            map
        } else {
            HashMap::with_capacity(0)
        }
    }

    pub fn get_state<T: Send + Sync + 'static>(&self) -> Option<&T> {
        match self.state {
            StateHolder::None => None,
            StateHolder::Some(state) => state.try_get(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::Request as HRequest;
    use hyper::{Method, Uri};
    use super::Params;
    use std::str::FromStr;

    #[test]
    fn test_query_param() {
        let c = Container::new();
        let hr = HRequest::new(Method::Get, Uri::from_str("/bla?hallo=welt&hallo=blubb").unwrap());
        let req = Request::from_hyper(hr, &c, Params::new());
        assert_eq!("welt", req.query_first("hallo").unwrap());
        assert_eq!(None, req.query_first("ne"));
        assert_eq!(2, req.query("hallo").unwrap().len());
    }
}