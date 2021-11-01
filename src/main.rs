#![feature(proc_macro_hygiene, decl_macro)]

extern crate rand;
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
use rocket_contrib::templates::Template;


#[macro_use] extern crate serde_derive;

#[macro_use] extern crate simple_error;
use simple_error::SimpleError;

extern crate sqlite_traits;
#[macro_use] extern crate sqlite_derive;

extern crate serde;
 extern crate serde_json;
use serde_json::Value;


//extern crate reqwest;

extern crate uuid;

mod model;

mod login_users;
use login_users::{AdminUser, NormalUser};

use rocket::response::NamedFile;
use std::path::Path;
use std::path::PathBuf;
use rocket::response::Redirect;
use rocket::request::Form;
use rocket_contrib::json::{JsonValue};
use rocket::http::{Cookie, Cookies};
use rocket::State;

extern crate chrono;
use chrono::prelude::*;

use model::Persistance;
use model::user::{User, Usertype};
use model::log::Log;
use model::DbObject;

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
fn overview(_n: NormalUser,u : User, l: Usertype, persistance: State<Persistance>) ->  Result<Template, Box<dyn Error>> {
    let logs = Log::query()
        .Where("user_email", u.email.clone())
        .OrderBy("timestamp_start", sqlite_traits::Ordering::Descending)
        .all()?
        .iter()
        .map(|l| {
            let tstart = Local.timestamp(l.timestamp_start,0);
            let tdiff = Local.timestamp(l.timestamp_end, 0).signed_duration_since(tstart);
            let tdiffstr = format!{"{:02}:{:02}:{:02}", tdiff.num_hours(), (tdiff.num_minutes() % 60), (tdiff.num_seconds() % 60)};
            json!({
                "date": tstart.to_rfc2822(),
                "duration": tdiffstr,
                "project": l.entry
            }) 
        })
        .collect::<Vec<JsonValue>>();
    Ok(Template::render("overview", json!({
        "Usertype": l.to_json_object(),
        "user": u,
        "logs":logs
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
struct UserProjectUpdate{
    userid: i32,
    current_project: String
}

#[post("/user_update_project", data="<data>")]
fn user_update_project(_admin: AdminUser, data: Form<UserProjectUpdate>, persistance: State<Persistance>, mut cookies: Cookies) -> Result<Redirect, Box<dyn Error>>{
    let mut u = User::query().Where("id", data.userid).get()?.ok_or(SimpleError::new("User not in db"))?;
    u.current_project = data.current_project.clone();
    u.save()?;
    let msg = String::from("User updated");
    set_message_and_redirect!(msg, "/", cookies)
}

#[derive(FromForm)]
struct LoginCredentials{
    username: String
}

#[post("/login", data="<creds>")]
fn login(creds: Form<LoginCredentials>, mut cookies: Cookies, persistance: State<Persistance>) -> Result<Redirect, Box<dyn Error>>{
    eprintln!("Login called");
    if let Ok(token) = persistance.log_in(&creds.username, &persistance.get_last_hash()?){
        cookies.add_private(Cookie::new("token", token));
    }else{
        eprintln!("Login went wrong");
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
fn logout(_l: Usertype, mut cookies: Cookies, persistance: State<Persistance>) -> Result<Redirect, Box<dyn Error>>{
    if let Some(cookie) = cookies.get_private("token"){
        persistance.log_out(cookie.value())?;
        cookies.remove_private(cookie);
    }
    Ok(Redirect::to("/"))
}


#[get("/usersettings")]
fn usersettings(_admin: AdminUser,u: Usertype, persistance: State<Persistance>) -> Result<Template, Box<dyn Error>>{
    let users = User::query().all()?.iter().map(|u| json!({
        "id": u.id, 
        "email": u.email, 
        "card_hash": u.card_hash,
        "current_project": u.current_project,
        "Usertype": Usertype::from_str(&u.usertype).to_json_object()
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
    card_hash: String,
    current_project: String
}

#[post("/admin_create_user", data="<data>")]
fn admin_create_user(_admin: AdminUser, data: Form<UserCreation>, persistance: State<Persistance>, mut cookies: Cookies) -> Result<Redirect, Box<dyn Error>>{
    let users = User::query().all()?;
    if data.email.len() > 1 && users.iter().all(|ou| ou.email != data.email){
        if data.card_hash.len() > 0{
            let mut user = User::new(Usertype::from_str(&data.usertype), &data.email, &data.card_hash, &data.current_project);
            user.save()?;
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
struct AdminUserUpdate{
    userid: i32,
    usertype: String,
    email: String,
    card_hash: String,
    current_project: String
}
#[post("/admin_update_user", data="<data>")]
fn admin_update_username(_admin: AdminUser, data: Form<AdminUserUpdate>, persistance: State<Persistance>, mut cookies: Cookies) -> Result<Redirect, Box<dyn Error>>{
    let mut u = User::query().Where("id", data.userid).get()?.ok_or(SimpleError::new("User not in db"))?;
    if data.card_hash.len() > 0{
        u.usertype = data.usertype.clone();
        println!("{:?}, {:?}", u.usertype, &data.usertype);
        u.email = data.email.clone();
        u.card_hash = data.card_hash.clone();
        u.current_project = data.current_project.clone();
        u.save()?;
        let msg = String::from("User updated");
        set_message_and_redirect!(msg, "/usersettings", cookies)
    }else{
        let msg = String::from("Card hash too short");
        set_message_and_redirect!(msg, "/usersettings", cookies)
    }
}

#[derive(FromForm)]
struct UserId{
    userid: i32
}

#[post("/admin_delete_user", data="<data>")]
fn admin_delete_user(_admin: AdminUser, data: Form<UserId>, persistance: State<Persistance>, mut cookies: Cookies) -> Result<Redirect, Box<dyn Error>>{
    let mut u = User::query().Where("id", data.userid).get()?.ok_or(SimpleError::new("User not in db"))?;
    u.delete()?;
    let msg = String::from("User deleted");
    set_message_and_redirect!(msg, "/usersettings", cookies)
}


#[get("/status")]
fn status()-> Option<NamedFile> {
    NamedFile::open(Path::new("templates/status.html")).ok()
}

#[get("/status.json")]
fn status_json(_admin: AdminUser, persistance: State<Persistance>) -> Result<JsonValue, Box<dyn Error>>{
    Ok(json!({
        "hash": persistance.get_last_hash()?,
        "user": persistance.get_current_user()?
    }))
}

#[get("/logs")]
fn logs(_admin: AdminUser,u: Usertype, persistance: State<Persistance>) -> Result<Template, Box<dyn Error>>{
        let logs = Log::query()
        .OrderBy("timestamp_start", sqlite_traits::Ordering::Descending)
        .all()?
        .iter()
        .map(|l| {
            let tstart = Local.timestamp(l.timestamp_start,0);
            let tdiff = Local.timestamp(l.timestamp_end, 0).signed_duration_since(tstart);
            let tdiffstr = format!{"{:02}:{:02}:{:02}", tdiff.num_hours(), (tdiff.num_minutes() % 60), (tdiff.num_seconds() % 60)};
            json!({
                "email": l.user_email,
                "date": tstart.to_rfc2822(),
                "duration": tdiffstr,
                "project": l.entry
            }) 
        })
        .collect::<Vec<JsonValue>>();
    Ok(Template::render("logs", json!({
        "Usertype": u.to_json_object(),
        "logs":logs
    })))
}


#[get("/unlock/<card_hash>")]
fn unlock(card_hash: String, persistance: State<Persistance>) -> Result<String, Box<dyn Error>>{
    //Store hash
    persistance.set_last_hash(card_hash.clone())?;
    if persistance.get_current_user()?.is_none() {
        //Find a user with that card hash
        let u = User::query().Where("card_hash", card_hash).get()?.ok_or(SimpleError::new("User not in db"))?;
        //Create log entry
        let mut l = Log::new(u.email.clone(), u.current_project.clone(), Local::now().timestamp(), 0);
        //Save log entry in DB
        l.save();
        //Set user in persistance
        persistance.set_current_user(Some(u))?;
        persistance.set_current_log(Some(l))?;
        Ok(String::from("Laser unlocked"))
    }else{
        Err(Box::new(SimpleError::new("Laser already in use")))
    }
}

#[get("/lock")]
fn lock(persistance: State<Persistance>) -> Result<String, Box<dyn Error>>{
    if !persistance.get_current_user()?.is_none() {
        //Finish log entry
        let l = persistance.get_current_log()?;
        if let Some(mut log) = l{
            log.timestamp_end = Local::now().timestamp();
            log.save()?;
        }

        //Log out
        persistance.set_current_log(None)?;
        persistance.set_current_user(None)?;
        Ok(String::from("Laser locked"))
    }else{
        Ok(String::from("Laser already in use"))
    }
}

fn main() {
    rocket::ignite()
    .manage(model::Persistance::new())
    .mount("/", routes![
        favicon, //Icon 
        login_form, overview, files, //Login, main page, static files
        clear_message, //Message clearing for the popup system
        login, logout,  //POST-Actions for login and logout
        usersettings, admin_create_user, admin_update_username, admin_delete_user, user_update_project,//User-Settings-Page and associated actions
        status, status_json, logs,
        lock, unlock
    ])
    .attach(Template::fairing()) //Templating-system
    //Ad-hoc fairing that forwards to login from any restricted path hit while not logged in
    .attach(login_users::get_logon_redirect_fairing())
    .launch();
}