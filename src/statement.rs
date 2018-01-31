use std::ptr::null_mut;
use std::marker::PhantomData;

use odbc_sys::{SQLAllocHandle, SQLExecute, SQLFetch, SQLFreeHandle, SQLFreeStmt, SQLPrepare,
               SQLHANDLE, SQLHSTMT, SQLINTEGER, SQLRETURN, SQL_CLOSE, SQL_HANDLE_STMT,
               SQL_NO_DATA, SQL_SUCCESS, SQL_SUCCESS_WITH_INFO};

use super::connection::Connection;
use super::param_binding::ParamBinding;
use super::col_binding::{ColBinding, FetchSize};

pub struct Statement<'conn, 'env: 'conn, P: ParamBinding, C: ColBinding> {
    conn: PhantomData<&'conn Connection<'env>>,
    stmt: SQLHSTMT,
    is_positioned: bool,
    params: P,
    cols: C,
}

impl<'conn, 'env, P: ParamBinding, C: ColBinding> Statement<'conn, 'env, P, C> {
    pub fn new(conn: &'conn Connection<'env>, stmt_str: &str) -> Result<Self, SQLRETURN> {
        let mut stmt: SQLHANDLE = null_mut();

        let rc = unsafe { SQLAllocHandle(SQL_HANDLE_STMT, conn.handle(), &mut stmt) };
        if rc != SQL_SUCCESS {
            return Err(rc);
        }

        let stmt = stmt as SQLHSTMT;

        let rc = unsafe { SQLPrepare(stmt, stmt_str.as_ptr(), stmt_str.len() as SQLINTEGER) };
        if rc != SQL_SUCCESS {
            return Err(rc);
        }

        Ok(Statement {
            conn: PhantomData,
            stmt,
            is_positioned: false,
            params: P::new(),
            cols: C::new(),
        })
    }

    pub unsafe fn handle(&self) -> SQLHANDLE {
        self.stmt as SQLHANDLE
    }

    pub fn params(&mut self) -> &mut P::Params {
        self.params.params()
    }

    pub fn cols(&self) -> &C::Cols {
        self.cols.cols()
    }

    pub fn exec(&mut self) -> Result<(), SQLRETURN> {
        if self.is_positioned {
            let rc = unsafe { SQLFreeStmt(self.stmt, SQL_CLOSE) };
            if rc != SQL_SUCCESS {
                return Err(rc);
            }

            self.is_positioned = false;
        }

        unsafe {
            self.params.bind(self.stmt).map_err(|err| err.rc())?;
            self.cols.bind(self.stmt).map_err(|err| err.rc())?;
        }

        let rc = unsafe { SQLExecute(self.stmt) };
        if rc != SQL_SUCCESS && rc != SQL_NO_DATA {
            return Err(rc);
        }

        Ok(())
    }

    pub fn fetch(&mut self) -> Result<bool, SQLRETURN> {
        let rc = unsafe { SQLFetch(self.stmt) };
        match rc {
            SQL_SUCCESS | SQL_SUCCESS_WITH_INFO => (),
            SQL_NO_DATA => return Ok(false),
            rc => return Err(rc),
        }

        self.is_positioned = true;

        Ok(self.cols.fetch())
    }
}

impl<'conn, 'env, P: ParamBinding, C: ColBinding> Drop for Statement<'conn, 'env, P, C> {
    fn drop(&mut self) {
        let _ = unsafe { SQLFreeHandle(SQL_HANDLE_STMT, self.handle()) };
    }
}

impl<'conn, 'env, P: ParamBinding, C: ColBinding + FetchSize> FetchSize
    for Statement<'conn, 'env, P, C>
{
    fn fetch_size(&self) -> usize {
        self.cols.fetch_size()
    }

    fn set_fetch_size(&mut self, size: usize) {
        self.cols.set_fetch_size(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::connection::Environment;
    use super::super::nullable::Nullable;
    use super::super::param_binding::{NoParams, ParamSet, Params};
    use super::super::col_binding::{Cols, NoCols, RowSet};
    use super::super::tests::CONN_STR;

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

    #[test]
    fn bind_nullable_param() {
        let env = Environment::new().unwrap();
        let conn = Connection::new(&env, CONN_STR).unwrap();

        let mut stmt: Statement<Params<Nullable<i32>>, Cols<i32>> =
            Statement::new(&conn, "SELECT ?").unwrap();
        *stmt.params() = Some(42).into();
        stmt.exec().unwrap();
        assert!(stmt.fetch().unwrap());
        assert_eq!(42, *stmt.cols());
        assert!(!stmt.fetch().unwrap());
    }

    #[test]
    fn bind_nullable_col() {
        let env = Environment::new().unwrap();
        let conn = Connection::new(&env, CONN_STR).unwrap();

        let mut stmt: Statement<Params<i32>, Cols<Nullable<i32>>> =
            Statement::new(&conn, "SELECT ?").unwrap();
        *stmt.params() = 42;
        stmt.exec().unwrap();
        assert!(stmt.fetch().unwrap());
        assert_eq!(Some(42), (*stmt.cols()).into());
        assert!(!stmt.fetch().unwrap());
    }

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
