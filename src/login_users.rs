use rocket::request::FromRequest;
use rocket::request::Request;
use rocket::request;
use rocket::outcome::Outcome::{Success, Forward};
use model::user::{User, Usertype};
use sqlite_traits::DbObject;
use rocket::State;
use model::Persistance;
use rocket::fairing::AdHoc;
use rocket::response::Responder;
use rocket::response::Redirect;
use rocket::http::{Cookie};


#[derive(Copy, Clone)]
pub struct AdminUser{}

impl From<AdminUser> for Usertype{
    fn from(_: AdminUser) -> Usertype{
        Usertype::Admin
    }
}

#[derive(Copy, Clone)]
pub struct NormalUser{}

impl From<NormalUser> for Usertype{
    fn from(_: NormalUser) -> Usertype{
        Usertype::User
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for AdminUser{
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
        if Usertype::from_request(request)? == Usertype::Admin{
            Success(AdminUser{})
        }else{
            Forward(())
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for NormalUser{
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
        if Usertype::from_request(request)? <= Usertype::User{
            Success(NormalUser{})
        }else{
            Forward(())
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Usertype{
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
        if let Some(cookie) = request.cookies().get_private("token"){
            let p =request.guard::<State<Persistance>>()?;
            if let Some(t) = p.get_user_role(cookie.value()){
                Success(t)
            }else{
                Forward(())
            }
        }else{
            Forward(())
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for User{
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
        if let Some(cookie) = request.cookies().get_private("token"){
            let p =request.guard::<State<Persistance>>()?;
            if let Some(id) = p.get_user_id(cookie.value()){
                    if let Ok(Some(user)) = User::query().Where("id", id).get(){
                        return Success(user)
                    }
                    Forward(())
            }else{
                Forward(())
            }
        }else{
            Forward(())
        }
    }
}

pub fn get_logon_redirect_fairing() -> AdHoc{
    AdHoc::on_response("logon_redirect_fairing", |req, res| {
            if req.uri().path() != "/"{
                let mut cookies =  req.cookies();
                let mut new_cookie = Cookie::new("redirect_to", String::from(req.uri().path()));
                new_cookie.set_path("/");
                if res.status() == rocket::http::Status::NotFound {
                    if let Some(cookie) = cookies.get_private("token"){
                        if let rocket::outcome::Outcome::Success(p) = req.guard::<State<Persistance>>(){
                            if p.get_user_role(cookie.value()).is_none(){
                                if let Ok(new_res) =Redirect::to("/").respond_to(req){
                                    res.merge(new_res);
                                    println!{"{}", String::from(req.uri().path())};
                                    res.adjoin_header(&new_cookie);
                                }
                            }
                        }
                    }else if let Ok(new_res) =Redirect::to("/").respond_to(req){
                            res.merge(new_res);
                            println!{"{}", String::from(req.uri().path())};
                            res.adjoin_header(&new_cookie);
                        }
                    }  
            }
        })
}