extern crate r2d2;
extern crate rusqlite;
mod pool;
pub mod dbobject;
use self::pool::SqliteConnectionManager;
use std::error::Error;
use std::path::Path;

pub use dbobject::Ordering;

pub type DbConnection = r2d2::PooledConnection<SqliteConnectionManager>;
pub type ToSql = rusqlite::types::ToSql;

pub struct Persistance{
    connection_pool: r2d2::Pool<SqliteConnectionManager>,
}

impl Persistance{
    pub fn new(db_path: &Path) -> Self{
        Persistance{
            connection_pool: r2d2::Pool::builder().build(SqliteConnectionManager::file(db_path)).unwrap(),
        }      
    }

    pub fn get_conn(&self) ->  Result<DbConnection, Box<Error>>{
        Ok(self.connection_pool.get()?)
    }
}