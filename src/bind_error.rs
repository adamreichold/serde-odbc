use std::fmt;
use std::error;

use odbc_sys::{SQLRETURN, SQL_ERROR};

use serde::ser;

#[derive(Debug)]
pub struct BindError {}

pub type BindResult = Result<(), BindError>;

impl BindError {
    pub fn rc(&self) -> SQLRETURN {
        SQL_ERROR // TODO
    }
}

impl fmt::Display for BindError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        Ok(()) // TODO
    }
}

impl error::Error for BindError {
    fn description(&self) -> &str {
        return ""; // TODO
    }
}

impl ser::Error for BindError {
    fn custom<T>(msg: T) -> Self {
        BindError {} // TODO
    }
}
