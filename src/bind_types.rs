use odbc_sys::{SqlCDataType, SqlDataType};

pub trait BindTypes {
    fn c_data_type() -> SqlCDataType;
    fn data_type() -> SqlDataType;
}

macro_rules! impl_bind_types {
    ($type:ident, $c_data_type:ident, $data_type:ident) => {
        impl BindTypes for $type {
            fn c_data_type() -> SqlCDataType {
                SqlCDataType::$c_data_type
            }
            fn data_type() -> SqlDataType {
                SqlDataType::$data_type
            }
        }
    }
}

impl_bind_types!(i8, SQL_C_STINYINT, SQL_EXT_TINYINT);
impl_bind_types!(i16, SQL_C_SSHORT, SQL_SMALLINT);
impl_bind_types!(i32, SQL_C_SLONG, SQL_INTEGER);
impl_bind_types!(i64, SQL_C_SBIGINT, SQL_EXT_BIGINT);

impl_bind_types!(u8, SQL_C_UTINYINT, SQL_EXT_TINYINT);
impl_bind_types!(u16, SQL_C_USHORT, SQL_SMALLINT);
impl_bind_types!(u32, SQL_C_ULONG, SQL_INTEGER);
impl_bind_types!(u64, SQL_C_UBIGINT, SQL_EXT_BIGINT);

impl_bind_types!(f32, SQL_C_FLOAT, SQL_FLOAT);
impl_bind_types!(f64, SQL_C_DOUBLE, SQL_DOUBLE);

impl_bind_types!(bool, SQL_C_BIT, SQL_EXT_TINYINT);
