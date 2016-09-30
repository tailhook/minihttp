use std::io;
use std::fmt;
use std::fmt::Write;

use bytes::buf::{BlockBuf, Fmt};
use bytes::MutBuf;
use tokio_proto::Serialize;
use tokio_proto::pipeline::Frame;


#[derive(Debug)]
pub struct Response {
    code: u16,
    reason: String,

    pub headers: Option<String>,
    pub body: Option<String>,
}

pub struct Serializer;

impl Response {

    pub fn new() -> Response {
        Response {
            code: 200,
            reason: "OK".to_string(),
            headers: None,
            body: None,
        }
    }
    pub fn set_status(&mut self, code: u16) {
        self.code = code;
        // TODO: change to proper reason if code is known
    }
    pub fn set_reason(&mut self, reason: String) {
        self.reason = reason;
    }

}

impl fmt::Display for Response {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "minihttp::Response {} {}", self.code, self.reason)
    }
}

impl Serialize for Serializer {
    type In = Frame<Response, (), io::Error>;

    fn serialize(&mut self, msg: Self::In, buf: &mut BlockBuf) {
        // println!("serialize: {:?}", msg);
        // TODO: serialize Response (Status + Headers);
        //      serialize Body (sized body / streaming body);
        match msg {
            Frame::Message(_) => {
                write!(Fmt(buf), "HTTP/1.1 200 OK\r\n").unwrap();
                buf.write_slice(b"Content-Length: 12\r\n");
                buf.write_slice(b"\r\n");
                buf.write_slice(b"Hello, world");
                // buf.write_slice(b"\r\n");
            },
            _ => {},
        };
    }
}
