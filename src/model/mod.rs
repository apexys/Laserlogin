use std::collections::HashMap;
use std::path::Path;
use std::error::Error;
use std::sync::Mutex;
use simple_error;

pub use sqlite_traits::dbobject;
use sqlite_traits::Persistance as SqlitePersistance;
use sqlite_traits::dbobject::{DbObject, DbConnection};

pub mod user;
pub mod log;

pub mod config;

use self::user::{User, Usertype};
use self::config::Config;

use uuid::Uuid;
use rand::Rng;

pub fn initialize(c: &DbConnection){
    user::User::initialize(c).unwrap();
    log::Log::initialize(c).unwrap();
    config::Config::initialize(c).unwrap();
}

pub struct Persistance{
    p: SqlitePersistance,
    tokens: Mutex<HashMap<String, (i64, Usertype)>>,
}

impl Persistance{
    pub fn new() -> Self{
        let dbPath = Path::new("db.db");
        let needs_first_user = !dbPath.exists();


        let p =Persistance{
            p: SqlitePersistance::new(dbPath),
            tokens: Mutex::new(HashMap::new()),
        };

        let conn = p.get_conn().unwrap();

        initialize(&conn);

        if needs_first_user{
            if Path::new("first_user").exists(){
                let email = std::fs::read_to_string("first_user").unwrap().as_str().trim().to_string();
                let mut u =User::new(Usertype::Admin, &email, "", "");
                User::create(&conn, &mut u).unwrap();
            }
        }

        //TODO: finish implementing salts, move to salts on per-user-basis
        if Config::query(p.get_conn().unwrap()).Where(Config::fields().name, "salt").get().is_none(){
            Config::create(&conn, &mut Config::new("salt", &rand::thread_rng().sample_iter(&rand::distributions::Alphanumeric).take(64).collect::<String>())).unwrap();
        }

        p        
    }

    pub fn get_conn(&self) ->  Result<DbConnection, Box<Error>>{
        self.p.get_conn()
    }

    pub fn get_user_role(&self, token: &str) -> Option<Usertype>{
        Some(self.tokens.lock().ok()?.get(token)?.1)
    }

    pub fn get_user_id(&self, token: &str) -> Option<i64>{
        Some(self.tokens.lock().ok()?.get(token)?.0)
    }

    pub fn log_in(&self, email: &str, password: &str)-> Result<String, Box<Error>>{
        match User::query(self.get_conn()?).Where(User::fields().email,String::from(email)).get() {
            None => bail!("Username not found"),
            Some(u) => {
                if u.verify(password){
                    let token = Uuid::new_v4().hyphenated().to_string();
                    self.tokens.lock().map_err(|_| simple_error::SimpleError::new("Lock broken"))?.insert(String::from(token.as_str()), (u.id, u.usertype));
                    Ok(token)
                }else{
                    bail!("Password incorrect")
                }
            }
        } 
    }

    pub fn log_out(&self, token: &str)-> Result<(), Box<Error>>{
        match self.tokens.lock().map_err(|_| simple_error::SimpleError::new("Lock broken"))?.remove(token){
            Some(_) => Ok(()),
            None => bail!{"Error aquiring lock"}
        }
    }

}