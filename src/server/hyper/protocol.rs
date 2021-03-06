// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use native_tls::Pkcs12;
use super::{Server, InternalServer, ServerStopper};
use hyper::server::Http;
use std::sync::Arc;

pub enum Protocol {
    Http,
    Https(Pkcs12)
}

impl Protocol {
    pub fn run(self, server: Server) -> Result<ServerStopper, ::hyper::Error> {
        match self {
            Protocol::Http => Self::run_http(server),
            Protocol::Https(pkcs) => Self::run_https(pkcs, server),
        }
    }

    pub fn run_https(pkcs: Pkcs12, server: Server) -> Result<ServerStopper, ::hyper::Error> {
        use hyper::server::Http;
        use native_tls::TlsAcceptor;
        use tokio_proto::TcpServer;
        use tokio_tls::proto;

        let tls_cx = TlsAcceptor::builder(pkcs).unwrap()
            .build().unwrap();

        let proto = proto::Server::new(Http::new(), tls_cx);

        let addr = server.addr.clone();
        let srv = TcpServer::new(proto, addr);
        let router = server.router;
        let state = server.state;
        srv.serve(move || Ok(InternalServer { router: router.clone(), state: state.clone() }));

        Ok(ServerStopper::default())
    }

    pub fn run_http(server: Server) -> Result<ServerStopper, ::hyper::Error> {
        //fixme return server, but what type does it have???
        let addr = server.addr.clone();
        let router = server.router;
        let state = server.state;
        let stopper = server.stopper;
        state.set(stopper);
        let s = Http::new().bind(&addr, move || Ok(InternalServer { router: router.clone(), state: state.clone() }))?;

        let stopper = super::ServerStopper { stop: Arc::new(::std::sync::atomic::AtomicBool::new(false)) };
        s.run_until(stopper.clone())?;
        Ok(stopper)
    }
}