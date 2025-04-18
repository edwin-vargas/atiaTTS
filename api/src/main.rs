use actix_web::{
    web, 
    App, 
    HttpServer
};
use actix_files as fs;
mod tts_api;  
mod user_api; 
use user_api::{
    create_user, 
    signin, 
    update_plan
};
use tts_api::text_to_speech;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server on http://localhost:5566");

    HttpServer::new(|| {
        App::new()
            .service(fs::Files::new("/", "../client").index_file("inicio.html"))
            
            // API Routes
            
            .route("/user", web::post().to(create_user))
            .route("/signin", web::post().to(signin))
            .route("/plan", web::post().to(update_plan))
            .route("/tts", web::post().to(text_to_speech))
            //.route("/upload", web::post().to(upload_files)) 
    })
    .bind("127.0.0.1:5566")?
    .run()
    .await
}
