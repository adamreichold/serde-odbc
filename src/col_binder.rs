use std::mem::size_of;

use odbc_sys::{SQLBindCol, SQLHSTMT, SQLLEN, SQLPOINTER, SQLRETURN, SQLUSMALLINT, SQL_SUCCESS,
               SQL_SUCCESS_WITH_INFO};

use serde::Serialize;

use super::bind_error::{BindError, BindResult};
use super::bind_types::BindTypes;
use super::binder::{Binder, BinderImpl};

struct ColBinder {
    stmt: SQLHSTMT,
    col_nr: SQLUSMALLINT,
}

pub unsafe fn bind_cols<Cols: Serialize>(stmt: SQLHSTMT, cols: &Cols) -> Result<(), SQLRETURN> {
    let mut binder = Binder::new(ColBinder { stmt, col_nr: 0 });

    cols.serialize(&mut binder).map_err(|err| err.rc())
}

impl BinderImpl for ColBinder {
    fn bind<T: BindTypes>(
        &mut self,
        value_ptr: SQLPOINTER,
        indicator_ptr: *mut SQLLEN,
    ) -> BindResult {
        self.col_nr += 1;

        let rc = unsafe {
            SQLBindCol(
                self.stmt,
                self.col_nr,
                T::c_data_type(),
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
}
