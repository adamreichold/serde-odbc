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
use std::ptr::null;

use odbc_sys::{
    SQLSetStmtAttr, SQLHSTMT, SQLLEN, SQLPOINTER, SQL_ATTR_ROWS_FETCHED_PTR,
    SQL_ATTR_ROW_ARRAY_SIZE, SQL_ATTR_ROW_BIND_TYPE,
};
use serde::ser::Serialize;

use super::col_binder::bind_cols;
use super::error::{OdbcResult, Result};

pub trait ColBinding {
    fn new() -> Self;

    type Cols;
    fn cols(&self) -> &Self::Cols;

    unsafe fn bind(&mut self, stmt: SQLHSTMT) -> Result<()>;

    fn fetch(&mut self) -> bool;
}

pub struct Cols<C: Copy + Default + Serialize> {
    data: C,
    last_data: *const C,
}

pub struct NoCols {
    data: (),
}

pub struct RowSet<C: Copy + Default + Serialize> {
    data: Vec<C>,
    last_data: *const C,
    last_size: usize,
    rows_fetched: SQLLEN,
}

impl<C: Copy + Default + Serialize> ColBinding for Cols<C> {
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

    unsafe fn bind(&mut self, stmt: SQLHSTMT) -> Result<()> {
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

impl ColBinding for NoCols {
    fn new() -> Self {
        NoCols { data: () }
    }

    type Cols = ();
    fn cols(&self) -> &Self::Cols {
        &self.data
    }

    unsafe fn bind(&mut self, _stmt: SQLHSTMT) -> Result<()> {
        Ok(())
    }

    fn fetch(&mut self) -> bool {
        true
    }
}

impl<C: Copy + Default + Serialize> ColBinding for RowSet<C> {
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

    unsafe fn bind(&mut self, stmt: SQLHSTMT) -> Result<()> {
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

impl<C: Copy + Default + Serialize> RowSet<C> {
    pub fn fetch_size(&self) -> usize {
        self.data.capacity()
    }

    pub fn set_fetch_size(&mut self, size: usize) {
        let capacity = self.data.capacity();
        if size > capacity {
            self.data.reserve(size - capacity);
        }
    }

    unsafe fn bind_row_set(stmt: SQLHSTMT, size: usize, rows_fetched: &mut SQLLEN) -> Result<()> {
        SQLSetStmtAttr(
            stmt,
            SQL_ATTR_ROW_BIND_TYPE,
            size_of::<C>() as SQLPOINTER,
            0,
        )
        .check()?;

        SQLSetStmtAttr(stmt, SQL_ATTR_ROW_ARRAY_SIZE, size as SQLPOINTER, 0).check()?;

        SQLSetStmtAttr(
            stmt,
            SQL_ATTR_ROWS_FETCHED_PTR,
            (rows_fetched as *mut SQLLEN) as SQLPOINTER,
            0,
        )
        .check()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        connection::{Connection, Environment},
        param_binding::{NoParams, Params},
        statement::Statement,
        tests::CONN_STR,
    };

    #[test]
    fn bind_row_set() {
        let env = Environment::new().unwrap();
        let conn = Connection::new(&env, CONN_STR).unwrap();

        {
            let mut stmt: Statement<NoParams, NoCols> =
                Statement::new(&conn, "CREATE TEMPORARY TABLE tbl (col INTEGER NOT NULL)").unwrap();
            stmt.exec().unwrap();
        }

        {
            let mut stmt: Statement<Params<i32>, NoCols> =
                Statement::new(&conn, "INSERT INTO tbl (col) VALUES (?)").unwrap();
            for i in 0..128 {
                *stmt.params() = i;
                stmt.exec().unwrap();
            }
        }

        {
            let mut stmt: Statement<NoParams, RowSet<i32>> =
                Statement::new(&conn, "SELECT col FROM tbl ORDER BY col").unwrap();
            stmt.set_fetch_size(32);
            assert!(32 == stmt.fetch_size());
            stmt.exec().unwrap();
            for i in 0..4 {
                assert!(stmt.fetch().unwrap());
                assert_eq!(32, stmt.cols().len());
                stmt.cols().iter().enumerate().for_each(|(j, cols)| {
                    assert_eq!(32 * i + j, *cols as usize);
                });
            }
            assert!(!stmt.fetch().unwrap());
        }

        {
            let mut stmt: Statement<NoParams, RowSet<i32>> =
                Statement::new(&conn, "SELECT col FROM tbl ORDER BY col").unwrap();
            stmt.set_fetch_size(256);
            assert!(256 == stmt.fetch_size());
            stmt.exec().unwrap();
            assert!(stmt.fetch().unwrap());
            assert_eq!(128, stmt.cols().len());
            stmt.cols().iter().enumerate().for_each(|(i, cols)| {
                assert_eq!(i, *cols as usize);
            });
            assert!(!stmt.fetch().unwrap());
        }
    }
}
