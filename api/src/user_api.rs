mod user_db;
mod user_mod;
use user_mod::{
    SignupRequest, 
    SigninRequest, 
    PlanRequest, 
    // SignupResponse, 
    UserResponse, 
    // ErrorResponse, 
    // SuccessResponse
};
use actix_web::{
    web,
    HttpResponse, 
    Responder
};
use serde_json::json;
// User management endpoints

// pub async fn create_user(data: web::Json<SignupRequest>) -> impl Responder {
//     match user_db::insert_user(&data.user_name, &data.user_email, &data.user_pass) {
//         Ok(user_id) => {
//             HttpResponse::Ok().json(SignupResponse {
//                 user_id,
//                 success: true,
//             })
//         }
//         Err(e) => {HttpResponse::InternalServerError().json(ErrorResponse {error: format!("Fallo al crear: {}", e),})}
//     }
// }

pub async fn create_user(data: web::Json<SignupRequest>) -> impl Responder {
    let res= user_db::insert_user(
        &data.user_name, 
        &data.user_email, 
        &data.user_pass
    );
    
    if res.is_ok() {
        let user = res.unwrap();
        HttpResponse::Ok().json(json!({"user_id": user}))
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

pub async fn signin(data: web::Json<SigninRequest>) -> impl Responder {
    let res = user_db::get_user_by_email_pass(
        &data.user_email, 
        &data.user_pass
    );
    
    if res.is_ok() {
        let user = res.unwrap();
        HttpResponse::Ok().json(UserResponse {
            user_id: user.user_id,
            user_name: user.user_name,
            user_email: user.user_email
        })
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

pub async fn update_plan(data: web::Json<PlanRequest>) -> impl Responder {
    let res = user_db::add_user_to_plan(
        &data.user_id, 
        data.plan
    );    
    
    if res.is_ok() {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::InternalServerError().finish()
    }
}
