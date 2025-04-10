use actix_web::{get, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Ya jala la pinshi API, buenas noches")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server on http://localhost:3000");
    HttpServer::new(|| {
        App::new()
            .service(hello)
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}