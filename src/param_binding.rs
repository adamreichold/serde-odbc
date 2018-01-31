use std::ptr::null;
use std::mem::size_of;
use std::default::Default;

use odbc_sys::{SQLSetStmtAttr, SQLHSTMT, SQLPOINTER, SQL_ATTR_PARAMSET_SIZE,
               SQL_ATTR_PARAM_BIND_TYPE, SQL_SUCCESS};

use serde::ser::Serialize;

use super::bind_error::{BindError, BindResult};
use super::param_binder::bind_params;

pub trait ParamBinding {
    fn new() -> Self;

    type Params;
    fn params(&mut self) -> &mut Self::Params;

    unsafe fn bind(&mut self, stmt: SQLHSTMT) -> BindResult;
}

pub struct Params<P: Default + Serialize> {
    data: P,
    last_data: *const P,
}

pub type NoParams = Params<()>;

pub struct ParamSet<P: Serialize> {
    data: Vec<P>,
    last_data: *const P,
    last_size: usize,
}

impl<P: Default + Serialize> ParamBinding for Params<P> {
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

    unsafe fn bind(&mut self, stmt: SQLHSTMT) -> BindResult {
        let data = &self.data as *const P;

        if self.last_data != data {
            bind_params(stmt, &*data)?;
            self.last_data = data;
        }

        Ok(())
    }
}

impl<P: Serialize> ParamSet<P> {
    unsafe fn bind_param_set(stmt: SQLHSTMT, size: usize) -> BindResult {
        let rc = SQLSetStmtAttr(
            stmt,
            SQL_ATTR_PARAM_BIND_TYPE,
            size_of::<P>() as SQLPOINTER,
            0,
        );
        if rc != SQL_SUCCESS {
            return Err(BindError {}); // TODO
        }

        let rc = SQLSetStmtAttr(stmt, SQL_ATTR_PARAMSET_SIZE, size as SQLPOINTER, 0);
        if rc != SQL_SUCCESS {
            return Err(BindError {}); // TODO
        }

        Ok(())
    }
}

impl<P: Serialize> ParamBinding for ParamSet<P> {
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

    unsafe fn bind(&mut self, stmt: SQLHSTMT) -> BindResult {
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
