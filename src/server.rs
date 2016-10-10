use std::io;
use std::fmt::Debug;

use futures::{Future, Poll, Async};
use tokio_core::io::{Io};
use netbuf::Buf;
use httparse;

use request::Request;
use response::Response;
use error::Error;
// use super::parser::parse;


pub trait HttpHandler {
    // type Request;
    type Response: Debug;
    type Error;
    type Future: Future<Item=Response, Error=Self::Error>;

    fn call(&mut self, req: &mut Request) -> Self::Future;
}
pub trait NewHandler
{
    type Handler;

    fn new_handler(&self) -> Self::Handler;
}


/// Http Server handler.
///
/// Handles single HTTP connection.
/// It responsible for:
/// * reading incoming data into buf;
/// * parsing it using httparse;
/// * passing Request to upper Service;
/// * waiting for response;
/// * serializing into buffer;
/// * treating connection well;
pub struct HttpServer<S, H> {
    stream: S,
    // inbuf: Buf,
    // outbuf: Buf,
    // TODO: add service
    pipeline: PipelineDispatcher<H>,
}


impl<S, H> HttpServer<S, H>
    where S: Io,
          H: HttpHandler,
{
    pub fn new(stream: S, handler: H) -> HttpServer<S, H> {
        HttpServer {
            stream: stream,
            // handler: handler,
            // inbuf: Buf::new(),
            // outbuf: Buf::new(),
            pipeline: PipelineDispatcher::new(handler),
        }
    }

    fn handle(&self) {}

    fn flush(&mut self) -> io::Result<bool> {
        match self.stream.flush() {
            Ok(_) => Ok(false),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(true),
            Err(e) => return Err(e.into()),
        }
    }
}


impl<S, H> Future for HttpServer<S, H>
    where S: Io,
          H: HttpHandler,
{
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        // TODO:
        //  if there read data -> parse it
        //  if no read data -> read it into buffer [and parse it?]
        //  If there is response -> serialize it to out buffer;
        //  if there is data to be written -> write it;

        loop {
            let mut not_ready = false;
            not_ready |= try!(self.flush());

            not_ready |= try!(self.pipeline.read_from(&mut self.stream));
            not_ready |= try!(self.pipeline.dispatch(&mut self.stream));
            not_ready |= try!(self.pipeline.write_to(&mut self.stream));
            not_ready |= try!(self.flush());
            if not_ready {
                return Ok(Async::NotReady);
            }
        }

        // Loop until we block; otherwise we'd end task and close connection;
        // loop {
        //     let mut not_ready = false;

        //     // Try flush pending writes;
        //     //  ignore would block; as we always notready
        //     //  until 0 bytes read or response closes connection
        //     not_ready |= try!(self.flush());

        //     // Try read raw data into buffer;
        //     let read = match self.inbuf.read_from(&mut self.stream) {
        //         Ok(bytes) => Some(bytes),
        //         Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        //             not_ready = true;
        //             None
        //         },
        //         Err(e) => return Err(e.into()),
        //     };
        //     match read {
        //         // NOTE: should we try flushing pending data?
        //         Some(0) => return Ok(Async::Ready(())),
        //         Some(_) | None => {},
        //     }
        //     println!("read {:?} bytes", read);

        //     // Now we have to (a) parse it (b) process it and (c) serialize it;
        //     // fake it by echoing:
        //     self.handle();
        //     // if self.inbuf.len() > 0 {
        //     //     let mut headers = [httparse::EMPTY_HEADER; 16];
        //     //     let mut req = Request::new(&mut headers);
        //     //     match req.parse(&self.inbuf[..]) {
        //     //         Ok(_) => {},
        //     //         Err(_) => {},
        //     //     }
        //     //     // TODO: try parsing bytes into HttpRequest;
        //     //     //      do until complete request is parsed (or error);
        //     //     //      then pass request to service -> wait response;
        //     //     // here we need currently available request;
        //     //     //  that we can start parsing in;
        //     //     //  as soon as it parsed:
        //     //     //      we must pass it to service;
        //     //     //      create new empty request (or not);
        //     //     //      wait for new requests;
        //     //     //  Problem: Request body:
        //     //     //      we can't parse into new request case it must be
        //     //     //      body of previous request.
        //     //     //  Solution:
        //     //     //      advance state of parser to body reader;
        //     //     //      read or skip body;
        //     //     //      advance state of parser to initial;

        //     //     // self.parser.parse(&self.inbuf[..]).unwrap();

        //     //     // match self.inbuf.write_to(&mut self.outbuf) {
        //     //     //     Ok(b) => {println!("Copied {} bytes", b);},
        //     //     //     Err(e) => return Err(e.into()),
        //     //     // }
        //     // }

        //     // Try write out buffer;
        //     if self.outbuf.len() > 0 {
        //         let written = match self.outbuf.write_to(&mut self.stream) {
        //             Ok(bytes) => bytes,
        //             Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
        //                 not_ready = true;
        //                 0
        //             },
        //             Err(e) => return Err(e.into()),
        //         };
        //         println!("written {} bytes", written);
        //     }
        //     // Try flush pending writes;

        //     if not_ready {
        //         return Ok(Async::NotReady);
        //     }
        // }
    }
}


