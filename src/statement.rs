use std::ptr::null_mut;
use std::marker::PhantomData;
use std::default::Default;

use odbc_sys::{
    SQLINTEGER,
    SQLRETURN, SQL_SUCCESS, SQL_SUCCESS_WITH_INFO, SQL_NO_DATA,
    SQLHANDLE, SQLHSTMT,
    SQLAllocHandle, SQLFreeHandle, SQL_HANDLE_STMT,
    SQLPrepare, SQLExecute, SQLFetch, SQLFreeStmt, SQL_CLOSE,
};

use serde::ser::Serialize;

use super::connection::Connection;
use super::param_binder::bind_params;
use super::col_binder::bind_cols;


pub struct Statement< 'conn, 'env: 'conn, Params: Default + Serialize, Cols: Default + Serialize >{
    stmt: SQLHSTMT,
    conn: PhantomData< &'conn Connection< 'env > >,
    params: Box< Params >,
    cols: Box< Cols >,
}

impl< 'conn, 'env, Params: Default + Serialize, Cols: Default + Serialize > Statement< 'conn, 'env, Params, Cols > {

    pub fn new( conn: &'conn Connection< 'env >, stmt_str: &str ) -> Result< Statement< 'conn, 'env, Params, Cols >, SQLRETURN > {
        let mut stmt: SQLHANDLE = null_mut();

        let rc = unsafe { SQLAllocHandle( SQL_HANDLE_STMT, conn.handle(), &mut stmt ) };
        if rc != SQL_SUCCESS {
            return Err( rc );
        }

        let stmt = stmt as SQLHSTMT;

        let rc = unsafe { SQLPrepare( stmt, stmt_str.as_ptr(), stmt_str.len() as SQLINTEGER ) };
        if rc != SQL_SUCCESS {
            return Err( rc );
        }

        let params = Box::new( Default::default() ); 
        bind_params( stmt, &params )?;

        let cols = Box::new( Default::default() );
        bind_cols( stmt, &cols )?;
        
        Ok(
            Statement{
                stmt,
                conn: PhantomData,
                params,
                cols, 
            }
        )
    }

    pub unsafe fn handle( &self ) -> SQLHANDLE {
        self.stmt as SQLHANDLE
    }

    pub fn params( &mut self ) -> &mut Params {
        &mut self.params
    }

    pub fn cols( &self ) -> &Cols {
        &self.cols
    }

    pub fn exec( &mut self ) -> Result< (), SQLRETURN > {
        let rc = unsafe { SQLFreeStmt( self.stmt, SQL_CLOSE ) };
        if rc != SQL_SUCCESS {
            return Err( rc );
        }

        let rc = unsafe { SQLExecute( self.stmt ) };
        if rc != SQL_SUCCESS {
            return Err( rc );
        }

        Ok( () )
    }

    pub fn fetch( &mut self ) -> Result< bool, SQLRETURN > {
        let rc = unsafe { SQLFetch( self.stmt ) };
        match rc {
            SQL_SUCCESS | SQL_SUCCESS_WITH_INFO => Ok( true ),
            SQL_NO_DATA => Ok( false ),
            rc => Err( rc )
        }
    }
}

impl< 'conn, 'env, Params: Default + Serialize, Cols: Default + Serialize > Drop for Statement< 'conn, 'env, Params, Cols > {

    fn drop( &mut self ) {
        let _ = unsafe { SQLFreeHandle( SQL_HANDLE_STMT, self.handle() ) };
    }
}


#[ cfg( test ) ]
mod tests {

    use super::*;
    use super::super::connection::Environment;
    use super::super::nullable::Nullable;
    use super::super::tests::CONN_STR;

    #[ test ]
    fn exec_stmt() {
        let env = Environment::new().unwrap();
        let conn = Connection::new( &env, CONN_STR ).unwrap();

        let mut stmt: Statement< ( i32, () ), ( Nullable< i32 >, () ) > = Statement::new( &conn, "SELECT ?" ).unwrap();
        stmt.params().0 = 42;
        stmt.exec().unwrap();
        assert!( stmt.fetch().unwrap() );
        assert_eq!( Some( 42 ), stmt.cols().0.into() );
        assert!( !stmt.fetch().unwrap() );
    }
}