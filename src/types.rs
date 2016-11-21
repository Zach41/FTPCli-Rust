use std::fmt;
use std::error::Error;
use std::convert::From;

#[derive(Debug)]
pub enum FtpError {
    ConnectionError(::std::io::Error),
    SecureError(String),         // not used
    InvalidResponse(String),
    InvalidAddress(::std::net::AddrParseError),
}

impl From<::std::io::Error> for FtpError {
    fn from(err: ::std::io::Error) -> FtpError {
        FtpError::ConnectionError(err)
    }
}

impl From<::std::net::AddrParseError> for FtpError {
    fn from(err: ::std::net::AddrParseError) -> FtpError {
        FtpError::InvalidAddress(err)
    }
}

pub type Result<T> = ::std::result::Result<T, FtpError>;

/// `Line` contains a command code and the contents of a line of text read from network
#[derive(Debug)]
pub struct Line(pub u32, pub String);

impl fmt::Display for FtpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &FtpError::ConnectionError(ref ioerr) => {
                write!(f, "FTP Connection Error: {}", ioerr)
            },
            &FtpError::SecureError(ref desc) =>  {
                write!(f, "FTP SecureError: {}", desc.clone())
            },
            &FtpError::InvalidResponse(ref desc) =>  {
                write!(f, "FTP InvalidResponse: {}", desc.clone())
            },
            &FtpError::InvalidAddress(ref perr) =>  {
                write!(f, "FTP InvalidAddress: {}", perr)
            },
        }
    }
}

impl Error for FtpError {
    fn description(&self) -> &str {
        match *self {
            FtpError::ConnectionError(ref ioerr)    => ioerr.description(),
            FtpError::SecureError(ref desc)         => desc.as_str(),
            FtpError::InvalidResponse(ref desc)     => desc.as_str(),
            FtpError::InvalidAddress(ref perr)      => perr.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            FtpError::ConnectionError(ref ioerr) => Some(ioerr),
            FtpError::SecureError(_) => None,
            FtpError::InvalidResponse(_) => None,
            FtpError::InvalidAddress(ref perr) => Some(perr),
        }
    }
}

