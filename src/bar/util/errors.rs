use std::error::Error as StdError;
use std::fmt;
use std::io;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(Box<StdError + Send + Sync>);

impl Error {
    pub fn new<E>(err: E) -> Error
        where E: Into<Box<StdError + Send + Sync>>
    {
        Error(err.into())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        self.0.description()
    }

    fn cause(&self) -> Option<&StdError> {
        self.0.cause()
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::new(err)
    }
}

impl From<fmt::Error> for Error {
    fn from(err: fmt::Error) -> Error {
        Error::new(format!("{}", err))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(fmt)
    }
}
