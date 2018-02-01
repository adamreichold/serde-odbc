use std::ptr::null_mut;

use odbc_sys::{SQLLEN, SQLPOINTER};

use serde::ser::{Impossible, Serialize, SerializeStruct, SerializeTuple, Serializer};

use error::{Error, Result};
use bind_types::BindTypes;

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
    value_ptr: SQLPOINTER,
    indicator_ptr: *mut SQLLEN,
    set_indicator: bool,
}

impl<I: BinderImpl> Binder<I> {
    pub fn bind<T: Serialize>(impl_: I, value: &T) -> Result<()> {
        let mut binder = Binder {
            impl_,
            value_ptr: ((value as *const T) as *mut T) as SQLPOINTER,
            indicator_ptr: null_mut(),
            set_indicator: false,
        };

        value.serialize(&mut binder)
    }
}

macro_rules! fn_serialize {
    ($method:ident, $type:ident) => {
        fn $method(self, _value: $type) -> Result<()> {
            self.impl_.bind::<$type>(self.value_ptr, self.indicator_ptr)
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
        self.impl_.bind_str(
            value.len(),
            (value.as_ptr() as *mut u8) as SQLPOINTER,
            self.indicator_ptr,
        )
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
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        Ok(())
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

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.set_indicator = (name == "Nullable" || name == "String") && len == 2;

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
        name: &'static str,
        value: &T,
    ) -> Result<()> {
        if self.set_indicator && name == "indicator" {
            self.indicator_ptr = ((value as *const T) as *mut T) as *mut SQLLEN;
            return Ok(());
        }

        self.value_ptr = ((value as *const T) as *mut T) as SQLPOINTER;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        if self.set_indicator {
            self.indicator_ptr = null_mut();
            self.set_indicator = false;
        }

        Ok(())
    }
}
