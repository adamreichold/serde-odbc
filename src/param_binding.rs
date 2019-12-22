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
    SQLSetStmtAttr, SQLHSTMT, SQLPOINTER, SQL_ATTR_PARAMSET_SIZE, SQL_ATTR_PARAM_BIND_TYPE,
};
use serde::ser::Serialize;

use super::error::{OdbcResult, Result};
use super::param_binder::bind_params;

pub trait ParamBinding {
    fn new() -> Self;

    type Params;
    fn params(&mut self) -> &mut Self::Params;

    unsafe fn bind(&mut self, stmt: SQLHSTMT) -> Result<()>;
}

pub struct Params<P: Copy + Default + Serialize> {
    data: P,
    last_data: *const P,
}

pub struct NoParams {
    data: (),
}

pub struct ParamSet<P: Copy + Serialize> {
    data: Vec<P>,
    last_data: *const P,
    last_size: usize,
}

impl<P: Copy + Default + Serialize> ParamBinding for Params<P> {
    fn new() -> Self {
        Params {
            data: Default::default(),
            last_data: null(),
        }
    }

    type Params = P;
    fn params(&mut self) -> &mut Self::Params {
        &mut self.data
    }

    unsafe fn bind(&mut self, stmt: SQLHSTMT) -> Result<()> {
        let data = &self.data as *const P;

        if self.last_data != data {
            bind_params(stmt, &*data)?;
            self.last_data = data;
        }

        Ok(())
    }
}

impl ParamBinding for NoParams {
    fn new() -> Self {
        NoParams { data: () }
    }

    type Params = ();
    fn params(&mut self) -> &mut Self::Params {
        &mut self.data
    }

    unsafe fn bind(&mut self, _stmt: SQLHSTMT) -> Result<()> {
        Ok(())
    }
}

impl<P: Copy + Serialize> ParamBinding for ParamSet<P> {
    fn new() -> Self {
        ParamSet {
            data: Vec::new(),
            last_data: null(),
            last_size: 0,
        }
    }

    type Params = Vec<P>;
    fn params(&mut self) -> &mut Self::Params {
        &mut self.data
    }

    unsafe fn bind(&mut self, stmt: SQLHSTMT) -> Result<()> {
        let data = self.data.first().unwrap() as *const P;
        let size = self.data.len();

        if self.last_data != data {
            bind_params(stmt, &*data)?;
            self.last_data = data;
        }

        if self.last_size != size {
            Self::bind_param_set(stmt, size)?;
            self.last_size = size;
        }

        Ok(())
    }
}

impl<P: Copy + Serialize> ParamSet<P> {
    unsafe fn bind_param_set(stmt: SQLHSTMT, size: usize) -> Result<()> {
        SQLSetStmtAttr(
            stmt,
            SQL_ATTR_PARAM_BIND_TYPE,
            size_of::<P>() as SQLPOINTER,
            0,
        )
        .check()?;

        SQLSetStmtAttr(stmt, SQL_ATTR_PARAMSET_SIZE, size as SQLPOINTER, 0).check()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        col_binding::{Cols, NoCols},
        connection::{Connection, Environment},
        statement::Statement,
        tests::CONN_STR,
    };

    #[test]
    fn bind_param_set() {
        let env = Environment::new().unwrap();
        let conn = Connection::new(&env, CONN_STR).unwrap();

        {
            let mut stmt: Statement<NoParams, NoCols> =
                Statement::new(&conn, "CREATE TEMPORARY TABLE tbl (col INTEGER NOT NULL)").unwrap();
            stmt.exec().unwrap();
        }

        {
            let mut stmt: Statement<ParamSet<i32>, NoCols> =
                Statement::new(&conn, "INSERT INTO tbl (col) VALUES (?)").unwrap();
            for i in 0..128 {
                stmt.params().push(i);
            }
            stmt.exec().unwrap();
        }

        {
            let mut stmt: Statement<NoParams, Cols<i32>> =
                Statement::new(&conn, "SELECT col FROM tbl ORDER BY col").unwrap();
            stmt.exec().unwrap();
            for i in 0..128 {
                assert!(stmt.fetch().unwrap());
                assert_eq!(i, *stmt.cols());
            }
            assert!(!stmt.fetch().unwrap());
        }
    }
}
