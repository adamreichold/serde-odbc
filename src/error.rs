/*
This file is part of serde-odbc.

serde-odbc is free software: you can redistribute it and/or modify
it under the terms of the GNU Lesser General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

serde-odbc is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Lesser General Public License for more details.

You should have received a copy of the GNU Lesser General Public License
along with serde-odbc.  If not, see <http://www.gnu.org/licenses/>.
*/
use std::error;
use std::fmt;
use std::result;

use odbc_sys::{SQLRETURN, SQL_NO_DATA, SQL_SUCCESS, SQL_SUCCESS_WITH_INFO};
use serde::ser;

#[derive(Debug)]
pub enum Error {
    Odbc(SQLRETURN),
    Serde(String),
}

pub type Result<T> = result::Result<T, Error>;

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Odbc(_) => "ODBC error",
            Error::Serde(_) => "Serde error",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        match *self {
            Error::Odbc(rc) => write!(fmt, "ODBC error: {:?}", rc),
            Error::Serde(ref msg) => write!(fmt, "Serde error: {}", msg),
        }
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Serde(msg.to_string())
    }
}

pub trait OdbcResult {
    fn check(self) -> Result<()>;
}

impl OdbcResult for SQLRETURN {
    fn check(self) -> Result<()> {
        match self {
            SQL_SUCCESS | SQL_SUCCESS_WITH_INFO | SQL_NO_DATA => Ok(()),
            rc => Err(Error::Odbc(rc)),
        }
    }
}
