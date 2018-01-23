use std::ptr::null_mut;
use std::marker::PhantomData;

use odbc_sys::{
    SQLSMALLINT,
    SQLRETURN, SQL_SUCCESS,
    SQLHANDLE, SQLHENV, SQLHDBC,
    SQLAllocHandle, SQLFreeHandle, SQL_HANDLE_ENV, SQL_HANDLE_DBC,
    SQLSetEnvAttr, SQL_ATTR_ODBC_VERSION, SQL_OV_ODBC3, SQL_ATTR_CONNECTION_POOLING,
    SQLDriverConnect, SQL_DRIVER_COMPLETE_REQUIRED,
};


pub struct Environment( SQLHENV );

impl Environment {

    pub fn new() -> Result<Environment, SQLRETURN> {
        let mut env: SQLHANDLE = null_mut();
        
        let rc = unsafe { SQLAllocHandle( SQL_HANDLE_ENV, null_mut(), &mut env ) };
        if rc != SQL_SUCCESS {
            return Err( rc );
        }

        let env = env as SQLHENV;

        let rc = unsafe { SQLSetEnvAttr( env, SQL_ATTR_ODBC_VERSION, SQL_OV_ODBC3.into(), 0 ) };
        if rc != SQL_SUCCESS {
            return Err( rc );
        }

        let rc = unsafe { SQLSetEnvAttr( env, SQL_ATTR_CONNECTION_POOLING, null_mut(), 0 ) };
        if rc != SQL_SUCCESS {
            return Err( rc );
        }
        
        Ok( Environment( env ) )
    }

    pub unsafe fn handle( &self ) -> SQLHANDLE {
        self.0 as SQLHANDLE
    }
}

impl Drop for Environment {

    fn drop( &mut self ) {
        let _ = unsafe { SQLFreeHandle( SQL_HANDLE_ENV, self.handle() ) };
    }
}


pub struct Connection< 'env >( SQLHDBC, PhantomData< &'env Environment > );

impl< 'env > Connection< 'env > {

    pub fn new( env: &'env Environment, conn_str: &str ) -> Result< Connection< 'env >, SQLRETURN > {
        let mut dbc: SQLHANDLE = null_mut();

        let rc = unsafe { SQLAllocHandle( SQL_HANDLE_DBC, env.handle(), &mut dbc ) };
        if rc != SQL_SUCCESS {
            return Err( rc );
        }

        let dbc = dbc as SQLHDBC;

        let rc = unsafe { SQLDriverConnect( dbc, null_mut(), conn_str.as_ptr(), conn_str.len() as SQLSMALLINT, null_mut(), 0, null_mut(), SQL_DRIVER_COMPLETE_REQUIRED ) };
        if rc != SQL_SUCCESS {
            return Err( rc );
        }

        // TODO: Disable auto commit

        Ok( Connection( dbc, PhantomData ) )
    }

    pub unsafe fn handle( &self ) -> SQLHANDLE {
        self.0 as SQLHANDLE
    }

    pub fn begin< 'conn >( &'conn self ) -> Transaction< 'conn, 'env > {
        Transaction( Some( self ) )
    }
}

impl< 'env > Drop for Connection< 'env > {

    fn drop( &mut self ) {
        let _ = unsafe { SQLFreeHandle( SQL_HANDLE_DBC, self.handle() ) };
    }
}


pub struct Transaction< 'conn, 'env: 'conn >( Option< &'conn Connection< 'env > > );

impl< 'conn, 'env > Transaction< 'conn, 'env > {

    pub fn commit( mut self ) -> Result< (), SQLRETURN > {
        let dbc = self.0.take().unwrap().0;

        // TODO: SQLEndTrans

        Ok( () )
    }

    pub fn rollback( mut self ) -> Result< (), SQLRETURN > {
        let dbc = self.0.take().unwrap().0;

        // TODO: SQLEndTrans

        Ok( () )
    }
}

impl< 'conn, 'env > Drop for Transaction< 'conn, 'env > {

    fn drop( &mut self ) {
        let conn = self.0.take();
        if conn.is_none() {
            return;
        }

        let dbc = conn.unwrap().0;

        // TODO: SQLEndTrans
    }
}


#[ cfg( test ) ]
mod tests {

    use super::*;
    use super::super::tests::CONN_STR;

    #[ test ]
    fn make_env() {
        Environment::new().unwrap();
    }

    #[ test ]
    fn make_conn() {
        let env = Environment::new().unwrap();
        Connection::new( &env, CONN_STR ).unwrap();
    }

    #[ test ]
    fn commit_trans() {
        let env = Environment::new().unwrap();
        let conn = Connection::new( &env, CONN_STR ).unwrap();

        let trans = conn.begin();
        trans.commit().unwrap();
    }
}
