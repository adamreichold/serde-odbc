extern crate odbc_sys;
extern crate serde;

mod connection;
mod statement;
mod bind;
mod param_binder;
mod col_binder;

pub use connection::*;
pub use statement::*;


#[ cfg( test ) ]
mod tests {

    pub const CONN_STR: &'static str = "Driver=Sqlite3;Database=:memory:;";
}