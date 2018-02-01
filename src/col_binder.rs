use std::mem::size_of;

use odbc_sys::{SQLBindCol, SQLHSTMT, SQLLEN, SQLPOINTER, SQLUSMALLINT, SQL_C_CHAR};

use serde::Serialize;

use super::error::{OdbcResult, Result};
use super::bind_types::BindTypes;
use super::binder::{Binder, BinderImpl};

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
        }.check()
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
        }.check()
    }
}
