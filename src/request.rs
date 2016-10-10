use std::io;
// use std::fmt;
use std::convert::From;
use std::str;
use std::ascii::AsciiExt;

use httparse;
use netbuf::Buf;
use futures::{Async, Poll};

use super::error::Error;
use super::response::Response;


type Slice = (usize, usize);

/// Enum representing HTTP request methods.
///
/// ```rust,ignore
/// match req.method {
///     Method::Get => {},   // handle GET
///     Method::Post => {},  // handle POST requests
///     Method::Other(m) => { println!("Custom method {}", m); },
///     _ => {}
///     }
/// ```
#[derive(Debug,PartialEq)]
pub enum Method {
    Options,
    Get,
    Head,
    Post,
    Put,
    Delete,
    Trace,
    Connect,
    Other(String),
}

impl<'a> From<&'a str> for Method
{
    
    fn from(s: &'a str) -> Method {
        match s {
            "OPTIONS"   => Method::Options,
            "GET"       => Method::Get,
            "HEAD"      => Method::Head,
            "POST"      => Method::Post,
            "PUT"       => Method::Put,
            "DELETE"    => Method::Delete,
            "TRACE"     => Method::Trace,
            "CONNECT"   => Method::Connect,
            s => Method::Other(s.to_string()),
        }
    }
}

pub enum Header {
    Host,
    ContentType,
    Raw,
}

// /// Enum reprsenting HTTP version.
// #[derive(Debug, Clone)]
// pub enum Version {
//     Http10,
//     Http11,
// }
// impl fmt::Display for Version {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             Version::Http10 => f.write_str("HTTP/1.0"),
//             Version::Http11 => f.write_str("HTTP/1.1"),
//         }
//     }
// }
const MIN_HEADERS_ALLOCATE: usize = 16;

/// Request struct
///
/// some known headers may be moved to upper structure (ie, Host)
#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub version: u8,

    buf: Buf,
    request_size: usize,
    headers: Vec<(Slice, Slice)>,

    // some known headers
    host: Option<Slice>,
    content_type: Option<Slice>,
    // add more known headers;

    response: Option<Response>,
}

impl Request {
    pub fn new() -> Request {
        Request {
            method: Method::from(""),
            version: 0,
            path: "".to_string(),
            buf: Buf::new(),
            request_size: 0,
            headers:  Vec::with_capacity(MIN_HEADERS_ALLOCATE),

            host: None,
            content_type: None,
            response: None,
        }
    }

    pub fn make_response(&mut self) -> &mut Response {
        if self.response.is_none() {
            self.response = Some(Response::new(self.version));
        }
        self.response.as_mut().unwrap()
    }

    pub fn read_from<R: io::Read>(&mut self, stream: &mut R) -> Poll<(), Error> {
        match self.buf.read_from(stream) {
            Ok(_) => Ok(Async::Ready(())),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(Async::NotReady),
            Err(e) => Err(Error::ReadError(e)),
        }
    }

    pub fn parse(&mut self) {
        if self.request_size == 0 {
            self.parse_request()
        } else {
            self.parse_body()
        };
    }

    pub fn is_ready(&self) -> bool {
        self.request_size != 0
    }

    pub fn parse_from<R: io::Read>(&mut self, stream: &mut R) -> Poll<(), Error> {
        let res = self.read_from(stream);
        self.parse();
        res
    }

    fn parse_request(&mut self) -> Poll<(), Error> {
        let mut headers = [httparse::EMPTY_HEADER; MIN_HEADERS_ALLOCATE];
        let mut parser = httparse::Request::new(&mut headers);

        self.request_size = match parser.parse(&self.buf[..]) {
            Ok(httparse::Status::Complete(amt)) => amt,
            Ok(httparse::Status::Partial) => {
                return Ok(Async::NotReady)
            },
            Err(e) => {
                return Err(Error::ParseError(e))
            }
        };
        self.method = Method::from(parser.method.unwrap());
        self.version = parser.version.unwrap();
        self.path = parser.path.unwrap().to_string();

        // process headers;
        let start = self.buf[..].as_ptr() as usize;
        let buf_len = self.buf.len() as usize;
        let toslice = |a: &[u8]| {
                let start = a.as_ptr() as usize - start;
                assert!(start < buf_len);
                (start, start + a.len())
            };
        for h in parser.headers.iter() {
            let name = toslice(h.name.as_bytes());
            let value = toslice(h.value);
            self.headers.push((name, value));
            if h.name.eq_ignore_ascii_case("host") {
                self.host = Some(value);
            }
        }
        Ok(Async::Ready(()))
    }

    fn parse_body(&mut self) -> Poll<(), Error> {
        Ok(Async::Ready(()))
    }

    // Public interface

    /// Value of Host header
    pub fn host<'req>(&'req self) -> Option<&'req [u8]> {
        match self.host {
            Some(s) => Some(&self.buf[s.0..s.1]),
            None => None,
        }
    }

    /// Value of Content-Type header
    pub fn content_type<'req>(&'req self) -> Option<&'req [u8]> {
        match self.content_type {
            Some(s) => Some(&self.buf[s.0..s.1]),
            None => None,
        }
    }

    // interface to body

    /// Read request body into buffer.
    pub fn read_body(&mut self, buf: &mut [u8]) -> Poll<usize, Error> {
        // this must/should be hooked to underlying tcp stream
        Ok(Async::NotReady)
    }
}
