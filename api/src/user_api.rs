mod user_db;
use actix_web::{ web, HttpResponse, Responder };
use serde::{ Deserialize, Serialize };
use serde_json::json;

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub user_pass: Option<String>,
    pub plan: Option<i32>,
}


pub async fn create_user(req: web::Json<User>) -> impl Responder {
    let res= user_db::insert_user(
        req.user_name.as_ref().unwrap(), 
        &req.user_email.as_ref().unwrap(), 
        req.user_pass.as_ref().unwrap()
    );
    
    if res.is_ok() {
        HttpResponse::Ok().json(json!({"user_id": res.unwrap()}))
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

pub async fn signin(req: web::Json<User>) -> impl Responder {
    let res = user_db::get_user_by_email_pass(
        req.user_email.as_ref().unwrap(), 
        req.user_pass.as_ref().unwrap()
    );

    if res.is_ok() {
        let user = res.unwrap();
        HttpResponse::Ok().json(json!({"user_id":user.user_id}))
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

pub async fn update_plan(req: web::Json<User>) -> impl Responder {
    let res = user_db::add_user_to_plan(
        req.user_id.as_ref().unwrap(), 
        req.plan.unwrap()
    );    
    
    if res.is_ok() {
        HttpResponse::Ok().json(json!({"plan": req.plan}))
    } else {
        HttpResponse::InternalServerError().finish()
    }
}
