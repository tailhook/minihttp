use std::io;

use futures::{Future, Poll, Async};
use tokio_core::io::{Io};
use netbuf::Buf;
use httparse;

use request::Request;
// use super::parser::parse;

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
pub struct HttpServer<S> {
    stream: S,
    inbuf: Buf,
    outbuf: Buf,
    // TODO: add service
}


impl<S> HttpServer<S>
    where S: Io,
{
    pub fn new(stream: S) -> HttpServer<S> {
        HttpServer {
            stream: stream,
            inbuf: Buf::new(),
            outbuf: Buf::new(),
        }
    }

    fn handle(&self) {
        if self.inbuf.len() > 0 {
            let mut headers = [httparse::EMPTY_HEADER; 16];
            let mut req = Request::new(&mut headers);
            match req.parse(&self.inbuf[..]) {
                Ok(_) => {},
                Err(_) => {},
            }
        }
    }
}


impl<S> Future for HttpServer<S>
    where S: Io,
{
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        // TODO:
        //  if there read data -> parse it
        //  if no read data -> read it into buffer [and parse it?]
        //  If there is response -> serialize it to out buffer;
        //  if there is data to be written -> write it;

        // Loop until we block; otherwise we'd end task and close connection;
        loop {
            let mut not_ready = false;

            // Try flush pending writes;
            //  ignore would block; as we always notready
            //  until 0 bytes read or response closes connection
            match self.stream.flush() {
                Ok(_) => {},
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    not_ready = true;
                },
                Err(e) => return Err(e.into()),
            }

            // Try read raw data into buffer;
            let read = match self.inbuf.read_from(&mut self.stream) {
                Ok(bytes) => Some(bytes),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    not_ready = true;
                    None
                },
                Err(e) => return Err(e.into()),
            };
            match read {
                // NOTE: should we try flushing pending data?
                Some(0) => return Ok(Async::Ready(())),
                Some(_) | None => {},
            }
            println!("read {:?} bytes", read);

            // Now we have to (a) parse it (b) process it and (c) serialize it;
            // fake it by echoing:
            self.handle();
            // if self.inbuf.len() > 0 {
            //     let mut headers = [httparse::EMPTY_HEADER; 16];
            //     let mut req = Request::new(&mut headers);
            //     match req.parse(&self.inbuf[..]) {
            //         Ok(_) => {},
            //         Err(_) => {},
            //     }
            //     // TODO: try parsing bytes into HttpRequest;
            //     //      do until complete request is parsed (or error);
            //     //      then pass request to service -> wait response;
            //     // here we need currently available request;
            //     //  that we can start parsing in;
            //     //  as soon as it parsed:
            //     //      we must pass it to service;
            //     //      create new empty request (or not);
            //     //      wait for new requests;
            //     //  Problem: Request body:
            //     //      we can't parse into new request case it must be
            //     //      body of previous request.
            //     //  Solution:
            //     //      advance state of parser to body reader;
            //     //      read or skip body;
            //     //      advance state of parser to initial;

            //     // self.parser.parse(&self.inbuf[..]).unwrap();

            //     // match self.inbuf.write_to(&mut self.outbuf) {
            //     //     Ok(b) => {println!("Copied {} bytes", b);},
            //     //     Err(e) => return Err(e.into()),
            //     // }
            // }

            // Try write out buffer;
            if self.outbuf.len() > 0 {
                let written = match self.outbuf.write_to(&mut self.stream) {
                    Ok(bytes) => bytes,
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        not_ready = true;
                        0
                    },
                    Err(e) => return Err(e.into()),
                };
                println!("written {} bytes", written);
            }
            // Try flush pending writes;

            if not_ready {
                return Ok(Async::NotReady);
            }
        }
    }
}
