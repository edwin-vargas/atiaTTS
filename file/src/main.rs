use actix_web::{get, rt, web, App, Error, HttpRequest, HttpResponse, HttpServer};
// Correct imports based on common actix-ws v0.2 usage
use actix_ws::{Message, MessageStream, ProtocolError, Session};
// Import AggregatedMessage if you explicitly aggregate continuations
use actix_ws::AggregatedMessage;
use futures_util::stream::{StreamExt, TryStreamExt}; // Need TryStreamExt for aggregate
use rand::Rng;
use std::path::PathBuf;
use std::process::Command;
// Arc removed as we won't share Session that way anymore
// use std::sync::Arc;
use std::thread;
use tokio::fs;
use tokio::sync::mpsc;

// Struct para pasar datos al thread de espeak
struct EspeakJob {
    original_filename: String,
    text_content: String,
}

// Struct para enviar resultados de vuelta al WebSocket
#[derive(Debug)] // Added Debug for easier logging if needed
enum EspeakResult {
    Success(String, Vec<u8>), // original_filename, wav_bytes
    Failure(String),          // original_filename
}

async fn process_text_file(
    job: EspeakJob,
    result_sender: mpsc::UnboundedSender<EspeakResult>,
) {
    // Generar nombre único para el archivo WAV temporal
    let random_suffix: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    let temp_wav_filename = format!(
        "{}_{}.wav",
        job.original_filename
            .split('.')
            .next()
            .unwrap_or("output"), // Usa nombre base o 'output'
        random_suffix
    );
    let output_file_path = PathBuf::from(&temp_wav_filename);

    println!("Executing espeak for {}: Outputting to {}", job.original_filename, output_file_path.display());

    // Ejecutar espeak (bloqueante, por eso está en thread::spawn)
    let status = Command::new("espeak")
        .arg(&job.text_content)
        .arg("-w")
        .arg(&output_file_path)
        .status(); // Consider adding .output() for stderr on failure

    match status {
        Ok(exit_status) if exit_status.success() => {
            println!("espeak success for {}", job.original_filename);
            // Leer el archivo WAV generado
            match fs::read(&output_file_path).await {
                Ok(wav_bytes) => {
                    println!("Read {} bytes for {}", wav_bytes.len(), job.original_filename);
                    // Enviar bytes de vuelta
                    if result_sender.send(EspeakResult::Success(
                        job.original_filename.clone(),
                        wav_bytes,
                    )).is_err() {
                         eprintln!("Receiver dropped for success message: {}", job.original_filename);
                    }
                }
                Err(e) => {
                    eprintln!("Error leyendo archivo WAV '{}': {}", temp_wav_filename, e);
                    if result_sender.send(EspeakResult::Failure(job.original_filename.clone())).is_err() {
                         eprintln!("Receiver dropped for read failure message: {}", job.original_filename);
                    }
                }
            }
            // Limpiar archivo temporal (ignorar error si falla)
            if let Err(e) = fs::remove_file(&output_file_path).await {
                 eprintln!("Failed to remove temp file '{}': {}", output_file_path.display(), e);
            }
        }
        Ok(exit_status) => {
             eprintln!("espeak failed for '{}' with status: {}", job.original_filename, exit_status);
              if result_sender.send(EspeakResult::Failure(job.original_filename.clone())).is_err() {
                 eprintln!("Receiver dropped for espeak failure status message: {}", job.original_filename);
              }
             // Intentar limpiar si el archivo se creó parcialmente
             let _ = fs::remove_file(&output_file_path).await;
        }
        Err(e) => {
            eprintln!("Error executing espeak for '{}': {}", job.original_filename, e);
            if result_sender.send(EspeakResult::Failure(job.original_filename.clone())).is_err() {
                eprintln!("Receiver dropped for espeak execution error message: {}", job.original_filename);
            }
            // Intentar limpiar si el archivo se creó parcialmente
            let _ = fs::remove_file(&output_file_path).await;
        }
    }
}

