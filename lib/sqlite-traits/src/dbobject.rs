use rusqlite::types::ToSql;
pub use rusqlite::Row;
use std::marker::PhantomData;
use std::marker::Sized;
use std::error::Error;
use std::collections::HashMap;

pub use DbConnection;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Ordering{
    Ascending,
    Descending
}

pub struct Query<T: DbObject>{
    pub connection: DbConnection,
    pub table_name: &'static str,
    constraints: Vec<(&'static str, Box<ToSql>)>,
    limit: Option<i64>,
    order: Vec<(&'static str, Ordering)>, 
    group: Vec<&'static str>,
    phantom: PhantomData<T>
}

impl<T: DbObject> Query<T>{
    pub fn new(conn: DbConnection, table_name: &'static str) -> Query<T>{
        Query{
            connection: conn,
            table_name: table_name,
            constraints: Vec::new(),
            limit: None,
            order: Vec::new(),
            group: Vec::new(),
            phantom: PhantomData
        }
    }

    fn select_string(&self) -> String{
        let mut  s = String::with_capacity(20);
        s.push_str("SELECT * FROM ");
        s.push_str(self.table_name);
        s
    }

    fn constraint_string(&self) -> String{
        if self.constraints.len() > 0{
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
        if self.order.len() > 0{
        let mut s = String::with_capacity(20);
        s.push_str(" ORDER BY ");
        self.order.iter().enumerate().for_each(|(i, (column, order))| {
            if i > 0 {
                s.push_str(", ");
            }
            s.push_str(column);
            s.push_str(" ");
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
        if self.group.len() > 0{
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

    pub fn all(self) -> Result<Vec<T>, Box<Error>> where T: DbObject{
        let mut stmt = self.connection.prepare(&[self.select_string(), self.constraint_string(), self.group_string(), self.order_string(), self.limit_string()].join(" "))?;

        let mut v: Vec<&ToSql> = Vec::with_capacity(self.constraints.len());
        self.constraints.iter().for_each(|c| v.push(&*c.1));

        let mut rows = stmt.query(&v)?;

        let mut res: Vec<T> = Vec::new();

        while let Some(row) = rows.next() {
            res.push(DbObject::from_row(&row?)?);
        }
        
        return Ok(res);
    }

    pub fn map_all(self) -> Result<HashMap<i64, T>, Box<Error>> where T: DbObject{
        let mut stmt = self.connection.prepare(&[self.select_string(), self.constraint_string(), self.group_string(), self.order_string(), self.limit_string()].join(" "))?;

        let mut v: Vec<&ToSql> = Vec::with_capacity(self.constraints.len());
        self.constraints.iter().for_each(|c| v.push(&*c.1));

        let mut rows = stmt.query(&v)?;

        let mut res: HashMap<i64, T> = HashMap::new();

        while let Some(row) = rows.next() {
            let item: T = DbObject::from_row(&row?)?;
            res.insert(item.id(), item);
        }
        
        return Ok(res);
    }

    pub fn get(self) -> Option<T> where T: DbObject{
        let mut stmt = self.connection.prepare(&[self.select_string(), self.constraint_string(), self.group_string(), self.order_string(), self.limit_string()].join(" ")).ok()?;

        let mut v: Vec<&ToSql> = Vec::with_capacity(self.constraints.len());
        self.constraints.iter().for_each(|c| v.push(&*c.1));

        let mut rows = stmt.query(&v).ok()?;

        if let Some(row) = rows.next() {
            Some(DbObject::from_row(&row.ok()?).ok()?)
        }else{
            None
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

pub trait DbObject<RHS = Self> {
    fn id(&self) -> i64;
    fn initialize(conn: &DbConnection) ->Result<(), Box<Error>>;
    fn create(conn: &DbConnection, o: &mut Self) ->Result<(), Box<Error>>;
    fn update(conn: &DbConnection, o: &Self) ->Result<(), Box<Error>>;
    fn delete(conn: &DbConnection, o: &Self) ->Result<(), Box<Error>>;
    fn query(conn: DbConnection) -> Query<Self> where Self: DbObject + Sized;
    fn from_row(r: &Row) -> Result<Self, Box<Error>> where Self: DbObject + Sized;
}


