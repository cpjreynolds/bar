use std::error::Error as StdError;
use std::fmt;
use std::io;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    desc: String,
    detail: Vec<String>,
    cause: Option<Box<StdError>>,
}

impl Error {
    pub fn new(desc: &str) -> Error {
        Error {
            desc: String::from(desc),
            detail: Vec::new(),
            cause: None,
        }
    }

    pub fn with_cause<E>(desc: &str, cause: E) -> Error
        where E: StdError + 'static
    {
        let mut err = Error::new(desc);
        err.add_detail(cause.description());
        err.cause = Some(box cause);

        err
    }

    pub fn add_detail(&mut self, detail: &str) {
        self.detail.push(String::from(detail));
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        &self.desc[..]
    }

    fn cause(&self) -> Option<&StdError> {
        if let Some(ref cause) = self.cause {
            Some(&**cause)
        } else {
            None
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::with_cause("I/O error", err)
    }
}

impl From<fmt::Error> for Error {
    fn from(err: fmt::Error) -> Error {
        let mut error = Error::new("format error");
        error.add_detail(&format!("{}", err));
        error
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "{}\n", self.desc));

        for l in &self.detail {
            try!(writeln!(f, "{}", l));
        }

        Ok(())
    }
}
