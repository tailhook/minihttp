// use std::io;
// use std::fmt;
use std::convert::From;
use std::str;
use std::ascii::AsciiExt;

use httparse;
use futures::{Async, Poll};

use super::error::Error;

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
pub enum Method<'buf> {
    Options,
    Get,
    Head,
    Post,
    Put,
    Delete,
    Trace,
    Connect,
    Other(&'buf str),
}

impl<'a> From<&'a str> for Method<'a>
{
    
    fn from(s: &'a str) -> Method<'a> {
        match s {
            "OPTIONS"   => Method::Options,
            "GET"       => Method::Get,
            "HEAD"      => Method::Head,
            "POST"      => Method::Post,
            "PUT"       => Method::Put,
            "DELETE"    => Method::Delete,
            "TRACE"     => Method::Trace,
            "CONNECT"   => Method::Connect,
            s => Method::Other(s),
        }
    }
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

/// Request struct
///
/// some known headers may be moved to upper structure (ie, Host)
#[derive(Debug,PartialEq)]
pub struct Request<'headers, 'buf: 'headers> {
    pub method: Method<'buf>,
    pub path: &'buf str,
    pub version: u8,

    request_size: usize,
    headers: &'headers mut [httparse::Header<'buf>],

    host: Option<&'buf str>,
}

impl<'h, 'b> Request<'h, 'b> {
    pub fn new(headers: &'h mut [httparse::Header<'b>]) -> Request<'h, 'b> {
        Request {
            method: Method::from(""),
            version: 0,
            path: "",
            request_size: 0,
            headers: headers,
            host: None,
        }
    }

    pub fn parse(&mut self, buf: &'b [u8]) -> Poll<(), Error> {
        {
            let mut parser = httparse::Request::new(self.headers);

            let amt = match parser.parse(buf) {
                Ok(httparse::Status::Complete(amt)) => amt,
                Ok(httparse::Status::Partial) => {
                    return Ok(Async::NotReady)
                },
                Err(e) => {
                    return Err(Error::ParseError(e))
                }
            };
            self.request_size = amt;
            self.method = Method::from(parser.method.unwrap());
            self.version = parser.version.unwrap();
            self.path = parser.path.unwrap();
        }
        // process headers;
        for header in self.headers.iter(){
            println!("{:?}", header);
            if header.name.eq_ignore_ascii_case("host") {
                self.host = match str::from_utf8(header.value) {
                    Ok(val) => Some(val),
                    Err(_) => None,
                }
            }
        };
        Ok(Async::Ready(()))
    }

    pub fn request_size(&self) -> usize {
        self.request_size
    }

    /// Returns value of Host header
    pub fn host(&self) -> Option<&'b str> {
        self.host
    }
}
