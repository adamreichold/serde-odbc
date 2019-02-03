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
    SQLAllocHandle, SQLDriverConnect, SQLEndTran, SQLFreeHandle, SQLSetConnectAttr, SQLSetEnvAttr,
    SqlCompletionType, SQLHANDLE, SQLHDBC, SQLHENV, SQLSMALLINT, SQL_ATTR_AUTOCOMMIT,
    SQL_ATTR_CONNECTION_POOLING, SQL_ATTR_ODBC_VERSION, SQL_COMMIT, SQL_DRIVER_COMPLETE_REQUIRED,
    SQL_HANDLE_DBC, SQL_HANDLE_ENV, SQL_OV_ODBC3, SQL_ROLLBACK,
};

use crate::error::{OdbcResult, Result};

pub struct Environment(SQLHENV);

impl Environment {
    pub fn new() -> Result<Self> {
        let mut env: SQLHANDLE = null_mut();

        unsafe { SQLAllocHandle(SQL_HANDLE_ENV, null_mut(), &mut env) }.check()?;

        let env = env as SQLHENV;

        unsafe { SQLSetEnvAttr(env, SQL_ATTR_ODBC_VERSION, SQL_OV_ODBC3.into(), 0) }.check()?;
        unsafe { SQLSetEnvAttr(env, SQL_ATTR_CONNECTION_POOLING, null_mut(), 0) }.check()?;

        Ok(Environment(env))
    }

    pub unsafe fn handle(&self) -> SQLHANDLE {
        self.0 as SQLHANDLE
    }
}

impl Drop for Environment {
    fn drop(&mut self) {
        let _ = unsafe { SQLFreeHandle(SQL_HANDLE_ENV, self.handle()) };
    }
}

pub struct Connection(SQLHDBC);

impl Connection {
    pub fn new(env: &Environment, conn_str: &str) -> Result<Self> {
        let mut dbc: SQLHANDLE = null_mut();

        unsafe { SQLAllocHandle(SQL_HANDLE_DBC, env.handle(), &mut dbc) }.check()?;

        let dbc = dbc as SQLHDBC;

        unsafe {
            SQLDriverConnect(
                dbc,
                null_mut(),
                conn_str.as_ptr(),
                conn_str.len() as SQLSMALLINT,
                null_mut(),
                0,
                null_mut(),
                SQL_DRIVER_COMPLETE_REQUIRED,
            )
        }
        .check()?;

        unsafe { SQLSetConnectAttr(dbc, SQL_ATTR_AUTOCOMMIT, null_mut(), 0) }.check()?;

        Ok(Connection(dbc))
    }

    pub unsafe fn handle(&self) -> SQLHANDLE {
        self.0 as SQLHANDLE
    }

    pub fn begin(&self) -> Transaction {
        Transaction(Some(self))
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = unsafe { SQLFreeHandle(SQL_HANDLE_DBC, self.handle()) };
    }
}

pub struct Transaction<'conn>(Option<&'conn Connection>);

impl<'conn> Transaction<'conn> {
    pub fn commit(mut self) -> Result<()> {
        Self::end(self.0.take().unwrap(), SQL_COMMIT)
    }

    pub fn rollback(mut self) -> Result<()> {
        Self::end(self.0.take().unwrap(), SQL_ROLLBACK)
    }

    fn end(conn: &'conn Connection, completion_type: SqlCompletionType) -> Result<()> {
        unsafe { SQLEndTran(SQL_HANDLE_DBC, conn.handle(), completion_type) }.check()
    }
}

impl<'conn> Drop for Transaction<'conn> {
    fn drop(&mut self) {
        if let Some(conn) = self.0.take() {
            let _ = Self::end(conn, SQL_ROLLBACK);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::CONN_STR;
    use super::*;

    #[test]
    fn make_env() {
        Environment::new().unwrap();
    }

    #[test]
    fn make_conn() {
        let env = Environment::new().unwrap();
        Connection::new(&env, CONN_STR).unwrap();
    }

    #[test]
    fn commit_trans() {
        let env = Environment::new().unwrap();
        let conn = Connection::new(&env, CONN_STR).unwrap();

        let trans = conn.begin();
        trans.commit().unwrap();
    }
}
