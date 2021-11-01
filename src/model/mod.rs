use std::collections::HashMap;
use std::path::Path;
use std::error::Error;
use std::sync::Mutex;
use simple_error;

pub use sqlite_traits::DbObject;

pub mod user;
pub mod log;

pub mod config;

use self::user::{User, Usertype};
use self::log::Log;
use self::config::Config;

use uuid::Uuid;
use rand::Rng;


pub fn initialize(){
    user::User::initialize().unwrap();
    log::Log::initialize().unwrap();
    config::Config::initialize().unwrap();
}

pub struct Persistance{
    tokens: Mutex<HashMap<String, (i64, Usertype)>>,
    current_user: Mutex<Option<User>>,
    current_log: Mutex<Option<Log>>,
    last_hash: Mutex<String>
}

impl Persistance{
    pub fn new() -> Self{
        let dbPath = Path::new("db.db");
        let needs_first_user = !dbPath.exists();


        let p =Persistance{
            tokens: Mutex::new(HashMap::new()),
            current_user: Mutex::new(None),
            current_log: Mutex::new(None),
            last_hash: Mutex::new(String::new())
        };

        let db_path = "db.db";
        sqlite_traits::Persistence::initialize_from_file(&dbPath).unwrap();

        initialize();

        if needs_first_user && Path::new("first_user").exists() {
            let email = std::fs::read_to_string("first_user").unwrap().as_str().trim().to_string();
            let mut u =User::new(Usertype::Admin, &email, "", "");
            u.save();
        }

        //TODO: finish implementing salts, move to salts on per-user-basis
        if Config::query().Where("name", "salt").get().ok().flatten().is_none(){
            let mut config = Config::new("salt", &rand::thread_rng().sample_iter(&rand::distributions::Alphanumeric).take(64).map(|v| v.to_string()).collect::<String>());
            config.save();
        }

        p        
    }

    pub fn get_current_log(&self) -> Result<Option<Log>, Box<dyn Error>>{
        let log = self.current_log.lock().map_err(|_| simple_error::SimpleError::new("Lock broken"))?;
        Ok((*log).clone())
    }

    pub fn set_current_log(&self, new_log: Option<Log>) -> Result<(), Box<dyn Error>>{
        let mut l = self.current_log.lock().map_err(|_| simple_error::SimpleError::new("Lock broken"))?;
        *l = new_log.clone();
        Ok(())
    }

    pub fn get_current_user(&self) -> Result<Option<User>, Box<dyn Error>>{
        let user = self.current_user.lock().map_err(|_| simple_error::SimpleError::new("Lock broken"))?;
        Ok((*user).clone())
    }

    pub fn set_current_user(&self, new_user: Option<User>) -> Result<(), Box<dyn Error>>{
        let mut u = self.current_user.lock().map_err(|_| simple_error::SimpleError::new("Lock broken"))?;
        *u = new_user;
        Ok(())
    }

    pub fn get_last_hash(&self) -> Result<String, Box<dyn Error>>{
        let hash = self.last_hash.lock().map_err(|_| simple_error::SimpleError::new("Lock broken"))?;
        Ok((*hash).clone())
    }

    pub fn set_last_hash(&self, new_hash: String) -> Result<(), Box<dyn Error>>{
        let mut u = self.last_hash.lock().map_err(|_| simple_error::SimpleError::new("Lock broken"))?;
        *u = new_hash;
        Ok(())
    }

    pub fn get_user_role(&self, token: &str) -> Option<Usertype>{
        Some(self.tokens.lock().ok()?.get(token)?.1)
    }

    pub fn get_user_id(&self, token: &str) -> Option<i64>{
        Some(self.tokens.lock().ok()?.get(token)?.0)
    }

    pub fn log_in(&self, email: &str, last_hash: &str)-> Result<String, Box<dyn Error>>{
        match dbg!(User::query().Where("email" ,String::from(email)).get())? {
            None => bail!("Username not found"),
            Some(u) => {
                if u.verify(last_hash){
                    let token = Uuid::new_v4().hyphenated().to_string();
                    self.tokens.lock().map_err(|_| simple_error::SimpleError::new("Lock broken"))?.insert(String::from(token.as_str()), (u.id.unwrap_or(-1), Usertype::from_str(&u.usertype)));
                    Ok(token)
                }else{
                    bail!("Password incorrect")
                }
            }
        } 
    }

    pub fn log_out(&self, token: &str)-> Result<(), Box<dyn Error>>{
        match self.tokens.lock().map_err(|_| simple_error::SimpleError::new("Lock broken"))?.remove(token){
            Some(_) => Ok(()),
            None => bail!{"Error aquiring lock"}
        }
    }

}