use std::ptr::null_mut;

use odbc_sys::{SQLLEN, SQLPOINTER};

use serde::ser::{Impossible, Serialize, SerializeStruct, SerializeTuple, Serializer};

use bind_error::{BindError, BindResult};
use bind_types::BindTypes;

pub trait BinderImpl {
    fn bind<T: BindTypes>(
        &mut self,
        value_ptr: SQLPOINTER,
        indicator_ptr: *mut SQLLEN,
    ) -> BindResult;
}

pub struct Binder<I: BinderImpl> {
    impl_: I,
    value_ptr: SQLPOINTER,
    indicator_ptr: *mut SQLLEN,
    is_nullable: bool,
}

impl<I: BinderImpl> Binder<I> {
    pub fn new(impl_: I) -> Self {
        Binder {
            impl_,
            value_ptr: null_mut(),
            indicator_ptr: null_mut(),
            is_nullable: false,
        }
    }
}

impl<'a, I: BinderImpl> Serializer for &'a mut Binder<I> {
    type Ok = ();
    type Error = BindError;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, value: bool) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_char(self, value: char) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_i8(self, value: i8) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_i16(self, value: i16) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_i32(self, _value: i32) -> BindResult {
        self.impl_.bind::<i32>(self.value_ptr, self.indicator_ptr)
    }

    fn serialize_i64(self, value: i64) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_u8(self, value: u8) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_u16(self, value: u16) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_u32(self, value: u32) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_u64(self, value: u64) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_f32(self, value: f32) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_f64(self, value: f64) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_str(self, value: &str) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_bytes(self, value: &[u8]) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_none(self) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_unit(self) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_unit_struct(self, name: &'static str) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> BindResult {
        Ok(()) // TODO
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, BindError> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, BindError> {
        Err(BindError {}) // TODO
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, BindError> {
        Err(BindError {}) // TODO
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, BindError> {
        self.is_nullable = name == "Nullable" && len == 2;

        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, BindError> {
        Err(BindError {}) // TODO
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, BindError> {
        Err(BindError {}) // TODO
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, BindError> {
        Err(BindError {}) // TODO
    }
}

impl<'a, I: BinderImpl> SerializeTuple for &'a mut Binder<I> {
    type Ok = ();
    type Error = BindError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> BindResult {
        self.value_ptr = ((value as *const T) as *mut T) as SQLPOINTER;
        value.serialize(&mut **self)
    }

    fn end(self) -> BindResult {
        Ok(())
    }
}

impl<'a, I: BinderImpl> SerializeStruct for &'a mut Binder<I> {
    type Ok = ();
    type Error = BindError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        name: &'static str,
        value: &T,
    ) -> BindResult {
        if self.is_nullable {
            match name {
                "indicator" => {
                    self.indicator_ptr = ((value as *const T) as *mut T) as *mut SQLLEN;
                    Ok(())
                }

                "value" => {
                    self.value_ptr = ((value as *const T) as *mut T) as SQLPOINTER;
                    value.serialize(&mut **self)
                }

                name => panic!("Unexpected field {} inside nullable struct.", name),
            }
        } else {
            self.value_ptr = ((value as *const T) as *mut T) as SQLPOINTER;
            value.serialize(&mut **self)
        }
    }

    fn end(self) -> BindResult {
        if self.is_nullable {
            self.indicator_ptr = null_mut();
            self.is_nullable = false;
        }

        Ok(())
    }
}
