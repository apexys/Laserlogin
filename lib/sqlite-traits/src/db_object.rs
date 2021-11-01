use rusqlite::{NO_PARAMS, ToSql, params};
use smallvec::SmallVec;
use std::error::Error;
use crate::{Persistence, Query, SqlObject};

pub trait DbObject: SqlObject{
    fn initialize() ->Result<(), Box<dyn Error>>;
    fn save(&mut self) ->Result<(), Box<dyn Error>>;
    fn delete(&mut self) ->Result<(), Box<dyn Error>>;
    fn query() -> Query<Self> where Self: DbObject + SqlObject + Sized;
}

impl<T> DbObject for T where T: SqlObject{
    fn initialize() ->Result<(), Box<dyn Error>> {
        //Create a new table
        let mut query = "CREATE TABLE IF NOT EXISTS ".to_string();
        query.push_str(T::get_table_name());
        query.push_str(" (");
        query.push_str("id INTEGER PRIMARY KEY"); //with a primary key
        for (column, typename) in T::fields_with_sqltypes(){ //and columns for each field
            query.push_str(", ");
            query.push_str(column);
            query.push(' ');
            query.push_str(typename.to_sqlite_str());
        }
        query.push(')');
        //Execute the query
        let connection = Persistence::get_connection()?;
        connection.execute(&query, NO_PARAMS)?;
        Ok(())
    }

    fn save(&mut self) ->Result<(), Box<dyn Error>> {
        if let Some(id) = self.get_id(){
            let mut query = "UPDATE ".to_string();
            query.push_str(T::get_table_name());
            query.push_str(" SET ");
            let mut values: SmallVec<[&dyn ToSql; 10]> = SmallVec::new();
            for (index, (column, value)) in self.fields_with_values().into_iter().enumerate(){
                if index > 0 {
                    query.push_str(", ");
                }
                query.push_str(column);
                query.push_str(" = ?");
                values.push(value);
            }
            query.push_str(" WHERE id = ?");
            values.push(&id);
            let connection = Persistence::get_connection()?;
            connection.execute(&query, values)?;
        }else{
            let mut query = "INSERT INTO ".to_string();
            query.push_str(T::get_table_name());
            query.push_str(" DEFAULT VALUES");
            let connection = Persistence::get_connection()?;
            connection.execute(&query, NO_PARAMS)?;
            self.set_id(Some(connection.last_insert_rowid()));
            drop(connection);
            self.save()?;
        }
        Ok(())
    }

    fn delete(&mut self) ->Result<(), Box<dyn Error>> {
        if let Some(id) = self.get_id(){ //No id == not in database
            let mut query = "DELETE FROM ".to_string();
            query.push_str(T::get_table_name());
            query.push_str(" WHERE id = ?");
            let connection = Persistence::get_connection()?;
            connection.execute(&query, params![id])?;
            self.set_id(None);
        }
        Ok(())
    }

    fn query() -> Query<Self> where Self: DbObject + Sized {
        Query::new()
    }
}
