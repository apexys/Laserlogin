#![feature(proc_macro_hygiene, decl_macro)]

extern crate rand;
#[macro_use] extern crate rocket;
extern crate rocket_contrib;
use rocket_contrib::templates::Template;


#[macro_use] extern crate serde_derive;

#[macro_use] extern crate simple_error;
use simple_error::SimpleError;

extern crate sqlite_traits;
#[macro_use] extern crate sqlite_derive;

extern crate serde;
#[macro_use] extern crate serde_json;
use serde_json::Value;


extern crate reqwest;

extern crate uuid;

mod model;

mod login_users;
use login_users::{AdminUser, NormalUser};

use rocket::response::NamedFile;
use std::path::Path;
use std::path::PathBuf;
use rocket::response::Redirect;
use rocket::request::Form;
use rocket_contrib::json::Json;
use rocket::http::{Cookie, Cookies};
use rocket::State;

extern crate chrono;
use chrono::prelude::*;

use model::Persistance;
use model::user::{User, Usertype};
use model::log::Log;
use model::dbobject::DbObject;

use std::error::Error;

#[get("/", rank = 2)]
fn login_form() -> Option<NamedFile> {
    NamedFile::open(Path::new("templates/login.html")).ok()
}

#[get("/favicon.ico")]
fn favicon() -> Option<NamedFile>{
    NamedFile::open(Path::new("static/favicon.ico")).ok()
}

#[get("/")]
fn overview(l: Usertype, persistance: State<Persistance>) ->  Result<Template, Box<Error>> {
    Ok(Template::render("overview", json!({
        "Usertype": l.to_json_object(),
    })))
}

#[get("/static/<file..>")]
fn files(file: PathBuf) -> Result<NamedFile, String>  {
    let path = Path::new("static/").join(file);
    NamedFile::open(&path).map_err(|_| format!("Bad path: /static/{:?}", path))
}

#[get("/clear_message")]
fn clear_message(mut cookies: Cookies) -> &'static str{
        cookies.remove(Cookie::named("message"));
        "message cleared"
}

macro_rules! set_message_and_redirect {
    ($message:path, $path:expr, $cookies:expr) => {{
        let mut c = Cookie::new("message",$message);
        c.set_path("/");
        $cookies.add(c);
        Ok(Redirect::to($path.to_string()))
    }};
}

#[derive(FromForm)]
struct LoginCredentials{
    username: String,
    password: String
}

#[post("/login", data="<creds>")]
fn login(creds: Form<LoginCredentials>, mut cookies: Cookies, persistance: State<Persistance>) -> Result<Redirect, Box<Error>>{
    if let Ok(token) = persistance.log_in(&creds.username, &creds.password){
        cookies.add_private(Cookie::new("token", token));
    }
    let mut redirect_str = String::from("/");
    if let Some(redirect_to) = cookies.get("redirect_to"){
        redirect_str = String::from(redirect_to.value());
        println!{"Redirect cookie: {}", &redirect_str};
    }
    cookies.remove(Cookie::named("redirect_to"));
    Ok(Redirect::to(redirect_str))
}


#[get("/logout")]
fn logout(_l: Usertype, mut cookies: Cookies, persistance: State<Persistance>) -> Result<Redirect, Box<Error>>{
    if let Some(cookie) = cookies.get_private("token"){
        persistance.log_out(cookie.value())?;
        cookies.remove_private(cookie);
    }
    Ok(Redirect::to("/"))
}


#[get("/usersettings")]
fn usersettings(_admin: AdminUser,u: Usertype, persistance: State<Persistance>) -> Result<Template, Box<Error>>{
    let users = User::query(persistance.get_conn()?).all()?.iter().map(|u| json!({
        "id": u.id, 
        "email": u.email, 
        "card_hash": u.card_hash,
        "Usertype": u.usertype.to_json_object()
    })).collect::<Value>();
    Ok(Template::render("usersettings", json!({
        "Usertype": u.to_json_object(),
        "users": users
    })))
}

#[derive(FromForm)]
struct UserCreation{
    usertype: String,
    email: String,
    card_hash: String
}

#[post("/admin_create_user", data="<data>")]
fn admin_create_user(_admin: AdminUser, data: Form<UserCreation>, persistance: State<Persistance>, mut cookies: Cookies) -> Result<Redirect, Box<Error>>{
    let users = User::query(persistance.get_conn()?).all()?;
    if data.email.len() > 1 && users.iter().all(|ou| ou.email != data.email){
        if data.card_hash.len() > 0{
            User::create(&persistance.get_conn()?, &mut User::new(Usertype::from_str(&data.usertype), &data.email, &data.card_hash, "Default project"))?;
            let msg = String::from("User created");
            set_message_and_redirect!(msg, "/usersettings", cookies)
        }else{
            let msg = String::from("Card hash too short");
            set_message_and_redirect!(msg, "/usersettings", cookies)
        }
    }else{
        let msg = String::from("Email already registered");
        set_message_and_redirect!(msg, "/usersettings", cookies)
    }
}

#[derive(FromForm)]
struct AdminUsernameUpdate{
    userid: i32,
    email: String
}
#[post("/admin_update_user_email", data="<data>")]
fn admin_update_username(_admin: AdminUser, data: Form<AdminUsernameUpdate>, persistance: State<Persistance>, mut cookies: Cookies) -> Result<Redirect, Box<Error>>{
    let mut u = User::query(persistance.get_conn()?).Where(User::fields().id, data.userid).get().ok_or(SimpleError::new("User not in db"))?;
    let users = User::query(persistance.get_conn()?).all()?;
    if data.email.len() > 1 && users.iter().all(|ou| ou.email != data.email){
        u.email = data.email.clone();
        User::update(&persistance.get_conn()?,&u)?;
        let msg = String::from("Email changed");
        set_message_and_redirect!(msg, "/usersettings", cookies)
    }else{
        let msg = String::from("Email already registered");
        set_message_and_redirect!(msg, "/usersettings", cookies)
    }
}

#[get("/status")]
fn status(_admin: AdminUser,u: Usertype, persistance: State<Persistance>) -> Result<Template, Box<Error>>{
    Ok(Template::render("status", json!({})))
}

#[get("/logs")]
fn logs(_admin: AdminUser,u: Usertype, persistance: State<Persistance>) -> Result<Template, Box<Error>>{
    Ok(Template::render("logs", json!({})))
}



fn main() {
    rocket::ignite()
    .manage(model::Persistance::new())
    .mount("/", routes![
        favicon, //Icon 
        login_form, overview, files, //Login, main page, static files
        clear_message, //Message clearing for the popup system
        login, logout,  //POST-Actions for login and logout
        usersettings, admin_create_user, admin_update_username, //User-Settings-Page and associated actions
        status, logs
    ])
    .attach(Template::fairing()) //Templating-system
    //Ad-hoc fairing that forwards to login from any restricted path hit while not logged in
    .attach(login_users::get_logon_redirect_fairing())
    .launch();
}