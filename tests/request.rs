extern crate futures;
extern crate httparse;
extern crate minihttp;

use std::str;
use std::io;
use std::io::{Read, Write};
use minihttp::request::Method;
use minihttp::request::Request;


struct Buf<'b> {
    data: &'b [u8],
}
impl<'b> Buf<'b> {
    pub fn new(buf: &'b [u8]) -> Buf {
        Buf {data: buf}
    }
}
impl<'b> Read for Buf<'b> {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        buf.write(self.data)
    }
}

#[test]
fn method_from_str() {
    assert_eq!(Method::from("GET"), Method::Get);
    assert_eq!(Method::from("get"), Method::Other("get".to_string()));
    assert_eq!(Method::from("Get"), Method::Other("Get".to_string()));

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
    assert_eq!(format!("{:?}", Method::Other("patch".to_string())),
               "Other(\"patch\")");
}


#[test]
fn request() {
    let mut req = Request::new();
    let buf = b"GET /path HTTP/1.1\r\nHost: example.com\r\n\r\n";
    let mut buf = Buf::new(buf);

    assert_eq!(req.parse_from(&mut buf).unwrap(), futures::Async::Ready(()));
    assert_eq!(req.method, Method::Get);
    assert_eq!(req.path, "/path".to_string());
    assert_eq!(req.version, 1);

    assert_eq!(req.host().unwrap(), b"example.com");
}

#[test]
fn partial_request() {
    let mut req = Request::new();
    let mut buf = Buf::new(b"HEAD /path?with=query HTTP/1.1\r\n");

    assert_eq!(req.parse_from(&mut buf), Ok(futures::Async::NotReady));

    let mut buf = Buf::new(b"Host: www.example.com\r\n\r\n");

    assert_eq!(req.parse_from(&mut buf), Ok(futures::Async::Ready(())));

    assert_eq!(req.method, Method::Head);
    assert_eq!(req.path, "/path?with=query".to_string());
    assert_eq!(req.version, 1);

    assert_eq!(str::from_utf8(req.host().unwrap()).unwrap(), "www.example.com");
}
