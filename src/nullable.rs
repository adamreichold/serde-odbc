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
use std::default::Default;
use std::mem::{size_of, uninitialized};

use odbc_sys::{SQLLEN, SQL_NULL_DATA};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Nullable<T> {
    indicator: SQLLEN,
    value: T,
}

impl<T> Nullable<T> {
    pub fn assign(&mut self, value: T) {
        self.indicator = size_of::<T>() as SQLLEN;
        self.value = value;
    }

    pub fn get(&self) -> Option<&T> {
        match self.indicator {
            SQL_NULL_DATA => None,
            _ => Some(&self.value),
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        match self.indicator {
            SQL_NULL_DATA => None,
            _ => Some(&mut self.value),
        }
    }
}

impl<T> Default for Nullable<T> {
    fn default() -> Self {
        Nullable {
            indicator: SQL_NULL_DATA,
            value: unsafe { uninitialized() },
        }
    }
}

impl<T> From<Option<T>> for Nullable<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            None => Default::default(),
            Some(value) => Nullable {
                indicator: size_of::<T>() as SQLLEN,
                value: value,
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
    use super::super::col_binding::Cols;
    use super::super::connection::{Connection, Environment};
    use super::super::param_binding::Params;
    use super::super::statement::Statement;
    use super::super::tests::CONN_STR;
    use super::*;

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
