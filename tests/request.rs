extern crate futures;
extern crate httparse;
extern crate minihttp;

// use std::convert::From;
use minihttp::request::Method;
use minihttp::request::Request;


#[test]
fn method_from_str() {
    assert_eq!(Method::from("GET"), Method::Get);
    assert_eq!(Method::from("get"), Method::Other("get"));
    assert_eq!(Method::from("Get"), Method::Other("Get"));

    assert_eq!(Method::from("OPTIONS"), Method::Options);
    assert_eq!(Method::from("GET"), Method::Get);
    assert_eq!(Method::from("HEAD"), Method::Head);
    assert_eq!(Method::from("POST"), Method::Post);
    assert_eq!(Method::from("PUT"), Method::Put);
    assert_eq!(Method::from("DELETE"), Method::Delete);
    assert_eq!(Method::from("TRACE"), Method::Trace);
    assert_eq!(Method::from("CONNECT"), Method::Connect);
}

#[test]
fn debug_fmt() {
    assert_eq!(format!("{:?}", Method::Get), "Get");
    assert_eq!(format!("{:?}", Method::Other("patch")),
               "Other(\"patch\")");
}


#[test]
fn test_request() {
    let mut headers = [httparse::EMPTY_HEADER; 10];
    let mut req = Request::new(&mut headers);

    let buf = b"GET /path HTTP/1.1\r\nHost: example.com\r\n\r\n";

    assert_eq!(req.parse(buf).unwrap(), futures::Async::Ready(()));
    assert_eq!(req.path, "/path");
    assert_eq!(req.method, Method::Get);
    assert_eq!(req.version, 1);

    assert_eq!(req.host(), Some("example.com"));
}
