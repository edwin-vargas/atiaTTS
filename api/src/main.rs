use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use uuid::Uuid;
use serde::Serialize;
use rusqlite::{params, Connection, Result};

#[derive(Serialize)]  
struct UserResponse {
    id: String,
}

fn init_db() -> Result<Connection> {
    let conn = Connection::open("users.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY
        )",
        [],
    )?;
    Ok(conn)
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hola")
}

#[get("/user")]
async fn post_user() -> impl Responder {
    let uuid = Uuid::new_v4();
    
    let conn = match init_db() {
        Ok(c) => c,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to open DB"),
    };

    let insert_result = conn.execute(
        "INSERT INTO users (id) VALUES (?1)",
        params![uuid.to_string()],
    );

    match insert_result {
        Ok(_) => {
            let response = UserResponse {
                id: uuid.to_string(),
            };
            HttpResponse::Ok().json(response)
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to insert user"),
    }

}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server >> http://localhost:3000");
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(post_user)
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}