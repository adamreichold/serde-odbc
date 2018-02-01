extern crate generic_array;
extern crate odbc_sys;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate typenum;

mod bind_error;
mod bind_types;
mod binder;
mod param_binder;
mod col_binder;
mod param_binding;
mod col_binding;
mod nullable;
mod string;
mod connection;
mod statement;

pub use param_binding::{NoParams, ParamSet, Params};
pub use col_binding::{Cols, NoCols, RowSet};
pub use nullable::*;
pub use string::*;
pub use connection::*;
pub use statement::*;

#[cfg(test)]
mod tests {
    pub const CONN_STR: &'static str = "Driver=Sqlite3;Database=:memory:;";
}
