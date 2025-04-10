use actix_web::{get, App, HttpResponse, HttpServer, Responder};
mod espeaker;

#[get("/")]
async fn hello() -> impl Responder {
    espeaker::generate();
    HttpResponse::Ok().body("Hello from server")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server >> http://localhost:3000");
    HttpServer::new(|| {
        App::new()
            .service(hello)
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}