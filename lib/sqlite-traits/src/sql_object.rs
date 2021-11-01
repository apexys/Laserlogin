use std::error::Error;
use rusqlite::{Row, ToSql};
use crate::SqlType;

///Internal trait for database interaction
pub trait SqlObject{

    ///Set the id on the object or clear it (no id == not in the database, for example when deleting)
    fn set_id(&mut self, id: Option<i64>);

    ///Get the id of the object or None if it is not in the DB
    fn get_id(&self) -> Option<i64>;

    ///The name of the table that this object should be inserted into
    fn get_table_name() -> &'static str;

    ///A list of all field names and their respective SQL column types
    fn fields_with_sqltypes() -> Vec<(&'static str, SqlType)>;

    ///A list of all field names and their values
    fn fields_with_values<'a>(&'a self) -> Vec<(&'static str, &'a dyn ToSql)>;

    ///Create this object from a row
    fn deserialize_from_row(row: &Row) -> Result<Self, Box<dyn Error>> where Self: Sized;
}