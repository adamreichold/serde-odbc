use std::fmt;
use std::error;

use odbc_sys::{
    SQLPOINTER,
    SQLRETURN, SQL_ERROR,
    SqlCDataType, SqlDataType,
};

use serde::ser;


pub fn sql_ptr< T: ?Sized >( value: &T ) -> SQLPOINTER {
    ((value as *const T) as *mut T) as SQLPOINTER
}

pub trait BindTypes {
    fn c_data_type() -> SqlCDataType;
    fn data_type() -> SqlDataType;
}

impl BindTypes for i32 {
    fn c_data_type() -> SqlCDataType {
        SqlCDataType::SQL_C_SLONG
    }
    fn data_type() -> SqlDataType {
        SqlDataType::SQL_INTEGER
    }
}

#[ derive( Debug ) ]
pub struct BindError {}

pub type BindResult = Result< (), BindError >;

impl BindError {

    pub fn rc( &self ) -> SQLRETURN {
        SQL_ERROR // TODO
    }
}

impl fmt::Display for BindError {

    fn fmt( &self, fmt: &mut fmt::Formatter ) -> Result< (), fmt::Error > {
        Ok( () ) // TODO
    }
}

impl error::Error for BindError {

    fn description( &self) -> &str {
        return "" // TODO
    }
}

impl ser::Error for BindError {

    fn custom< T >( msg: T ) -> Self {
        BindError{} // TODO
    }
}