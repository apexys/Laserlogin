use rusqlite::Connection;
use std::error::Error;
use std::sync::{Mutex, MutexGuard};
use std::path::Path;
use once_cell::sync::OnceCell;

///Shorthand for the locked connection
pub type PersistenceConnection = MutexGuard<'static, Connection>;

///Storage for the connection to the sqlite database, initially null, but can be initialized exactly once
pub static PERSISTENCE_CELL: OnceCell<PersistenceProvider> = OnceCell::new();

///Pseudo-Type to attach static methods to (just for namespacing)
pub struct Persistence;

impl Persistence{
    ///Creates a new database in memory
    pub fn initialize_in_memory() -> Result<(), Box<dyn Error>>{
        PERSISTENCE_CELL.set(PersistenceProvider::initialize_in_memory()?).map_err(|_| "Persistence already initialized!")?;
        Ok(())
    }

    ///Opens or creates a database in a file
    pub fn initialize_from_file(file: &Path) -> Result<(), Box<dyn Error>>{
        PERSISTENCE_CELL.set(PersistenceProvider::initialize_from_file(file)?).map_err(|_| "Persistence already initialized!")?;
        Ok(())
    }

    ///Used to retrieve the connection behind a guard
    pub fn get_connection() -> Result<PersistenceConnection, Box<dyn Error>> {
        Ok(
            PERSISTENCE_CELL.get()
                .ok_or("Persistence not initialized yet!")?
                .get_connection()
        )
    }
}


///Inner implementation of the persistence provider. This just encapsulates the connection creation and separates concerns of struct access and connection management
pub struct PersistenceProvider{
    connection: Mutex<Connection>
}

impl PersistenceProvider{
    ///Creates a new database in memory
    pub fn initialize_in_memory() -> Result<PersistenceProvider, Box<dyn Error>>{
        Ok(
            PersistenceProvider{
                connection: 
                    Mutex::new(
                        Connection::open_in_memory()?
                    )
            }
        )
    }

    ///Opens or creates a database in a file
    pub fn initialize_from_file(file: &Path) -> Result<PersistenceProvider, Box<dyn Error>>{
        Ok(
            PersistenceProvider{
                connection: 
                    Mutex::new(
                        Connection::open(file)?
                    )
            }
        )
    }

    ///Acquires a lock on the database and returns the locked connection
    pub fn get_connection(&self) -> MutexGuard<Connection> {
        self.connection.lock().unwrap()
    }
}
