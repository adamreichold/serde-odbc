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
use std::mem::MaybeUninit;

use generic_array::{ArrayLength, GenericArray};
use odbc_sys::SQLLEN;
use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::binder::with_indicator;

#[derive(Clone)]
struct ByteArray<N: ArrayLength<u8>>(GenericArray<u8, N>);

impl<N: Clone + ArrayLength<u8>> Copy for ByteArray<N> where N::ArrayType: Copy {}

impl<N: ArrayLength<u8>> Serialize for ByteArray<N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0.as_slice())
    }
}

#[derive(Clone)]
pub struct String<N: ArrayLength<u8>> {
    indicator: SQLLEN,
    value: ByteArray<N>,
}

impl<N: Clone + ArrayLength<u8>> Copy for String<N> where N::ArrayType: Copy {}

impl<N: ArrayLength<u8>> Serialize for String<N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut serializer = serializer.serialize_struct("String", 1)?;
        with_indicator(&self.indicator as *const _ as *mut _, || {
            serializer.serialize_field("value", &self.value)
        })?;
        serializer.end()
    }
}

impl<N: ArrayLength<u8>> String<N> {
    pub fn clear(&mut self) {
        self.indicator = 0;
    }

    pub fn extend_from_slice(&mut self, value: &[u8]) {
        let len = min(N::to_usize() - self.indicator as usize, value.len());

        self.value.0.as_mut_slice()[self.indicator as usize..][..len]
            .copy_from_slice(&value[..len]);

        self.indicator += len as SQLLEN;
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.value.0.as_slice()[..self.indicator as usize]
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.value.0.as_mut_slice()[..self.indicator as usize]
    }
}

impl<N: ArrayLength<u8>> Default for String<N> {
    fn default() -> Self {
        Self {
            indicator: 0,
            value: unsafe {
                #[allow(clippy::uninit_assumed_init)]
                MaybeUninit::uninit().assume_init()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use generic_array::typenum::U8;

    use crate::{
        col_binding::Cols,
        connection::{Connection, Environment},
        param_binding::Params,
        statement::Statement,
        tests::CONN_STR,
    };

    #[test]
    fn default_str() {
        let value: String<U8> = Default::default();
        assert_eq!(&b""[..], value.as_slice());
    }

    #[test]
    fn make_str() {
        let mut value: String<U8> = Default::default();
        value.extend_from_slice(&b"foobar"[..]);
        assert_eq!(&b"foobar"[..], value.as_slice());
    }

    #[test]
    fn bind_str() {
        let env = Environment::new().unwrap();
        let conn = Connection::new(&env, CONN_STR).unwrap();

        let mut stmt: Statement<Params<String<U8>>, Cols<String<U8>>> =
            Statement::new(&conn, "SELECT ?").unwrap();
        stmt.params().extend_from_slice(b"foobarfoobar");
        stmt.exec().unwrap();
        assert!(stmt.fetch().unwrap());
        assert_eq!(&b"foobarfo"[..], stmt.cols().as_slice());
        assert!(!stmt.fetch().unwrap());
    }
}
