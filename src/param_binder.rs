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
use std::mem::size_of;

use odbc_sys::{
    SQLBindParameter, SQLHSTMT, SQLLEN, SQLPOINTER, SQLULEN, SQLUSMALLINT, SQL_C_CHAR,
    SQL_PARAM_INPUT, SQL_VARCHAR,
};
use serde::ser::Serialize;

use super::bind_types::BindTypes;
use super::binder::{Binder, BinderImpl};
use super::error::{OdbcResult, Result};

struct ParamBinder {
    stmt: SQLHSTMT,
    param_nr: SQLUSMALLINT,
}

pub unsafe fn bind_params<P: Serialize>(stmt: SQLHSTMT, params: &P) -> Result<()> {
    Binder::bind(ParamBinder { stmt, param_nr: 0 }, params)
}

impl BinderImpl for ParamBinder {
    fn bind<T: BindTypes>(
        &mut self,
        value_ptr: SQLPOINTER,
        indicator_ptr: *mut SQLLEN,
    ) -> Result<()> {
        self.param_nr += 1;

        unsafe {
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
        }
        .check()
    }

    fn bind_str(
        &mut self,
        length: usize,
        value_ptr: SQLPOINTER,
        indicator_ptr: *mut SQLLEN,
    ) -> Result<()> {
        self.param_nr += 1;

        unsafe {
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
        }
        .check()
    }
}
