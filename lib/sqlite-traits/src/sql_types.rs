
///Standard datatypes for sqlite columns
pub enum SqlType{
    Integer,
    Real,
    Blob,
    Text,
    Foreign,
    Unknown
}

impl SqlType{
    ///This is kinda unused right now, but might be interesting for automatically creating bindings to sqlite tables
    pub fn from_str(typename: &str) -> SqlType{
        match typename.to_lowercase().as_ref(){
            "integer" => SqlType::Integer,
            "real" => SqlType::Real,
            "blob" => SqlType::Blob,
            "text" => SqlType::Text,
            "foreign" => SqlType::Foreign,
            _=> SqlType::Unknown
        }
    }

    ///Returns a type-string from an SqlType
    pub fn to_str(&self ) -> &'static str{
        match self{
             SqlType::Integer => "integer",
             SqlType::Real => "real",
             SqlType::Blob => "blob",
             SqlType::Text => "text",
             SqlType::Foreign => "foreign",
             SqlType::Unknown => "unknown"
        }
    }

    ///Returns a type-string from an SqlType that we can directly use in sqlite (except for foreign, list and unknown)
    pub fn to_sqlite_str(&self) -> &'static str{
        match self{
            SqlType::Integer => "integer",
            SqlType::Real => "real",
            SqlType::Blob => "blob",
            SqlType::Text => "text",
            SqlType::Foreign => "integer",
            SqlType::Unknown => "unknown"
       }
    }
}

///Trait that returns a fitting sql type for a rust type. Use foreign for structs or the like, 
pub trait SqlTyped{
    fn get_sql_type() -> SqlType{   SqlType::Unknown}
}

///Default datatype implementations

macro_rules! impl_sql_typed {
    {$sqlType:expr; $($rustType:ty),+} => {
        $(
            impl SqlTyped for $rustType {
                fn get_sql_type() -> SqlType{$sqlType}
            }
        )*
    }
}

impl_sql_typed!(SqlType::Integer; i8, i16, i32, i64, u8, u16, u32, u64);

impl_sql_typed!(SqlType::Real; f32, f64);

impl_sql_typed!(SqlType::Text; String, &'_ str);

impl_sql_typed!(SqlType::Blob; Vec<u8>, &[u8]);
