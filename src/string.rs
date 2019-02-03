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
use std::cmp::min;
use std::default::Default;
use std::mem::uninitialized;
use std::ptr::copy_nonoverlapping;

use odbc_sys::{SQLLEN, SQL_NULL_DATA};

use serde::ser::{Serialize, SerializeStruct, Serializer};

use generic_array::{ArrayLength, GenericArray};

#[derive(Clone)]
struct ByteArray<N: ArrayLength<u8>>(GenericArray<u8, N>);

impl<N: ArrayLength<u8>> Serialize for ByteArray<N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0.as_slice())
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct String<N: ArrayLength<u8>> {
    indicator: SQLLEN,
    value: ByteArray<N>,
    null_terminator: u8,
}

impl<N: ArrayLength<u8>> Serialize for String<N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut serializer = serializer.serialize_struct("String", 2)?;
        serializer.serialize_field("indicator", &self.indicator)?;
        serializer.serialize_field("value", &self.value)?;
        serializer.end()
    }
}

impl<N: ArrayLength<u8>> String<N> {
    pub fn assign<'a>(&mut self, value: &'a [u8]) {
        self.indicator = min(N::to_usize(), value.len()) as SQLLEN;

        unsafe {
            copy_nonoverlapping(
                value.as_ptr(),
                (&mut self.value as *mut ByteArray<N>) as *mut u8,
                self.indicator as usize,
            );
        }
    }

    pub fn as_slice(&self) -> Option<&[u8]> {
        match self.indicator {
            SQL_NULL_DATA => None,
            indicator => Some(&self.value.0.as_slice()[..indicator as usize]),
        }
    }

    pub fn as_mut_slice(&mut self) -> Option<&mut [u8]> {
        match self.indicator {
            SQL_NULL_DATA => None,
            indicator => Some(&mut self.value.0.as_mut_slice()[..indicator as usize]),
        }
    }
}

impl<N: ArrayLength<u8>> Default for String<N> {
    fn default() -> Self {
        String {
            indicator: SQL_NULL_DATA,
            value: unsafe { uninitialized() },
            null_terminator: 0,
        }
    }
}

impl<'a, N: ArrayLength<u8>> From<&'a [u8]> for String<N> {
    fn from(value: &'a [u8]) -> Self {
        let mut result: Self = Default::default();
        result.assign(value);
        result
    }
}

impl<'a, N: ArrayLength<u8>> Into<Option<&'a [u8]>> for &'a String<N> {
    fn into(self) -> Option<&'a [u8]> {
        self.as_slice()
    }
}

impl<'a, N: ArrayLength<u8>> Into<Option<&'a mut [u8]>> for &'a mut String<N> {
    fn into(self) -> Option<&'a mut [u8]> {
        self.as_mut_slice()
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
    use typenum::U8;

    #[test]
    fn default_str() {
        let value: String<U8> = Default::default();
        assert_eq!(None, value.as_slice());
    }

    #[test]
    fn make_str() {
        let value: String<U8> = (&b"foobar"[..]).into();
        assert_eq!(Some(&b"foobar"[..]), value.as_slice());
    }

    #[test]
    fn bind_str() {
        let env = Environment::new().unwrap();
        let conn = Connection::new(&env, CONN_STR).unwrap();

        let mut stmt: Statement<Params<String<U8>>, Cols<String<U8>>> =
            Statement::new(&conn, "SELECT ?").unwrap();
        stmt.params().assign(b"foobarfoobar");
        stmt.exec().unwrap();
        assert!(stmt.fetch().unwrap());
        assert_eq!(Some(&b"foobarfo"[..]), stmt.cols().as_slice());
        assert!(!stmt.fetch().unwrap());
    }
}
