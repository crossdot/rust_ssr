extern crate iron;
extern crate mount;
extern crate router;
extern crate staticfile;
extern crate time;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate html5ever;

use iron::status;
use iron::{Iron, Request, Response, IronResult};
use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use iron::mime::Mime;

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
        println!("Request took: {time} ms, url: {url}", time = (delta as f64) / 1000000.0, url = req.url);
        Ok(res)
    }
}



fn say_hello(req: &mut Request) -> IronResult<Response> {
    println!("Running send_hello handler, URL path: {}", req.url.path().join("/"));
    Ok(Response::with((status::Ok, "This request was routed!")))
}

#[derive(Deserialize)]
struct JsonRequest {
}

#[derive(Serialize, Deserialize)]
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
            let out = serde_json::to_string(&response).unwrap();
            Ok(Response::with((content_type, status::Ok, out)))
        }, "json")
        .get("/error", |_: &mut Request| {
            let content_type = "application/json".parse::<Mime>().unwrap();
            let response = JsonResponse::error("some error occurred".to_string());
            let out = serde_json::to_string(&response).unwrap();
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


#[test]
fn test01() {
    let v = vec![10, 0, 2, 2, 41, 15, 61, 2, 2, 0];
    
    let mut avg_sum : usize = 0;

    use std::collections::HashMap;
    let mut mode = HashMap::<usize, usize>::new();

    for &item in &v {
        avg_sum += item;
        let mut c = mode.entry(item).or_insert(0);
        *c += 1;
    }

    let mode = mode;

    // let mut mode_max = (0, 0);
    // for (&key, &value) in &mode {
    //     if value > mode_max.1 {
    //         mode_max = (key, value);
    //     }
    // }
    let mut mode_max: (&usize, &usize) = (&0, &0);
    for entry in &mode {
        if entry.1 > mode_max.1 {
            mode_max = entry;
        }
    }

    println!("{:?} avg: {} median: {} mode: {}", v, avg_sum / v.len(), v[v.len() / 2 as usize], mode_max.0);
    assert!(true)
}

fn execute(cmd: &str) -> String {
    use std::process::Command;

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
                .args(&["/C", cmd])
                .output()
                .expect("failed to execute process")
    } else {
        Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .expect("failed to execute process")
    };

    let hello = output.stdout;
    String::from_utf8(hello).unwrap()
}

#[test]
fn test_execute() {
    println!("{}", execute("pwd"));
}

fn parse() -> String {
    use std::default::Default;

    use html5ever::{parse_document, serialize};
    use html5ever::driver::ParseOpts;
    use html5ever::rcdom::RcDom;
    use html5ever::tendril::TendrilSink;
    use html5ever::tree_builder::TreeBuilderOpts;

    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            // drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };
    // let stdin = io::stdin();
    let dom = parse_document(RcDom::default(), opts)
        .from_utf8()
        // .from_file("static/index.html")
        // .read_from(&mut stdin.lock())
        .read_from(&mut "some empty space".as_bytes())
        .unwrap();

    // The validator.nu HTML2HTML always prints a doctype at the very beginning.
    // io::stdout().write_all(b"<!DOCTYPE html>\n")
        // .ok().expect("writing DOCTYPE failed");
    // serialize(&mut io::stdout(), &dom.document, Default::default()).expect("serialization failed");

    let mut bytes = vec![];
    serialize(&mut bytes, &dom.document, Default::default()).unwrap();
    String::from_utf8(bytes).unwrap()
}

#[test]
fn test_parse_document() {
    println!("{}", parse());
}

#[test]
fn test_modify() {
    use html5ever::{ParseOpts, parse_document};
    use html5ever::tree_builder::TreeBuilderOpts;
    use html5ever::rcdom::RcDom;
    use html5ever::rcdom::NodeData::Element;
    use html5ever::serialize::{SerializeOpts, serialize};
    use html5ever::tendril::TendrilSink;

    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let data = "<!DOCTYPE html><html><body><a href=\"foo\"></a></body></html>";
    let dom = parse_document(RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut data.as_bytes())
        .unwrap();

    let html = &dom.document.children.borrow()[0];
    let body = &html.children.borrow()[1];

    {
        let a = &body.children.borrow()[0];
        if let Element { ref attrs, .. } = a.data {
            let mut attrs = attrs.borrow_mut();
            attrs[0].value.push_tendril(&From::from("#anchor"));
        }
    }

    let mut bytes = vec![];
    serialize(&mut bytes, &dom.document, SerializeOpts::default()).unwrap();
    let result = String::from_utf8(bytes).unwrap();
    println!("{}", result);
}