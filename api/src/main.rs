use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
// use uuid::Uuid;

mod db;

#[derive(Deserialize)]
struct SignupRequest {
    user_name: String,
    user_email: String,
    user_pass: String,
}

#[derive(Deserialize)]
struct SigninRequest {
    user_email: String,
    user_pass: String,
}

#[derive(Deserialize)]
struct PlanRequest {
    user_id: String,
    plan: i32,
}

#[derive(Serialize)]
struct UserResponse {
    user_id: String,
    user_name: String,
    user_email: String,
}

#[derive(Serialize)]
struct SignupResponse {
    user_id: String,
    success: bool,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct SuccessResponse {
    success: bool,
}

async fn create_user(data: web::Json<SignupRequest>) -> impl Responder {
    match db::insert_user(&data.user_name, &data.user_email, &data.user_pass) {
        Ok(user_id) => {
            HttpResponse::Ok().json(SignupResponse {
                user_id,
                success: true,
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to create user: {}", e),
            })
        }
    }
}

async fn signin(data: web::Json<SigninRequest>) -> impl Responder {
    match db::get_user_by_email_pass(&data.user_email, &data.user_pass) {
        Ok(user) => {
            HttpResponse::Ok().json(UserResponse {
                user_id: user.user_id,
                user_name: user.user_name,
                user_email: user.user_email,
            })
        }
        Err(_) => {
            HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Invalid email or password".to_string(),
            })
        }
    }
}

async fn update_plan(data: web::Json<PlanRequest>) -> impl Responder {
    match db::add_user_to_plan(&data.user_id, data.plan) {
        Ok(_) => {
            HttpResponse::Ok().json(SuccessResponse {
                success: true,
            })
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to update plan: {}", e),
            })
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Ensure database exists with all required tables
    if let Err(e) = db::ensure_db_exists() {
        eprintln!("Database initialization failed: {}", e);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Database initialization failed"));
    }
    
    println!("Server starting on http://127.0.0.1:8080");
    
    HttpServer::new(|| {
        App::new()
            // API routes
            .route("/user", web::post().to(create_user))
            .route("/signin", web::post().to(signin))
            .route("/plan", web::post().to(update_plan))
            // Serve static files from the ../client directory
            .service(fs::Files::new("/", "../client").index_file("indigo.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}