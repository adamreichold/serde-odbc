extern crate odbc_sys;
extern crate serde;
#[ macro_use ]
extern crate serde_derive;

mod connection;
mod statement;
mod nullable;
mod bind;
mod param_binder;
mod col_binder;

pub use connection::*;
pub use statement::*;
pub use nullable::*;


#[ cfg( test ) ]
mod tests {

    pub const CONN_STR: &'static str = "Driver=Sqlite3;Database=:memory:;";
}