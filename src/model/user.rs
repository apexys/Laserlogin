use rocket_contrib::json::JsonValue;
use std::error::Error;
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, PartialOrd, Ord)]
#[repr(u8)]
pub enum Usertype{
    Admin = 0,
    User
}

impl Usertype{
    pub fn from_str(s: &str) -> Usertype{
        match s {
            "Admin" => Usertype::Admin,
            "User" => Usertype::User,
            _ => unimplemented!()
        }
    }

    pub fn to_str(self) -> &'static str{
        match self{
            Usertype::Admin => "Admin",
            Usertype::User => "User"
        }
    }

    pub fn from_string(s: String) -> Usertype{
        Self::from_str(&s)
    }

    pub fn to_json_object(self) -> JsonValue{
        json!({
            "Admin": self <= Usertype::Admin,
            "User": self <= Usertype::User
        })
    }
}

#[derive(SqlObject, Debug, Serialize, Clone)]
pub struct User{
    pub id: Option<i64>,
    pub usertype: String,
    pub email: String,
    pub card_hash: String,
    pub current_project: String
}

impl User {
    pub fn new(usertype: Usertype, email: &str, card_hash: &str, active_project: &str)-> User{
        User{
            id: None,
            usertype: usertype.to_str().to_string(),
            email: String::from(email),
            card_hash: String::from(card_hash),
            current_project: String::from(active_project)
        }
    }

    pub fn verify(&self, hash: &str) -> bool{
        let usertype = Usertype::from_str(&self.usertype);
        if usertype == Usertype::Admin && self.card_hash.is_empty() {
            return true;
        }
        
        self.card_hash == hash
    }
}