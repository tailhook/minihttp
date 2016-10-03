use std::error;
use std::fmt;

use httparse;


#[derive(Debug,PartialEq)]
pub enum Error {
    ParseError(httparse::Error),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ParseError(_) => "httparse error",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParseError(ref err) => {
                write!(f, "parse error: {}", err)
            },
        }
    }
}

#[cfg(test)]
mod test {
    use httparse;
    use error::Error as MyError;
    use std::error::Error;

    #[test]
    fn test_parse_error() {

        let e = MyError::ParseError(httparse::Error::HeaderName);
        assert_eq!(e.description(), "httparse error");
        assert!(e.cause().is_none());
        assert_eq!(format!("{}", e), "parse error: invalid header name");
        assert_eq!(format!("{:?}", e), "ParseError(HeaderName)");
    }
}
