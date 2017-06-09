extern crate iron;
extern crate mount;
extern crate router;
extern crate staticfile;
extern crate time;
extern crate rustc_serialize;

use iron::status;
use iron::{Iron, Request, Response, IronResult};
use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use iron::mime::Mime;

use time::precise_time_ns;
use mount::Mount;
use router::Router;
use staticfile::Static;
use rustc_serialize::json;

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

#[derive(RustcDecodable)]
struct JsonRequest {
}

#[derive(RustcEncodable, RustcDecodable)]
struct JsonResponse {
    response: String,
    success: bool,
    error_message: String
}

impl JsonResponse {
    fn success(response: String) -> Self {
        JsonResponse { response: response, success: true, error_message: "".to_string() }
    }

    fn error(msg: String) -> Self {
        JsonResponse { response: "".to_string(), success: false, error_message: msg }
    }
}

fn main() {
    let mut router = Router::new();
    router
        .get("/json", |_: &mut Request| {
            let content_type = "application/json".parse::<Mime>().unwrap();
            let response = JsonResponse::success("some value".to_string());
            let out = json::encode(&response).unwrap();
            Ok(Response::with((content_type, status::Ok, out)))
        }, "json")
        .get("/error", |_: &mut Request| {
            let content_type = "application/json".parse::<Mime>().unwrap();
            let response = JsonResponse::error("some error occurred".to_string());
            let out = json::encode(&response).unwrap();
            Ok(Response::with((content_type, status::Ok, out)))
        }, "error")
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