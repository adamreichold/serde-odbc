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
extern crate generic_array;
extern crate odbc_sys;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate typenum;

mod error;
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

pub use error::{Error, Result};
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