struct PipelineDispatcher<H> {
    req: Request,
    handler: H,
}

impl<H> PipelineDispatcher<H>
    where H: HttpHandler,
{
    pub fn new(handler: H) -> PipelineDispatcher<H> {

        PipelineDispatcher {
            req: Request::new(),
            handler: handler,
            // result: None,
        }
    }

    pub fn read_from<R: io::Read>(&mut self, stream: &mut R) -> Result<bool, Error> {
        match try!(self.req.read_from(stream)) {
            Async::Ready(_) => Ok(false),
            Async::NotReady => Ok(true),
        }
    }

    pub fn dispatch<W: io::Write>(&mut self, stream: &mut W) -> Result<bool, Error> {
        self.req.parse();
        if !self.req.is_ready() {
            return Ok(false);
        }
        let mut fut = self.handler.call(&mut self.req);
        match fut.poll() {
            Ok(Async::Ready(mut resp)) => {
                println!("Poll ready: {:?}", resp);
                match resp.write_to(stream) {
                    Ok(Async::Ready(_)) => {
                        // TODO:
                        //      drop result;
                        //      create new request;
                        //      consume old request buffer;
                        //      drop old request;
                        Ok(false)
                    },
                    Ok(Async::NotReady) => Ok(true),
                    Err(e) => Err(Error::ReadError(e)),
                }
            },
            Ok(Async::NotReady) => {
                println!("Poll not ready");
                Ok(true)
            },
            Err(e) => Err(Error::ReadError(
                io::Error::new(io::ErrorKind::Other, "Other"))),
        }
        // Ok(false)
    }

    pub fn write_to<W: io::Write>(&mut self, stream: &mut W) -> Result<bool, Error> {
        // get response which must be flushed;
        //  call response.write_to(stream);
        //  if response is done -> dismiss it and try next one;
        //  if response is NotReady -> return false
        Ok(false)
    }
}

struct Dispatch
{
    request: Request,
    response: Option<Response>,
    result: Option<usize>,  // XXX
}

impl Dispatch
{
    pub fn new() -> Dispatch {
        Dispatch {
            request: Request::new(),
            response: None,
            result: None,
        }
    }
    pub fn read_from<R: io::Read>(&mut self, stream: &mut R) -> Poll<(), Error> {
        self.request.parse_from(stream)
    }
}

impl Future for Dispatch {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        // 1. read request;
        //      if request somehow ready (ie: request headers are read)
        //      continue to processing
        // 2. process request;
        //      call service, receive result future; poll to see if its ready;
        // 3. write response;
        //      write response to output stream;
        // 4. done;
        //      mark self done;
        //      check whole request is consumed (consume if not);
        Ok(Async::NotReady)
    }
}
