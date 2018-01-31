use std::ptr::null;
use std::mem::size_of;
use std::default::Default;

use odbc_sys::{SQLSetStmtAttr, SQLHSTMT, SQLLEN, SQLPOINTER, SQL_ATTR_ROWS_FETCHED_PTR,
               SQL_ATTR_ROW_ARRAY_SIZE, SQL_ATTR_ROW_BIND_TYPE, SQL_SUCCESS};

use serde::ser::Serialize;

use super::bind_error::{BindError, BindResult};
use super::col_binder::bind_cols;

pub trait ColBinding {
    fn new() -> Self;

    type Cols;
    fn cols(&self) -> &Self::Cols;

    unsafe fn bind(&mut self, stmt: SQLHSTMT) -> BindResult;

    fn fetch(&mut self) -> bool;
}

pub trait FetchSize {
    fn fetch_size(&self) -> usize;
    fn set_fetch_size(&mut self, size: usize);
}

pub struct Cols<C: Default + Serialize> {
    data: C,
    last_data: *const C,
}

pub type NoCols = Cols<()>;

pub struct RowSet<C: Clone + Default + Serialize> {
    data: Vec<C>,
    last_data: *const C,
    last_size: usize,
    rows_fetched: SQLLEN,
}

impl<C: Default + Serialize> ColBinding for Cols<C> {
    fn new() -> Self {
        Cols {
            data: Default::default(),
            last_data: null(),
        }
    }

    type Cols = C;
    fn cols(&self) -> &Self::Cols {
        &self.data
    }

    unsafe fn bind(&mut self, stmt: SQLHSTMT) -> BindResult {
        let data = &self.data as *const C;

        if self.last_data != data {
            bind_cols(stmt, &*data)?;
            self.last_data = data;
        }

        Ok(())
    }

    fn fetch(&mut self) -> bool {
        true
    }
}

impl<C: Clone + Default + Serialize> RowSet<C> {
    unsafe fn bind_row_set(stmt: SQLHSTMT, size: usize, rows_fetched: &mut SQLLEN) -> BindResult {
        let rc = SQLSetStmtAttr(
            stmt,
            SQL_ATTR_ROW_BIND_TYPE,
            size_of::<C>() as SQLPOINTER,
            0,
        );
        if rc != SQL_SUCCESS {
            return Err(BindError {}); // TODO
        }

        let rc = SQLSetStmtAttr(stmt, SQL_ATTR_ROW_ARRAY_SIZE, size as SQLPOINTER, 0);
        if rc != SQL_SUCCESS {
            return Err(BindError {}); // TODO
        }

        let rc = SQLSetStmtAttr(
            stmt,
            SQL_ATTR_ROWS_FETCHED_PTR,
            (rows_fetched as *mut SQLLEN) as SQLPOINTER,
            0,
        );
        if rc != SQL_SUCCESS {
            return Err(BindError {}); // TODO
        }

        Ok(())
    }
}

impl<C: Clone + Default + Serialize> ColBinding for RowSet<C> {
    fn new() -> Self {
        RowSet {
            data: Vec::new(),
            last_data: null(),
            last_size: 0,
            rows_fetched: 0,
        }
    }

    type Cols = Vec<C>;
    fn cols(&self) -> &Self::Cols {
        &self.data
    }

    unsafe fn bind(&mut self, stmt: SQLHSTMT) -> BindResult {
        let capacity = self.data.capacity();
        self.data.resize(capacity, Default::default());

        let data = self.data.first().unwrap() as *const C;
        let size = self.data.len();

        if self.last_data != data {
            bind_cols(stmt, &*data)?;
            self.last_data = data;
        }

        if self.last_size != size {
            Self::bind_row_set(stmt, size, &mut self.rows_fetched)?;
            self.last_size = size;
        }

        Ok(())
    }

    fn fetch(&mut self) -> bool {
        self.data.truncate(self.rows_fetched as usize);
        self.rows_fetched != 0
    }
}

impl<C: Clone + Default + Serialize> FetchSize for RowSet<C> {
    fn fetch_size(&self) -> usize {
        self.data.capacity()
    }

    fn set_fetch_size(&mut self, size: usize) {
        let capacity = self.data.capacity();
        if size > capacity {
            self.data.reserve(size - capacity);
        }
    }
}
