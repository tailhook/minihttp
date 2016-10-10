extern crate tokio_core;
extern crate futures;
extern crate minihttp;
#[macro_use] extern crate log;
extern crate env_logger;

use std::env;
use std::io;

use futures::Future;
use tokio_core::reactor::Core;

use minihttp::server;
use minihttp::request::Request;
use minihttp::response::Response;


struct HelloWorld;

impl server::HttpHandler for HelloWorld {
    // type Request = Request;
    type Response = Response;
    type Error = io::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&mut self, req: &mut Request) -> Self::Future {
        let mut resp = Response::new(req.version);
        resp.with_header("Content-Type".to_string(), "text/plain".to_string());
        resp.with_header("Content-Length".to_string(), "12".to_string());
        resp.with_body(b"Hello world!");
        futures::finished(resp).boxed()
    }
}
impl server::NewHandler for HelloWorld {
    type Handler = HelloWorld;

    fn new_handler(&self) -> HelloWorld {
        HelloWorld {}
    }
}


fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init().expect("init logging");

    let mut lp = Core::new().unwrap();

    let addr = "0.0.0.0:8080".parse().unwrap();

    let h = HelloWorld {};
    minihttp::core_serve(&lp.handle(), addr, h);

    lp.run(futures::empty::<(), ()>()).unwrap();
}
