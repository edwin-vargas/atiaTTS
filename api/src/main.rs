use actix_web::{
    web, 
    App, 
    HttpServer
};
use actix_files as fs;

mod db;
mod tts_api;  
use tts_api::text_to_speech;
mod user_api; 
use user_api::{
    create_user, 
    signin, 
    update_plan
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    // ¿A dónde sin DB?
    
    if let Err(e) = db::ensure_db_exists() {
        eprintln!("Database initialization failed: {}", e);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Database initialization failed"));
    }

    println!("Server on http://127.0.0.1:5566");

    HttpServer::new(|| {
        App::new()
            
            // API Rutas de combis
            
            .route("/user", web::post().to(create_user))
            .route("/signin", web::post().to(signin))
            .route("/plan", web::post().to(update_plan))
            .route("/tts", web::post().to(text_to_speech))
            //.route("/upload", web::post().to(upload_files))
            .service(fs::Files::new("/", "../client").index_file("indigo.html"))
    })
    .bind("127.0.0.1:5566")?
    .run()
    .await
}
