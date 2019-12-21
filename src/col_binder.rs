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

use odbc_sys::{SQLBindCol, SQLHSTMT, SQLLEN, SQLPOINTER, SQLUSMALLINT, SQL_C_CHAR};
use serde::ser::Serialize;

use super::bind_types::BindTypes;
use super::binder::{Binder, BinderImpl};
use super::error::{OdbcResult, Result};

struct ColBinder {
    stmt: SQLHSTMT,
    col_nr: SQLUSMALLINT,
}

pub unsafe fn bind_cols<C: Serialize>(stmt: SQLHSTMT, cols: &C) -> Result<()> {
    Binder::bind(ColBinder { stmt, col_nr: 0 }, cols)
}

impl BinderImpl for ColBinder {
    fn bind<T: BindTypes>(
        &mut self,
        value_ptr: SQLPOINTER,
        indicator_ptr: *mut SQLLEN,
    ) -> Result<()> {
        self.col_nr += 1;

        unsafe {
            SQLBindCol(
                self.stmt,
                self.col_nr,
                T::c_data_type(),
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
        self.col_nr += 1;

        unsafe {
            SQLBindCol(
                self.stmt,
                self.col_nr,
                SQL_C_CHAR,
                value_ptr,
                (length + 1) as SQLLEN,
                indicator_ptr,
            )
        }
        .check()
    }
}
