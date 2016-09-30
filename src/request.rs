use std::io;
use std::fmt;
use std::convert::From;

use bytes::buf::BlockBuf;
use httparse;
use tokio_proto::Parse;
use tokio_proto::pipeline::Frame;

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

/// Enum reprsenting HTTP version.
#[derive(Debug, Clone)]
pub enum Version {
    Http10,
    Http11,
}

/// Request struct
///
/// some known headers may be moved to upper structure (ie, Host)
#[derive(Debug,PartialEq)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub version: u8,
    // pub host: String,

    // TODO: implement
    // headers: Vec<(str, str)>,

    // body: &'a str,
}

impl Request {
    pub fn new() -> Request {
        Request {
            method: Method::Get,
            version: 0,
            path: "".to_string(),
        }
    }

    pub fn from(&mut self, req: httparse::Request) -> io::Result<()> {
        self.method = Method::from(req.method.unwrap());
        self.version = req.version.unwrap();
        self.path = "".to_string();
        Ok(())
    }
}


pub struct Parser;


impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Version::Http10 => f.write_str("HTTP/1.0"),
            Version::Http11 => f.write_str("HTTP/1.1"),
        }
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "minihttp::Request {} {}", self.path, self.version)
    }
}

impl Parse for Parser {
    type Out = Frame<Request, (), io::Error>;

    fn parse(&mut self, buf: &mut BlockBuf) -> Option<Self::Out> {
        // Only compact if needed
        if !buf.is_compact() {
            buf.compact();
        }

        let mut n = 0;

        let res = {
            // TODO: we should grow this headers array if parsing fails and asks for
            //       more headers
            let mut headers = [httparse::EMPTY_HEADER; 16];
            let mut r = httparse::Request::new(&mut headers);
            let status = match r.parse(buf.bytes().expect("buffer not compact")) {
                Ok(status) => status,
                Err(e) => {
                    println!("Got error: {}", e);
                    return Some(Frame::Error(
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("failed to parse http request: {}", e)
                        )));
                }
            };

            match status {
                httparse::Status::Complete(amt) => {
                    n = amt;
                    let method = match r.method.unwrap().to_uppercase().as_str() {
                        "GET" => Method::Get,
                        "HEAD" => Method::Head,
                        "POST" => Method::Post,
                        "PUT" => Method::Put,
                        "DELETE" => Method::Delete,
                        m => Method::Other(m.to_string()),
                    };

                    //let mut h = "http://example.com".to_string();
                    //h.push_str(r.path.unwrap());
                    //let res = Url::parse(h.as_str()).unwrap();
                    // println!("Parsed: {:?}", res);
                    // println!("Parsed path: {:?}", res.path());
                    // println!("Parsed query: {:?}", res.query().unwrap());

                    Some(Frame::Message(Request {
                        method: method,
                        path: r.path.unwrap().to_string(),
                        version: r.version.unwrap(),
                        //headers: r.headers
                        //    .iter()
                        //    .map(|h| (toslice(h.name.as_bytes()), toslice(h.value)))
                        //    .collect(),
                        //data: None,
                    }))
                }
                httparse::Status::Partial => {
                    None
                }
            }
        };

        match res {
            Some(Frame::Message(_)) => {
                buf.shift(n);
            }
            _ => {}
        };
        res
    }

    fn done(&mut self, _: &mut BlockBuf) -> Option<Self::Out> {
        Some(Frame::Done)
        // TODO: must check if request body is fully read;
    }
}
