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
use std::cell::Cell;
use std::ptr::null_mut;

use odbc_sys::{SQLLEN, SQLPOINTER};
use serde::ser::{Impossible, Serialize, SerializeStruct, SerializeTuple, Serializer};

use crate::bind_types::BindTypes;
use crate::error::{Error, Result};

thread_local! {
    static INDICATOR_PTR: Cell<*mut SQLLEN> = Cell::new(null_mut());
}

fn take_indicator() -> *mut SQLLEN {
    INDICATOR_PTR.with(|indicator_ptr| indicator_ptr.replace(null_mut()))
}

pub fn with_indicator<F, T>(indicator: *mut SQLLEN, f: F) -> T
where
    F: FnOnce() -> T,
{
    INDICATOR_PTR.with(|indicator_ptr| {
        indicator_ptr.set(indicator);

        struct Reset<'a>(&'a Cell<*mut SQLLEN>);

        impl Drop for Reset<'_> {
            fn drop(&mut self) {
                self.0.set(null_mut());
            }
        }

        let _reset = Reset(indicator_ptr);

        f()
    })
}

pub trait BinderImpl {
    fn bind<T: BindTypes>(
        &mut self,
        value_ptr: SQLPOINTER,
        indicator_ptr: *mut SQLLEN,
    ) -> Result<()>;

    fn bind_str(
        &mut self,
        length: usize,
        value_ptr: SQLPOINTER,
        indicator_ptr: *mut SQLLEN,
    ) -> Result<()>;
}

pub struct Binder<I: BinderImpl> {
    impl_: I,
    lower_bound: SQLPOINTER,
    upper_bound: SQLPOINTER,
    value_ptr: SQLPOINTER,
}

impl<I: BinderImpl> Binder<I> {
    pub fn bind<T: Serialize>(impl_: I, value: &T) -> Result<()> {
        let value_ptr = (value as *const T) as *mut T;

        let mut binder = Binder {
            impl_,
            lower_bound: value_ptr as SQLPOINTER,
            upper_bound: unsafe { value_ptr.add(1) } as SQLPOINTER,
            value_ptr: value_ptr as SQLPOINTER,
        };

        value.serialize(&mut binder)
    }
}

macro_rules! fn_serialize {
    ($method:ident, $type:ident) => {
        fn $method(self, _value: $type) -> Result<()> {
            assert!(self.lower_bound <= self.value_ptr);
            assert!(unsafe  { (self.value_ptr as *mut $type).add(1) as SQLPOINTER } <= self.upper_bound);

            self.impl_.bind::<$type>(self.value_ptr, take_indicator())
        }
    }
}

impl<'a, I: BinderImpl> Serializer for &'a mut Binder<I> {
    type Ok = ();
    type Error = Error;

    type SerializeTuple = Self;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;

    fn_serialize!(serialize_i8, i8);
    fn_serialize!(serialize_i16, i16);
    fn_serialize!(serialize_i32, i32);
    fn_serialize!(serialize_i64, i64);

    fn_serialize!(serialize_u8, u8);
    fn_serialize!(serialize_u16, u16);
    fn_serialize!(serialize_u32, u32);
    fn_serialize!(serialize_u64, u64);

    fn_serialize!(serialize_f32, f32);
    fn_serialize!(serialize_f64, f64);

    fn_serialize!(serialize_bool, bool);

    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        let value_ptr = (value.as_ptr() as *mut u8) as SQLPOINTER;

        assert!(self.lower_bound <= value_ptr);
        assert!(
            unsafe { (value_ptr as *mut u8).add(value.len()) } as SQLPOINTER <= self.upper_bound
        );

        self.impl_
            .bind_str(value.len(), value_ptr, take_indicator())
    }

    fn serialize_char(self, _value: char) -> Result<()> {
        unimplemented!();
    }

    fn serialize_str(self, _value: &str) -> Result<()> {
        unimplemented!();
    }

    fn serialize_none(self) -> Result<()> {
        unimplemented!();
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<()> {
        unimplemented!();
    }

    fn serialize_unit(self) -> Result<()> {
        unimplemented!();
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        unimplemented!();
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        unimplemented!();
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<()> {
        unimplemented!();
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()> {
        unimplemented!();
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        unimplemented!();
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unimplemented!();
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unimplemented!();
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        unimplemented!();
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        unimplemented!();
    }
}

impl<'a, I: BinderImpl> SerializeTuple for &'a mut Binder<I> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.value_ptr = ((value as *const T) as *mut T) as SQLPOINTER;

        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, I: BinderImpl> SerializeStruct for &'a mut Binder<I> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _name: &'static str,
        value: &T,
    ) -> Result<()> {
        self.value_ptr = ((value as *const T) as *mut T) as SQLPOINTER;

        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}
