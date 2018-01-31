use std::ptr::null_mut;
use std::marker::PhantomData;

use odbc_sys::{SQLAllocHandle, SQLDriverConnect, SQLEndTran, SQLFreeHandle, SQLSetConnectAttr,
               SQLSetEnvAttr, SQL_OV_ODBC3, SqlCompletionType, SQLHANDLE, SQLHDBC, SQLHENV,
               SQLRETURN, SQLSMALLINT, SQL_ATTR_AUTOCOMMIT, SQL_ATTR_CONNECTION_POOLING,
               SQL_ATTR_ODBC_VERSION, SQL_COMMIT, SQL_DRIVER_COMPLETE_REQUIRED, SQL_HANDLE_DBC,
               SQL_HANDLE_ENV, SQL_ROLLBACK, SQL_SUCCESS};

pub struct Environment(SQLHENV);

impl Environment {
    pub fn new() -> Result<Environment, SQLRETURN> {
        let mut env: SQLHANDLE = null_mut();

        let rc = unsafe { SQLAllocHandle(SQL_HANDLE_ENV, null_mut(), &mut env) };
        if rc != SQL_SUCCESS {
            return Err(rc);
        }

        let env = env as SQLHENV;

        let rc = unsafe { SQLSetEnvAttr(env, SQL_ATTR_ODBC_VERSION, SQL_OV_ODBC3.into(), 0) };
        if rc != SQL_SUCCESS {
            return Err(rc);
        }

        let rc = unsafe { SQLSetEnvAttr(env, SQL_ATTR_CONNECTION_POOLING, null_mut(), 0) };
        if rc != SQL_SUCCESS {
            return Err(rc);
        }

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

pub struct Connection<'env>(SQLHDBC, PhantomData<&'env Environment>);

impl<'env> Connection<'env> {
    pub fn new(env: &'env Environment, conn_str: &str) -> Result<Connection<'env>, SQLRETURN> {
        let mut dbc: SQLHANDLE = null_mut();

        let rc = unsafe { SQLAllocHandle(SQL_HANDLE_DBC, env.handle(), &mut dbc) };
        if rc != SQL_SUCCESS {
            return Err(rc);
        }

        let dbc = dbc as SQLHDBC;

        let rc = unsafe {
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
        };
        if rc != SQL_SUCCESS {
            return Err(rc);
        }

        let rc = unsafe { SQLSetConnectAttr(dbc, SQL_ATTR_AUTOCOMMIT, null_mut(), 0) };
        if rc != SQL_SUCCESS {
            return Err(rc);
        }

        Ok(Connection(dbc, PhantomData))
    }

    pub unsafe fn handle(&self) -> SQLHANDLE {
        self.0 as SQLHANDLE
    }

    pub fn begin<'conn>(&'conn self) -> Transaction<'conn, 'env> {
        Transaction(Some(self))
    }
}

impl<'env> Drop for Connection<'env> {
    fn drop(&mut self) {
        let _ = unsafe { SQLFreeHandle(SQL_HANDLE_DBC, self.handle()) };
    }
}

pub struct Transaction<'conn, 'env: 'conn>(Option<&'conn Connection<'env>>);

impl<'conn, 'env> Transaction<'conn, 'env> {
    pub fn commit(mut self) -> Result<(), SQLRETURN> {
        Self::end(self.0.take().unwrap(), SQL_COMMIT)
    }

    pub fn rollback(mut self) -> Result<(), SQLRETURN> {
        Self::end(self.0.take().unwrap(), SQL_ROLLBACK)
    }

    fn end(
        conn: &'conn Connection<'env>,
        completion_type: SqlCompletionType,
    ) -> Result<(), SQLRETURN> {
        let rc = unsafe { SQLEndTran(SQL_HANDLE_DBC, conn.handle(), completion_type) };
        match rc {
            SQL_SUCCESS => Ok(()),
            rc => Err(rc),
        }
    }
}

impl<'conn, 'env> Drop for Transaction<'conn, 'env> {
    fn drop(&mut self) {
        if let Some(conn) = self.0.take() {
            let _ = Self::end(conn, SQL_ROLLBACK);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tests::CONN_STR;

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
