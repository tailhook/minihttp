use std::io;
use std::fmt;
use std::io::Write;

use futures;
use futures::{Poll, Async};
use netbuf::Buf;


#[derive(Debug, PartialEq)]
enum ResponseState {
    CollectHeaders,
    WriteBody,
}

#[derive(Debug)]
pub struct Response {
    code: u16,
    reason: String,
    version: u8,
    buf: Buf,
    state: ResponseState,

    headers: Vec<String>,
    pub body: Option<String>,
}

impl Response {

    pub fn new(version: u8) -> Response {
        Response {
            code: 200,
            reason: "OK".to_string(),
            version: version,
            buf: Buf::new(),
            state: ResponseState::CollectHeaders,

            headers: Vec::with_capacity(16*2),
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

    pub fn write_to<W: io::Write>(&mut self, stream: &mut W) -> Poll<(), io::Error> {
        // TODO: maybe pass buf in new so it can't be touched
        //      and no write_to interface exposed;
        match self.buf.write_to(stream) {
            Ok(b) => {
                Ok(Async::Ready(()))
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(Async::NotReady),
            Err(e) => Err(e),
        }
    }

    pub fn with_header(&mut self, header: String, value: String) {
        assert_eq!(self.state, ResponseState::CollectHeaders);
        self.headers.push(header);
        self.headers.push(value);
    }

    pub fn with_body(&mut self, buf: &[u8]) {
        if self.state == ResponseState::CollectHeaders {
            self.flush_headers();
            self.state = ResponseState::WriteBody;
        }
        self.buf.write(buf);
        // write body
    }

    pub fn flush_headers(&mut self) {
        // self.buf.write();
        assert_eq!(self.state, ResponseState::CollectHeaders);
        write!(self.buf, "HTTP/1.{} {} {}\r\n",
               self.version, self.code, self.reason);
        for (h, v) in self.headers.chunks(2).map(|pair| (&pair[0], &pair[1])) {
            write!(self.buf, "{}: {}\r\n", h, v);
        }
        self.buf.write(b"Connection: close\r\n");
        self.buf.write(b"\r\n");
    }

    pub fn done(&mut self) {
        loop {
            match self.state {
                ResponseState::CollectHeaders => {
                    self.flush_headers();
                    self.state = ResponseState::WriteBody;
                },
                ResponseState::WriteBody => {
                    return
                }
            }
        }
    }
}


fn handle(req: super::request::Request) -> futures::Finished<Box<Response>, io::Error> {
    //let mut resp = req.make_response();
    let mut resp = Response::new(req.version);
    resp.set_status(200);
    // resp.write_body("Hello World");     // -> maybe start flushing data
    let resp = Box::new(resp);
    futures::finished(resp)
}
