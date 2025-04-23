use actix_web::{
    web, 
    App, 
    HttpServer
};

use actix_files as fs;
mod db;
mod tts_api;  
use tts_api::{plus_tts, pro_tts};
mod user_api; 
use user_api::{
    create_user, 
    signin,
    update_plan
};


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    println!("http://127.0.0.1:5566");

    HttpServer::new(|| {
        App::new()
            .route("/user", web::post().to(create_user))
            .route("/signin", web::post().to(signin))
            .route("/plan", web::post().to(update_plan))
            .route("/plustts", web::post().to(plus_tts))
            .route("/protts", web::get().to(pro_tts))
            .service(fs::Files::new("/", "../client").index_file("index.html"))
    })
    .bind("127.0.0.1:5566")?
    .run()
    .await
}
