use actix_web::{ web, App, HttpServer };
use actix_files as fs;
mod db;
mod tts_api;  
mod user_api; 
use tts_api::{ plus_tts, pro_tts, file_tts };
use user_api::{ create_user, signin, update_plan };

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("http://127.0.0.1:5566");
    if let Err(e) = db::ensure_db_exists() {
        eprintln!("Database initialization failed: {}", e);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Database initialization failed"));
    }
    HttpServer::new(|| {
        App::new()
            .route("/user", web::post().to(create_user))
            .route("/signin", web::post().to(signin))
            .route("/plan", web::post().to(update_plan))
            .route("/plustts", web::post().to(plus_tts))
            .route("/protts", web::get().to(pro_tts))
            .route("/filetts", web::get().to(file_tts))
            .route("/favicon.ico", web::get().to(|| {async { fs::NamedFile::open_async("../client/favicon.ico").await }}))
            .service(fs::Files::new("/", "../client").index_file("index.html"))
    })
    .bind("127.0.0.1:5566")?.run().await
}
