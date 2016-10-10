use std::io;

use httparse;
use futures::{Async, Poll};

// TODO: parse current buffer;
//      if request is complete:
//          check if request has body:
//              advance state to 'body parser' (allowing to skip body);
//                  check if body is read or skipped:
//                      advance state to 'new request';
//                  otherwise -> continue next call;
//          otherwise -> advance state to 'new request';
//      if request has error:
//          (check proper errors of parser);
//          if its body of previous request:
//              must be able to drop previous request
//              respond with ErrorResponse
//              and close connection;
//      otherwise -> continue next call;
//
// note: parser can shortcut request processing
//      by yielding error response;

/// Parsing states.
#[derive(Debug,PartialEq)]
pub enum State {
    /// Reading new request headers
    NewRequest,
    /// Not all request headers read
    ReadingRequest,
    /// Parsing / checking headers (eg: body length, etc)
    ProcessHeaders,
    /// Reading request body
    BodyProgress,
    Done,
}


pub fn parse<'a>(buf: &'a [u8], s: State, _: &mut super::request::Request) -> io::Result<State> {
    let res = match s {
        State::NewRequest | State::ReadingRequest => {
            let mut headers = [httparse::EMPTY_HEADER; 16];
            let mut req = httparse::Request::new(&mut headers);
            let status = match req.parse(buf) {
                Ok(status) => status,
                Err(e) => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other, e.to_string()))
                }
            };
            match status {
                httparse::Status::Complete(_) => {
                    // TODO: construct request
                    //  httparse done parsing request;
                    //  so we can assume it is complete;
                    //  now we must Build minihttp::Request
                    //  and, probably prepare Response
                    //
                    //  Do we need any other steps?
                    //
                    //  When parse is ready, we can continue and
                    //  process headers;
                    //  after headers are processed
                    //  we can verify request if it's correct (body length, etc)
                    //  If request is incorrect -> we must drop it;
                    //  (generate response early);
                    //
                    //  Also we must verify if connection should be kept alive
                    //  or upgradet to another state.
                    //
                    //  So on every 'bytes ready':
                    //  * create new request;
                    //  * parse bytes into request;
                    //  * check correctness of it;
                    //  * pass request to service;
                    //  * wait response ready;
                    //  * write response to output buffer;

                    //match res.from(req) {
                    //    Ok(()) => {},
                    //    Err(e) => {
                    //        return Err(io::Error::new(
                    //            io::ErrorKind::Other, e.to_string()));
                    //    },
                    //};
                    State::ProcessHeaders
                },
                httparse::Status::Partial => {
                    State::ReadingRequest
                },
            }
        },
        State::ProcessHeaders => {
            // Check keep-alive;
            // Check if body exists and it's length;
            // ???
            State::Done // or State::BodyProgress
        },
        State::BodyProgress => {
            State::Done
        },
        State::Done => {
            State::NewRequest
        },
    };
    Ok(res)
}
