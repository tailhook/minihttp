use std::io;

use httparse;

use super::request::Request;

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
    ReadHeaders,
    /// Reading request body
    BodyProgress,
    Done,
}


pub fn parse(buf: &[u8], s: State, res: &mut Request) -> io::Result<State> {
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
                    match res.from(req) {
                        Ok(()) => {},
                        Err(e) => {
                            return Err(io::Error::new(
                                io::ErrorKind::Other, e.to_string()));
                        },
                    };
                    State::ReadHeaders
                },
                httparse::Status::Partial => {
                    State::ReadingRequest
                },
            }
        },
        State::ReadHeaders => {
            State::Done
        },
        State::BodyProgress => {
            State::Done
        },
        State::Done => {unreachable!()},
    };
    Ok(res)
}
