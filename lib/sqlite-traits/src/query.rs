use smallvec::SmallVec;
use crate::{Persistence, SqlObject};
use std::marker::PhantomData;
use rusqlite::{OptionalExtension, types::ToSql};
use std::error::Error;
use crate::DbObject;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Ordering{
    Ascending,
    Descending
}

pub struct Query<T: DbObject + SqlObject>{
    constraints: SmallVec<[(&'static str, Box<dyn ToSql>); 3]>,
    limit: Option<i64>,
    order: SmallVec<[(&'static str, Ordering);1]>,
    group: SmallVec<[&'static str; 1]>,
    phantom: PhantomData<T>
}

impl<T: DbObject + SqlObject> Query<T>{
    pub fn new() -> Query<T>{
        Query{
            constraints: SmallVec::new(),
            limit: None,
            order: SmallVec::new(),
            group: SmallVec::new(),
            phantom: PhantomData
        }
    }

    fn select_string(&self) -> String{
        let mut s = String::with_capacity(20);
        s.push_str("SELECT * FROM ");
        s.push_str(T::get_table_name());
        s
    }

    fn constraint_string(&self) -> String{
        if !self.constraints.is_empty(){
            let mut s = String::with_capacity(20);
            s.push_str(" WHERE ");
            self.constraints.iter().enumerate().for_each(|(i, c)| {
                if i > 0 {
                    s.push_str(" AND ");
                }
                s.push_str(c.0);
                s.push_str(" = ?");
            });
            s
        }else{
            String::new()
        }
    }

    fn limit_string(&self) -> String{
        if let Some(limit) = self.limit{
            let mut s = String::with_capacity(10);
            s.push_str(" LIMIT ");
            s.push_str(&limit.to_string());
            s
        }else{
            String::new()
        }
    }

    fn order_string(&self) -> String{
        if !self.order.is_empty(){
            let mut s = String::with_capacity(20);
            s.push_str(" ORDER BY ");
            self.order.iter().enumerate().for_each(|(i, (column, order))| {
                if i > 0 {
                    s.push_str(", ");
                }
                s.push_str(column);
                s.push(' ');
                if order == &Ordering::Ascending {
                    s.push_str("ASC");
                }else{
                    s.push_str("DESC");
                }
            });
            s
        }else{
            String::new()
        }
    }

    fn group_string(&self) -> String{
        if !self.group.is_empty(){
        let mut s = String::with_capacity(20);
        s.push_str(" GROUP BY ");
        self.group.iter().enumerate().for_each(|(i, group)| {
            if i > 0 {
                s.push_str(", ");
            }
            s.push_str(group);
        });
        s
        }else{
            String::new()
        }
    }

    pub fn all(self) -> Result<Vec<T>, Box<dyn Error>> where T: DbObject + SqlObject{
        //Acquire connection
        let connection = Persistence::get_connection()?;

        let mut statement = connection.prepare_cached(&[self.select_string(), self.constraint_string(), self.group_string(), self.order_string(), self.limit_string()].join(" "))?;

        let params = self.constraints.iter().map(|c| &*c.1).collect::<SmallVec<[&dyn ToSql; 3]>>();

        let rows = statement.query_map(params, |row| Ok(SqlObject::deserialize_from_row(row)))?;

        let mut result_vec = Vec::with_capacity(rows.size_hint().1.unwrap_or(1));

        for row in rows{
            match row{//Then find out if the query succeeded
                Err(e) => {return Err(e.into());}, //Sqlite error
                Ok(Err(e)) => {return Err(e);}, //Conversion error
                Ok(Ok(object)) => {result_vec.push(object);} //Success
            }
        }

        Ok(result_vec)
    }

    pub fn get(self) -> Result<Option<T>, Box<dyn Error>> where T: DbObject + SqlObject{
        //Acquire connection
        let connection = Persistence::get_connection()?;

        let mut statement = connection.prepare_cached(&[self.select_string(), self.constraint_string(), self.group_string(), self.order_string(), self.limit_string()].join(" "))?;
        let params = self.constraints.iter().map(|c| &*c.1).collect::<SmallVec<[&dyn ToSql; 3]>>();

        match statement.query_row( //Query with
            params,  //The params (only the param values)
            |row| { //And a conversion function
                Ok(SqlObject::deserialize_from_row(row))
            }
        ).optional(){//Then find out if the query succeeded
            Ok(None) => Ok(None), //No row found
            Err(e) => Err(e.into()), //Sqlite error
            Ok(Some(Err(e))) => Err(e), //Conversion error
            Ok(Some(Ok(object))) => Ok(Some(object)) //Success
        }
    }

    #[allow(non_snake_case)]
    pub fn Where<X>(mut self, identifier: &'static str, value: X) -> Self where X: ToSql + 'static{
        self.constraints.push((identifier, Box::new(value)));
        self
    }

    #[allow(non_snake_case)]
    pub fn Limit(mut self, limit: i64) -> Self{
        self.limit = Some(limit);
        self
    }

    #[allow(non_snake_case)]
    pub fn OrderBy(mut self, identifier: &'static str, order: Ordering) -> Self{
        self.order.push((identifier, order));
        self
    }

    #[allow(non_snake_case)]
    pub fn GroupBy(mut self, identifier: &'static str) -> Self{
        self.group.push(identifier);
        self
    }

}
