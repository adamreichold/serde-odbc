use std::mem::size_of;

use odbc_sys::{SQLBindParameter, SQLHSTMT, SQLLEN, SQLPOINTER, SQLULEN, SQLUSMALLINT, SQL_C_CHAR,
               SQL_PARAM_INPUT, SQL_SUCCESS, SQL_SUCCESS_WITH_INFO, SQL_VARCHAR};

use serde::ser::Serialize;

use super::bind_error::{BindError, BindResult};
use super::bind_types::BindTypes;
use super::binder::{Binder, BinderImpl};

struct ParamBinder {
    stmt: SQLHSTMT,
    param_nr: SQLUSMALLINT,
}

pub unsafe fn bind_params<P: Serialize>(stmt: SQLHSTMT, params: &P) -> BindResult {
    Binder::bind(ParamBinder { stmt, param_nr: 0 }, params)
}

impl BinderImpl for ParamBinder {
    fn bind<T: BindTypes>(
        &mut self,
        value_ptr: SQLPOINTER,
        indicator_ptr: *mut SQLLEN,
    ) -> BindResult {
        self.param_nr += 1;

        let rc = unsafe {
            SQLBindParameter(
                self.stmt,
                self.param_nr,
                SQL_PARAM_INPUT,
                T::c_data_type(),
                T::data_type(),
                0,
                0,
                value_ptr,
                size_of::<T>() as SQLLEN,
                indicator_ptr,
            )
        };

        match rc {
            SQL_SUCCESS | SQL_SUCCESS_WITH_INFO => Ok(()),
            rc => Err(BindError {}), // TODO
        }
    }

    fn bind_str(
        &mut self,
        length: usize,
        value_ptr: SQLPOINTER,
        indicator_ptr: *mut SQLLEN,
    ) -> BindResult {
        self.param_nr += 1;

        let rc = unsafe {
            SQLBindParameter(
                self.stmt,
                self.param_nr,
                SQL_PARAM_INPUT,
                SQL_C_CHAR,
                SQL_VARCHAR,
                length as SQLULEN,
                0,
                value_ptr,
                1,
                indicator_ptr,
            )
        };

        match rc {
            SQL_SUCCESS | SQL_SUCCESS_WITH_INFO => Ok(()),
            rc => Err(BindError {}), // TODO
        }
    }
}
