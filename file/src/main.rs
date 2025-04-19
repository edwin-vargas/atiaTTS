use actix::*;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use actix_files::NamedFile;
use bytes::Bytes; // Import Bytes
use std::time::{Duration, Instant};
use log;

// --- WebSocket Actor ---

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

struct FileWsSession {
    hb: Instant,
    /// Stores the filename received in the text message before the binary data
    filename: Option<String>,
}

impl FileWsSession {
    fn new() -> Self {
        Self {
            hb: Instant::now(),
            filename: None,
        }
    }

    // Heartbeat logic
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
                // Assume the first text message is the filename
                let filename = text.trim().to_string();
                log::info!("Received potential filename: {}", filename);
                self.filename = Some(filename);
                // Optionally send confirmation back
                // ctx.text(format!("Ready to receive file: {}", self.filename.as_ref().unwrap()));
            }
            Ok(ws::Message::Binary(bin)) => {
                self.hb = Instant::now();
                log::info!("Received binary data.");

                if let Some(fname) = self.filename.take() { // Take ownership and clear
                    log::info!("Processing binary data for file: {}", fname);

                    // --- THIS IS WHERE YOU PROCESS THE FILE CONTENT ---
                    // For this example, we try to decode as UTF-8 and print
                    // WARNING: This will fail or print garbage for non-text files!
                    match String::from_utf8(bin.to_vec()) {
                        Ok(content) => {
                            println!("\n--- File Content for: {} ---\n", fname);
                            println!("{}", content);
                            println!("--- End of Content ---");
                            ctx.text(format!("Successfully processed text content for: {}", fname));
                        }
                        Err(_) => {
                            // Handle non-UTF8 data - just print bytes count here
                            let byte_count = bin.len();
                            println!("\n--- Received Binary Data for: {} ---", fname);
                            println!("Size: {} bytes", byte_count);
                            println!("(Could not decode as UTF-8 text)");
                            println!("--- End of Binary Data ---");
                             ctx.text(format!("Received {} bytes of binary data for: {}", byte_count, fname));
                        }
                    }
                     // --- End of Processing ---

                } else {
                    log::warn!("Received binary data but no filename was previously sent. Ignoring.");
                     ctx.text("Error: Received binary data without filename first.");
                }
                // Reset filename state for the next potential file
                 self.filename = None;
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
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

// --- HTTP Routes ---

async fn index() -> impl Responder {
    // Serve the HTML file
    NamedFile::open_async("./static/index.html").await.map_err(|e| {
        log::error!("Failed to open index.html: {}", e);
        actix_web::error::ErrorInternalServerError("Could not load page")
    })
}

/// WebSocket entry point
async fn ws_route(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    log::info!("WebSocket connection attempt from: {:?}", req.peer_addr());
    ws::start(FileWsSession::new(), &req, stream)
}


// --- Main Server Setup ---

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Setup logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let server_addr = "127.0.0.1";
    let server_port = 8080;

    log::info!("Starting HTTP server at http://{}:{}", server_addr, server_port);

    HttpServer::new(move || {
        App::new()
            .route("/", web::get().to(index)) // Route for the HTML page
            .route("/ws", web::get().to(ws_route)) // Route for WebSocket connection
            // Optional: Serve static files if your HTML needs CSS/JS
            // .service(Files::new("/static", "./static"))
            .wrap(actix_web::middleware::Logger::default()) // Log HTTP requests
    })
    .workers(2)
    .bind((server_addr, server_port))?
    .run()
    .await
}