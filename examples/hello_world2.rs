extern crate tokio_core;
extern crate futures;
extern crate minihttp;
#[macro_use] extern crate log;
extern crate env_logger;

use std::env;

use tokio_core::reactor::Core;


fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init().expect("init logging");

    let mut lp = Core::new().unwrap();

    let addr = "0.0.0.0:8080".parse().unwrap();

    minihttp::core_serve(&lp.handle(), addr);

    lp.run(futures::empty::<(), ()>()).unwrap();
}
