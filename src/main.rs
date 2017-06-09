//! This example shows how to serve static files at specific
//! mount points, and then delegate the rest of the paths to a router.
//!
//! It serves the docs from target/doc at the /docs/ mount point
//! and delegates the rest to a router, which itself defines a
//! handler for route /hello
//!
//! Make sure to generate the docs first with `cargo doc`,
//! then build the tests with `cargo run --example router`.
//!
//! Visit http://127.0.0.1:3000/hello to view the routed path.
//!
//! Visit http://127.0.0.1:3000/docs/mount/ to view the mounted docs.

extern crate iron;
extern crate mount;
extern crate router;
extern crate staticfile;
extern crate time;

use iron::status;
use iron::{Iron, Request, Response, IronResult};
use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use time::precise_time_ns;

use mount::Mount;
use router::Router;
use staticfile::Static;

use std::path::Path;
use std::time::Duration;

struct ResponseTime;
impl typemap::Key for ResponseTime { type Value = u64; }
impl BeforeMiddleware for ResponseTime {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<ResponseTime>(precise_time_ns());
        Ok(())
    }
}
impl AfterMiddleware for ResponseTime {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        let delta = precise_time_ns() - *req.extensions.get::<ResponseTime>().unwrap();
        println!("Request took: {} ms", (delta as f64) / 1000000.0);
        Ok(res)
    }
}



fn say_hello(req: &mut Request) -> IronResult<Response> {
    println!("Running send_hello handler, URL path: {}", req.url.path().join("/"));
    Ok(Response::with((status::Ok, "This request was routed!")))
}

fn main() {
    let mut router = Router::new();
    router
        .get("/hello", say_hello, "hello");

    let mut mount = Mount::new();
    mount
        .mount("/", router)
        .mount("/static/", Static::new(Path::new("static")).cache(Duration::from_secs(10*60)));

    let mut chain = Chain::new(mount);
    chain.link_before(ResponseTime);
    chain.link_after(ResponseTime);

    Iron::new(chain).http("127.0.0.1:3000").unwrap();
}