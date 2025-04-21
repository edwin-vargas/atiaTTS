use actix_web::{
    web, 
    App, 
    HttpServer,
    middleware::Logger
};
use actix_files as fs;
use std::fs as std_fs;
use actix_cors::Cors;
mod db;
mod tts_api;  
use tts_api::{plus_tts, upload_multiple_files};
mod user_api; 
use user_api::{
    create_user, 
    signin,
    update_plan
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    db::ensure_db_exists().ok();

    println!("http://127.0.0.1:5566");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec!["Content-Type"])
                    .max_age(3600),
            )
            .route("/user", web::post().to(create_user))
            .route("/signin", web::post().to(signin))
            .route("/plan", web::post().to(update_plan))
            .route("/plustts", web::post().to(plus_tts))
            .route("/protts", web::post().to(upload_multiple_files))
            //GET /user
            //tts/text
            //tts/files
            .service(fs::Files::new("/", "../client").index_file("index.html"))
    })
    .bind("127.0.0.1:5566")?
    .run()
    .await
}
