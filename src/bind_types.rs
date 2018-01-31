use odbc_sys::{SqlCDataType, SqlDataType};

pub trait BindTypes {
    fn c_data_type() -> SqlCDataType;
    fn data_type() -> SqlDataType;
}

impl BindTypes for i32 {
    fn c_data_type() -> SqlCDataType {
        SqlCDataType::SQL_C_SLONG
    }
    fn data_type() -> SqlDataType {
        SqlDataType::SQL_INTEGER
    }
}
