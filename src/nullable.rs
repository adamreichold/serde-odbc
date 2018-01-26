use std::mem::{ uninitialized, size_of };
use std::default::Default;

use odbc_sys::{ SQLLEN, SQL_NULL_DATA };

#[ derive( Clone, Copy, Debug, Serialize ) ]
pub struct Nullable< T > {
    indicator: SQLLEN,
    value: T,
}

impl< T > Default for Nullable< T > {

    fn default() -> Self {
        Nullable{
            indicator: SQL_NULL_DATA,
            value: unsafe { uninitialized() },
        }
    }
}

impl< T > From< Option< T > > for Nullable< T > {

    fn from( value: Option< T > ) -> Self {
        match value {
            
            None => Default::default(),

            Some( value ) => Nullable{
                indicator: size_of::< T >() as SQLLEN,
                value: value,
            }

        }
    }
}

impl< T > Into< Option< T > > for Nullable< T > {

    fn into( self ) -> Option< T > {
        match self.indicator {

            SQL_NULL_DATA => None,
            
            _ => Some( self.value ),
        
        }
    }
}