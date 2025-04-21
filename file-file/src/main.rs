use actix::prelude::*;
use actix_files::{NamedFile, self}; // Import NamedFile
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, middleware};
use actix_web_actors::ws;
use bytes::Bytes;
use std::{
    fs, // For creating directory
    io,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use log;
use uuid::Uuid; // Import Uuid

// --- Configuration ---
const TEMP_UPLOAD_DIR: &str = "./temp_uploads"; // Directory to store files temporarily

// --- WebSocket Actor ---

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
struct FileWsSession {
    hb: Instant,
    filename_to_process: Option<String>,
}

impl FileWsSession {
    fn new() -> Self {
        Self {
            hb: Instant::now(),
            filename_to_process: None,
        }
    }

    // Heartbeat logic (same as before)
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                log::info!("Websocket Client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for FileWsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("WebSocket session started");
        self.hb(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        log::info!("WebSocket session stopped");
        Running::Stop
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for FileWsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                let filename = text.trim().to_string();
                 log::info!("Received potential filename: {}", filename);
                if !filename.is_empty() {
                    // Sanitize filename slightly (basic example)
                    let sanitized_filename = filename
                        .chars()
                        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
                        .collect();
                    self.filename_to_process = Some(sanitized_filename);
                    ctx.text(format!(
                        "Server received filename: '{}'. Ready for binary data.",
                        self.filename_to_process.as_ref().unwrap()
                    ));
                } else {
                     log::warn!("Received empty text message, ignoring as filename.");
                     ctx.text("Server received empty text message.");
                }
            }
            Ok(ws::Message::Binary(bin)) => {
                self.hb = Instant::now();
                log::info!("Received binary data ({} bytes).", bin.len());

                if let Some(fname) = self.filename_to_process.take() {
                    let file_id = Uuid::new_v4().to_string();
                    let unique_filename = format!("{}-{}", file_id, fname);
                    let temp_dir = PathBuf::from(TEMP_UPLOAD_DIR);
                    let file_path = temp_dir.join(&unique_filename);

                    log::info!(
                        "Attempting to save file '{}' (UUID: {}) to path: {:?}",
                        fname, file_id, file_path
                    );

                    // --- Perform blocking file write in a blocking thread ---
                    let file_path_clone = file_path.clone(); // Clone for the closure
                    let actor_address = ctx.address(); // Get address to send result back

                    // web::block ensures fs::write doesn't block the async event loop
                    actix_web::rt::spawn(async move {
                        let write_result = web::block(move || {
                             fs::write(&file_path_clone, &bin) // fs::write takes &[u8], Bytes derefs to it
                        }).await;

                        match write_result {
                            Ok(Ok(())) => {
                                log::info!("Successfully saved file to {:?}", file_path);
                                let success_msg = format!("SAVED:{}:{}", file_id, fname);
                                // Send success message back to the WebSocket client
                                actor_address.do_send(WsMessage(success_msg));
                            }
                            Ok(Err(io_err)) => {
                                log::error!("Failed to write file to {:?}: {}", file_path, io_err);
                                let error_msg = format!("ERROR:Failed to save file on server: {}", io_err);
                                actor_address.do_send(WsMessage(error_msg));
                            }
                            Err(block_err) => {
                                log::error!("Blocking task failed for file write: {}", block_err);
                                let error_msg = format!("ERROR:Server task failed during file save.");
                                actor_address.do_send(WsMessage(error_msg));
                            }
                        }
                    });
                    // --- End blocking file write ---

                } else {
                    log::warn!("Received binary data but no filename was stored. Ignoring.");
                    ctx.text("Error: Server received binary data without receiving a filename first.");
                }
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            // ... other ws::Message types handling ...
            Ok(ws::Message::Continuation(_)) => {
                log::warn!("Continuation frames not supported in this simple example");
                ctx.stop();
            }
            Ok(ws::Message::Nop) => (),
            Err(e) => {
                 log::error!("WebSocket Protocol Error: {}", e);
                 ctx.stop()
            },
        }
    }
}

// Need a simple message type to send results back from the blocking task
#[derive(Message)]
#[rtype(result = "()")]
struct WsMessage(String);

// Handler for messages sent back from the blocking task
impl Handler<WsMessage> for FileWsSession {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}


// --- HTTP Routes ---

async fn index() -> impl Responder {
    NamedFile::open_async("./static/index.html").await.map_err(|e| {
        log::error!("Failed to open index.html: {}", e);
        actix_web::error::ErrorInternalServerError("Could not load page")
    })
}

async fn ws_route(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    log::info!("WebSocket connection attempt from: {:?}", req.peer_addr());
    ws::start(FileWsSession::new(), &req, stream)
}

// --- NEW: HTTP Download Route ---
async fn download_file(path: web::Path<(String, String)>) -> Result<NamedFile, Error> {
    let (uuid_str, filename) = path.into_inner();
    let unique_filename = format!("{}-{}", uuid_str, filename);
    let file_path = Path::new(TEMP_UPLOAD_DIR).join(unique_filename);

    log::info!("Download request for file at path: {:?}", file_path);

    // Attempt to open the file
    let named_file = NamedFile::open_async(&file_path).await
        .map_err(|e| {
            log::error!("Failed to open file {:?} for download: {}", file_path, e);
            match e.kind() {
                io::ErrorKind::NotFound => actix_web::error::ErrorNotFound("File not found or already deleted."),
                _ => actix_web::error::ErrorInternalServerError("Error accessing file."),
            }
        })?;

    // Set Content-Disposition to trigger download with the original filename
    Ok(named_file
        .set_content_disposition(actix_web::http::header::ContentDisposition {
            disposition: actix_web::http::header::DispositionType::Attachment,
            parameters: vec![actix_web::http::header::DispositionParam::Filename(filename)],
        }))
}


// --- Main Server Setup ---

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // --- Create temporary upload directory ---
    let temp_dir = Path::new(TEMP_UPLOAD_DIR);
    if !temp_dir.exists() {
         log::info!("Creating temporary upload directory: {:?}", temp_dir);
        fs::create_dir_all(temp_dir)?;
    } else {
         log::info!("Temporary upload directory already exists: {:?}", temp_dir);
    }
    // --- ---

    let server_addr = "127.0.0.1";
    let server_port = 8080;

    log::info!("Starting HTTP server at http://{}:{}", server_addr, server_port);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default()) // Logger should usually be one of the first middleware
            .route("/", web::get().to(index)) // Serve the HTML page
            .route("/ws", web::get().to(ws_route)) // WebSocket endpoint
            .route("/download/{uuid}/{filename}", web::get().to(download_file)) // NEW Download route
            // Optional: Serve static CSS/JS if needed
            // .service(actix_files::Files::new("/static", "./static"))
    })
    .workers(2)
    .bind((server_addr, server_port))?
    .run()
    .await
}