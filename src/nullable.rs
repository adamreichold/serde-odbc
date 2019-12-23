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
use std::mem::size_of;

use odbc_sys::{SQLLEN, SQL_NULL_DATA};
use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::binder::with_indicator;

#[derive(Clone, Copy, Debug)]
pub struct Nullable<T> {
    indicator: SQLLEN,
    value: T,
}

impl<T: Serialize> Serialize for Nullable<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut serializer = serializer.serialize_struct("Nullable", 1)?;
        with_indicator(&self.indicator as *const _ as *mut _, || {
            serializer.serialize_field("value", &self.value)
        })?;
        serializer.end()
    }
}

impl<T> Nullable<T> {
    pub fn as_ref(&self) -> Option<&T> {
        match self.indicator {
            SQL_NULL_DATA => None,
            _ => Some(&self.value),
        }
    }

    pub fn as_mut(&mut self) -> Option<&mut T> {
        match self.indicator {
            SQL_NULL_DATA => None,
            _ => Some(&mut self.value),
        }
    }
}

impl<T: Default> Default for Nullable<T> {
    fn default() -> Self {
        Nullable {
            indicator: SQL_NULL_DATA,
            value: Default::default(),
        }
    }
}

impl<T: Default> From<Option<T>> for Nullable<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            None => Default::default(),
            Some(value) => Nullable {
                indicator: size_of::<T>() as SQLLEN,
                value,
            },
        }
    }
}

impl<T> Into<Option<T>> for Nullable<T> {
    fn into(self) -> Option<T> {
        match self.indicator {
            SQL_NULL_DATA => None,
            _ => Some(self.value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        col_binding::Cols,
        connection::{Connection, Environment},
        param_binding::Params,
        statement::Statement,
        tests::CONN_STR,
    };

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
}