async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    // *** Ownership Change: Session is NOT put in Arc ***
    let (res, mut session, msg_stream) = actix_ws::handle(&req, stream)?; // Make session mutable here

    // Canal para comunicar resultados desde los threads de espeak hacia esta tarea WS
    let (result_sender, mut result_receiver) = mpsc::unbounded_channel::<EspeakResult>();

    // Tarea para recibir mensajes del cliente y lanzar threads
    // msg_stream is moved into this task
    rt::spawn(handle_incoming_messages(
        msg_stream, // msg_stream is moved here
        result_sender, // Sender is cloned and moved
    ));

    // Tarea para recibir resultados de los threads y enviarlos al cliente
    // *** Ownership Change: Session is MOVED into this task ***
    rt::spawn(async move {
        // `session` is now owned by this task
        while let Some(result) = result_receiver.recv().await {
            match result {
                EspeakResult::Success(filename, wav_bytes) => {
                    println!(
                        "Sending audio for: {} ({} bytes)",
                        filename,
                        wav_bytes.len()
                    );
                    // Call methods directly on the owned `session`
                    // Use map_err for basic error logging on send failures
                    if let Err(e) = session.text(format!("audio::{}", filename)).await {
                        eprintln!("Error sending text marker for {}: {}", filename, e);
                        break; // Stop processing if we can't send
                    }
                    if let Err(e) = session.binary(wav_bytes).await {
                         eprintln!("Error sending binary data for {}: {}", filename, e);
                         break; // Stop processing if we can't send
                    }
                }
                EspeakResult::Failure(filename) => {
                     println!("Espeak failed for: {}", filename);
                    // Informar al cliente del fallo (opcional)
                     if let Err(e) = session.text(format!("error::{}", filename)).await {
                         eprintln!("Error sending error marker for {}: {}", filename, e);
                         break; // Stop processing if we can't send
                     }
                }
            }
        }
        println!("Result channel closed, closing WebSocket session.");
        // Call close directly on the owned `session`
        let _ = session.close(None).await; // This consumes the session
    });

    Ok(res) // Responde HTTP 101 Switching Protocols
}

// Función separada para manejar los mensajes entrantes del WebSocket
async fn handle_incoming_messages(
    msg_stream: MessageStream, // Takes ownership of the stream
    result_sender: mpsc::UnboundedSender<EspeakResult>, // Takes ownership of sender clone
) {
    // *** Crucial Fix: Aggregate continuation frames ***
    // Process the stream to combine fragmented messages (Text/Binary)
    let mut aggregated_stream = msg_stream
        .aggregate_continuations()
        // Set a limit for aggregated message size (e.g., 1MB) to prevent DoS
        .max_continuation_size(1024 * 1024); // 1 MiB

    while let Some(msg) = aggregated_stream.next().await {
        match msg {
            // *** Now match on AggregatedMessage ***
            Ok(AggregatedMessage::Text(text)) => {
                // Asumimos formato "filename.txt::contenido del archivo..."
                if let Some((filename, content)) = text.split_once("::") {
                     println!("Received text for: {}", filename);
                     // Basic validation
                     if filename.trim().is_empty() || content.trim().is_empty() {
                         eprintln!("Received empty filename or content.");
                         continue;
                     }

                    let job = EspeakJob {
                        original_filename: filename.to_string(),
                        text_content: content.to_string(),
                    };
                    let sender_clone = result_sender.clone(); // Clone sender for the thread

                    // Lanzar thread bloqueante para espeak
                    thread::spawn(move || {
                        // Necesitamos un runtime de Tokio dentro del thread para fs::read/remove
                        let rt = match tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build() {
                                Ok(r) => r,
                                Err(e) => {
                                     eprintln!("Failed to build tokio runtime in thread: {}", e);
                                     // Send failure back immediately if runtime fails
                                      let _ = sender_clone.send(EspeakResult::Failure(job.original_filename));
                                     return;
                                }
                            };

                        rt.block_on(process_text_file(job, sender_clone));
                    });
                } else {
                    eprintln!("Received text message with unexpected format.");
                }
            }
            Ok(AggregatedMessage::Ping(msg)) => {
                 println!("WebSocket Ping received");
                 // actix-ws should handle Pong automatically if not intercepted
            }
            Ok(AggregatedMessage::Pong(_)) => {
                 println!("WebSocket Pong received");
            }
            Ok(AggregatedMessage::Close(reason)) => {
                 println!("Client initiated close: {:?}", reason);
                break; // Salir del bucle si el cliente cierra
            }
             Ok(AggregatedMessage::Binary(_)) => {
                 eprintln!("Received unexpected binary message from client.");
                 // Decide how to handle - maybe close connection?
             }
            Err(e) => {
                 // Handle potential errors during aggregation or reading from the stream
                 match e {
                     actix_ws::AggregationError::MaxContinuationSizeExceeded => {
                          eprintln!("WebSocket message exceeded max size limit.");
                     }
                     actix_ws::AggregationError::ProtocolError(pe) => {
                         eprintln!("WebSocket protocol error: {}", pe);
                     }
                     // Add other AggregationError variants if needed
                 }
                break; // Salir en caso de error irrecuperable
            }
        }
    }
     println!("Incoming message loop finished.");
    // result_sender is dropped here automatically when the function scope ends,
    // which signals the other task to finish by closing the channel.
}


#[get("/")] // Sirve el HTML
async fn index() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html"))) // Ensure index.html is in static/
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server on http://127.0.0.1:8080");
    // Set log level if needed: e.g., std::env::set_var("RUST_LOG", "info"); env_logger::init();
    HttpServer::new(|| {
        App::new()
            // Consider adding logger middleware: .wrap(actix_web::middleware::Logger::default())
            .service(index) // Ruta para el HTML
            .route("/ws", web::get().to(websocket_handler)) // Ruta para WebSocket
    })
    .bind(("127.0.0.1", 8080))?
    .workers(4) // Adjust as needed
    .run()
    .await
}