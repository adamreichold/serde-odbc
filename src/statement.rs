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
use std::ptr::null_mut;

use odbc_sys::{
    SQLAllocHandle, SQLExecute, SQLFetch, SQLFreeHandle, SQLFreeStmt, SQLPrepare, SQLHANDLE,
    SQLHSTMT, SQLINTEGER, SQL_CLOSE, SQL_HANDLE_STMT, SQL_NO_DATA,
};

use serde::ser::Serialize;

use super::col_binding::{ColBinding, RowSet};
use super::connection::Connection;
use super::error::{OdbcResult, Result};
use super::param_binding::ParamBinding;

pub struct Statement<P: ParamBinding, C: ColBinding> {
    stmt: SQLHSTMT,
    is_positioned: bool,
    params: P,
    cols: C,
}

impl<P: ParamBinding, C: ColBinding> Statement<P, C> {
    pub fn new(conn: &Connection, stmt_str: &str) -> Result<Self> {
        let mut stmt: SQLHANDLE = null_mut();

        unsafe { SQLAllocHandle(SQL_HANDLE_STMT, conn.handle(), &mut stmt) }.check()?;

        let stmt = stmt as SQLHSTMT;

        unsafe { SQLPrepare(stmt, stmt_str.as_ptr(), stmt_str.len() as SQLINTEGER) }.check()?;

        Ok(Statement {
            stmt,
            is_positioned: false,
            params: P::new(),
            cols: C::new(),
        })
    }

    pub fn handle(&self) -> SQLHANDLE {
        self.stmt as SQLHANDLE
    }

    pub fn params(&mut self) -> &mut P::Params {
        self.params.params()
    }

    pub fn cols(&self) -> &C::Cols {
        self.cols.cols()
    }

    pub fn exec(&mut self) -> Result<()> {
        if self.is_positioned {
            unsafe { SQLFreeStmt(self.stmt, SQL_CLOSE) }.check()?;

            self.is_positioned = false;
        }

        unsafe {
            self.params.bind(self.stmt)?;
            self.cols.bind(self.stmt)?;
        }

        unsafe { SQLExecute(self.stmt) }.check()
    }

    pub fn fetch(&mut self) -> Result<bool> {
        let rc = unsafe { SQLFetch(self.stmt) };

        rc.check()?;

        self.is_positioned = true;

        Ok(rc != SQL_NO_DATA && self.cols.fetch())
    }
}

impl<P: ParamBinding, C: Default + Clone + Serialize> Statement<P, RowSet<C>> {
    pub fn with_fetch_size(conn: &Connection, stmt_str: &str, fetch_size: usize) -> Result<Self> {
        let mut stmt = Self::new(conn, stmt_str)?;

        stmt.set_fetch_size(fetch_size);

        Ok(stmt)
    }

    pub fn fetch_size(&self) -> usize {
        self.cols.fetch_size()
    }

    pub fn set_fetch_size(&mut self, size: usize) {
        self.cols.set_fetch_size(size)
    }
}

impl<P: ParamBinding, C: ColBinding> Drop for Statement<P, C> {
    fn drop(&mut self) {
        let _ = unsafe { SQLFreeHandle(SQL_HANDLE_STMT, self.handle()) };
    }
}

#[cfg(test)]
mod tests {
    use super::super::col_binding::Cols;
    use super::super::connection::Environment;
    use super::super::param_binding::Params;
    use super::super::tests::CONN_STR;
    use super::*;

    #[test]
    fn exec_stmt() {
        let env = Environment::new().unwrap();
        let conn = Connection::new(&env, CONN_STR).unwrap();

        let mut stmt: Statement<Params<i32>, Cols<i32>> =
            Statement::new(&conn, "SELECT ?").unwrap();
        *stmt.params() = 42;
        stmt.exec().unwrap();
        assert!(stmt.fetch().unwrap());
        assert_eq!(42, *stmt.cols());
        assert!(!stmt.fetch().unwrap());
    }
}
