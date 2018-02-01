use std::error;
use std::result;
use std::fmt;

use odbc_sys::{SQLRETURN, SQL_NO_DATA, SQL_SUCCESS, SQL_SUCCESS_WITH_INFO};

use serde::ser;

#[derive(Debug)]
pub enum Error {
    OdbcError(SQLRETURN),
    SerdeError(String),
}

pub type Result<T> = result::Result<T, Error>;

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::OdbcError(_) => "ODBC error",
            &Error::SerdeError(_) => "Serde error",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        match self {
            &Error::OdbcError(rc) => write!(fmt, "ODBC error: {:?}", rc),
            &Error::SerdeError(ref msg) => write!(fmt, "Serde error: {}", msg),
        }
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::SerdeError(msg.to_string())
    }
}

pub trait OdbcResult {
    fn check(self) -> Result<()>;
}

impl OdbcResult for SQLRETURN {
    fn check(self) -> Result<()> {
        match self {
            SQL_SUCCESS | SQL_SUCCESS_WITH_INFO | SQL_NO_DATA => Ok(()),
            rc => Err(Error::OdbcError(rc)),
        }
    }
}
