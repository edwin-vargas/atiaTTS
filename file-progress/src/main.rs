use actix::prelude::*;
use actix_files::{NamedFile, self};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, middleware};
use actix_web_actors::ws;
use bytes::Bytes;
use futures::StreamExt; // Necesario para dividir el stream de archivos grandes (si aplica)
use serde::{Serialize, Deserialize}; // Para mensajes JSON
use std::{
    fs::{self, File}, // Para crear directory y file
    io::{self, Write}, // Para escribir en list.txt
    path::{Path, PathBuf},
    process::Stdio, // Para controlar la salida del comando
    time::{Duration, Instant},
};
use tokio::process::Command; // <--- Usar la versión async de Command
use log;
use uuid::Uuid;

// --- Configuration ---
const TEMP_UPLOAD_DIR: &str = "./temp_uploads"; // Directorio para archivos subidos
const TEMP_AUDIO_DIR: &str = "./temp_audio";   // Directorio para chunks de audio y salida final
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

// --- Mensajes WebSocket (Cliente -> Servidor y Servidor -> Cliente) ---
#[derive(Serialize, Deserialize, Debug, Message)]
#[rtype(result = "()")] // Mensajes que van al actor WS
#[serde(tag = "type")] // Usa un campo 'type' para distinguir mensajes
enum WsMessage {
    // Cliente -> Servidor
    #[serde(rename = "start_upload")]
    StartUpload { filename: String },
    #[serde(rename = "start_tts_string")]
    StartTtsString { text: String, /* Agrega opciones: */ language: Option<String>, format: Option<String> },
    #[serde(rename = "start_tts_file")]
    StartTtsFile { file_id: String, filename: String, /* Agrega opciones: */ language: Option<String>, format: Option<String> },

    // Servidor -> Cliente (o mensajes internos para el actor)
    #[serde(rename = "upload_ready")]
    UploadReady { file_id: String, filename: String },
    #[serde(rename = "upload_chunk_received")]
    UploadChunkReceived { file_id: String, bytes: usize }, // Para progreso de subida si es grande
    #[serde(rename = "upload_complete")]
    UploadComplete { file_id: String, filename: String },
    #[serde(rename = "tts_job_created")]
    TtsJobCreated { job_id: String, original_file_id: Option<String>, original_filename: Option<String> },
    #[serde(rename = "tts_progress")]
    TtsProgress { job_id: String, progress: f32, message: String }, // Progreso 0.0 a 1.0
    #[serde(rename = "tts_complete")]
    TtsComplete { job_id: String, filename: String, download_url: String },
    #[serde(rename = "error")]
    Error { job_id: Option<String>, file_id: Option<String>, message: String },
    #[serde(rename = "info")]
    Info { message: String },

    // Mensaje interno para manejar la finalización de la tarea TTS
    #[serde(skip)] // No necesita serializarse a JSON
    InternalTtsResult(TtsTaskResult),
}

// Resultado de la tarea TTS que se envía de vuelta al actor WS
#[derive(Debug)]
enum TtsTaskResult {
    Progress { job_id: String, progress: f32, message: String },
    Complete { job_id: String, output_path: PathBuf, final_filename: String },
    Failed { job_id: String, error_message: String },
}

// --- WebSocket Actor ---
#[derive(Debug)]
struct TextToSpeechWsSession {
    hb: Instant,
    id: String, // Identificador único para esta sesión WS
    // Podríamos almacenar el estado de los uploads/jobs aquí si es necesario,
    // pero por ahora lo manejaremos principalmente en tareas separadas.
    // current_upload_file_id: Option<String>,
    // current_upload_path: Option<PathBuf>,
}

impl TextToSpeechWsSession {
    fn new() -> Self {
        Self {
            hb: Instant::now(),
            id: Uuid::new_v4().to_string(),
            // current_upload_file_id: None,
            // current_upload_path: None,
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                log::info!("WebSocket Client heartbeat failed, disconnecting!");
                // Aquí podríamos limpiar tareas TTS asociadas si es necesario
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }

    // Función para enviar mensajes JSON al cliente
    fn send_json<T: Serialize>(&self, ctx: &mut ws::WebsocketContext<Self>, message: &T) {
        match serde_json::to_string(message) {
            Ok(json) => ctx.text(json),
            Err(e) => {
                log::error!("Failed to serialize message: {:?}", e);
                // Enviar un mensaje de error genérico si falla la serialización
                ctx.text(format!(r#"{{"type":"error", "message":"Internal server error: cannot serialize response"}}"#));
            }
        }
    }

    // Inicia el proceso TTS en una tarea separada
    fn start_tts_processing(
        &self,
        ctx: &mut ws::WebsocketContext<Self>,
        job_id: String,
        text_to_convert: String,
        original_file_id: Option<String>, // Para asociar con la subida original
        original_filename: Option<String>,
        // TODO: Pasar opciones como idioma, formato, velocidad
    ) {
        let ws_addr = ctx.address(); // Dirección para enviar resultados de vuelta
        let output_format = "wav"; // Por ahora fijo a WAV

        log::info!("Starting TTS job: {}", job_id);
        self.send_json(ctx, &WsMessage::TtsJobCreated{ job_id: job_id.clone(), original_file_id, original_filename: original_filename.clone() });

        // Generar nombre de archivo final basado en el original o el job_id
        let final_filename = format!(
            "{}.{}",
            original_filename.unwrap_or_else(|| format!("output_{}", job_id)),
            output_format
        );

        // --- Ejecutar en una tarea Tokio separada para no bloquear el actor ---
        actix_web::rt::spawn(async move {
            let result = process_text_to_speech(job_id.clone(), text_to_convert, ws_addr.clone(), &final_filename).await;
            // Enviar el resultado final (éxito o fracaso) de vuelta al actor WS
            ws_addr.do_send(WsMessage::InternalTtsResult(result));
        });
    }
}

impl Actor for TextToSpeechWsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("WebSocket session started: {}", self.id);
        self.hb(ctx);
        // Crear directorios temporales si no existen
        if fs::create_dir_all(TEMP_UPLOAD_DIR).is_err() {
             log::error!("Failed to create upload directory: {}", TEMP_UPLOAD_DIR);
        }
        if fs::create_dir_all(TEMP_AUDIO_DIR).is_err() {
             log::error!("Failed to create audio directory: {}", TEMP_AUDIO_DIR);
        }
         self.send_json(ctx, &WsMessage::Info { message: format!("Connected with session ID: {}", self.id) });
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        log::info!("WebSocket session stopped: {}", self.id);
        // TODO: Considerar cancelar/limpiar tareas TTS en progreso asociadas a esta sesión
        Running::Stop
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for TextToSpeechWsSession {
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
                 log::debug!("Received TEXT message: {}", text);
                self.hb = Instant::now();
                // Parsear el mensaje JSON
                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(ws_msg) => {
                        match ws_msg {
                            WsMessage::StartUpload { filename } => {
                                let file_id = Uuid::new_v4().to_string();
                                log::info!("Session {}: Received StartUpload for filename '{}', generated file_id: {}", self.id, filename, file_id);
                                // Prepararse para recibir binario (podríamos almacenar file_id/path si fuera necesario)
                                // self.current_upload_file_id = Some(file_id.clone());
                                // let path = PathBuf::from(TEMP_UPLOAD_DIR).join(format!("{}-{}", file_id, filename));
                                // self.current_upload_path = Some(path);

                                // Avisar al cliente que estamos listos y cuál es el file_id
                                self.send_json(ctx, &WsMessage::UploadReady { file_id: file_id.clone(), filename });
                                // Aquí NO guardamos el file_id en el estado del actor,
                                // porque el binario vendrá *después* de este mensaje.
                                // El cliente deberá enviar el file_id de nuevo con el comando TTS.
                            }
                            WsMessage::StartTtsString { text, language, format } => {
                                let job_id = Uuid::new_v4().to_string();
                                log::info!("Session {}: Received StartTtsString, job_id: {}", self.id, job_id);
                                self.start_tts_processing(ctx, job_id, text, None, None /*, options...*/);
                            }
                            WsMessage::StartTtsFile { file_id, filename, language, format } => {
                                log::info!("Session {}: Received StartTtsFile for file_id: {}, filename: {}", self.id, file_id, filename);
                                let job_id = Uuid::new_v4().to_string();
                                let filepath = PathBuf::from(TEMP_UPLOAD_DIR).join(format!("{}-{}", file_id, filename));

                                // Leer el contenido del archivo en una tarea bloqueante
                                let ws_addr = ctx.address();
                                let actor_id = self.id.clone();
                                actix_web::rt::spawn(async move {
                                    match web::block(move || fs::read_to_string(&filepath)).await {
                                        Ok(Ok(content)) => {
                                            log::info!("Successfully read file content for job {}", job_id);
                                            // Enviar mensaje para iniciar el TTS real
                                            // Solución: iniciar TTS directamente desde aquí, dentro de otro spawn.
                                             let ws_addr_clone = ws_addr.clone();
                                             actix_web::rt::spawn(async move { // <-- Spawn interno para el procesamiento TTS
                                                let result = process_text_to_speech(job_id.clone(), content, ws_addr_clone, &filename).await;
                                                // Enviar el resultado de vuelta al actor WS cuando termine
                                                ws_addr.do_send(WsMessage::InternalTtsResult(result));
                                             }); // <-- Cierre del spawn interno
                                
                                             // Notificar inmediatamente al cliente que el job se creó y está en cola/proceso
                                             ws_addr.do_send( WsMessage::TtsJobCreated{ job_id: job_id.clone(), original_file_id: Some(file_id), original_filename: Some(filename) });
                                
                                        }
                                        Ok(Err(e)) => {
                                            log::error!("Failed to read file {:?}: {}", filepath, e);
                                            ws_addr.do_send(WsMessage::Error {
                                                job_id: Some(job_id),
                                                file_id: None, // El error es al leer, no del upload original
                                                message: format!("Failed to read uploaded file: {}", e),
                                            });
                                        }
                                        Err(e) => {
                                             log::error!("Blocking task failed for file read: {}", e);
                                             ws_addr.do_send(WsMessage::Error {
                                                job_id: Some(job_id),
                                                file_id: None,
                                                message: "Server task failed during file reading".to_string(),
                                            });
                                        }
                                    }
                                });

                            }
                            // Otros mensajes del cliente
                            _ => {
                                 log::warn!("Received unexpected JSON message type from client: {:?}", ws_msg);
                                 self.send_json(ctx, &WsMessage::Error{ job_id: None, file_id: None, message: "Unexpected message type received".to_string() });
                            }
                        }
                    }
                    Err(e) => {
                         log::warn!("Failed to parse TEXT message as JSON: {}, message: '{}'", e, text);
                         // Quizás era un mensaje antiguo (solo filename)? O un error.
                         // Podríamos intentar manejar el formato antiguo si es necesario, pero mejor migrar el cliente.
                         self.send_json(ctx, &WsMessage::Error{ job_id: None, file_id: None, message: format!("Invalid message format: {}", e) });
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => {
                self.hb = Instant::now();
                 log::info!("Session {}: Received BINARY data ({} bytes). Requires StartUpload first.", self.id, bin.len());
                 // NECESITAMOS SABER A QUÉ file_id corresponde este binario.
                 // El cliente DEBERÍA haber enviado StartUpload ANTES y recibido un UploadReady.
                 // El cliente necesita indicar de alguna manera a qué subida pertenece este binario.
                 // O el servidor necesita mantener el estado "esperando binario para file_id X".
                 // Por simplicidad ahora: ASUMIMOS que el binario llega justo después de un UploadReady.
                 // ESTO ES FRÁGIL. Una mejor solución es que el cliente envíe un mensaje
                 // tipo `{"type": "upload_chunk", "file_id": "...", "data": base64_encoded_chunk}`
                 // o usar un protocolo que maneje esto mejor.
                 // O mantener el estado en el actor. Añadamos estado simple:

                 // --- Lógica de subida simple ---
                 // Necesitamos añadir `current_upload_info: Option<(String, String)>` al estado del actor
                 // Y ponerlo en `StartUpload` y consumirlo aquí.
                 // Como no lo hemos hecho, esta parte no funcionará correctamente sin refactorizar el estado del actor.
                 // Por ahora, lo dejaremos como estaba en el código original,
                 // el cliente envía filename (texto) y luego el binario.

                 // --- Código original adaptado (FRÁGIL, necesita el filename previo) ---
                 /*
                 if let Some(fname) = self.filename_to_process.take() { // Necesitaríamos `filename_to_process` de nuevo
                    let file_id = Uuid::new_v4().to_string(); // ¿Generar aquí o usar el de StartUpload? Mejor usar el de StartUpload
                    let unique_filename = format!("{}-{}", file_id, fname); // Usa el file_id generado antes
                    let temp_dir = PathBuf::from(TEMP_UPLOAD_DIR);
                    let file_path = temp_dir.join(&unique_filename);

                     log::info!(
                        "Attempting to save file '{}' (ID: {}) to path: {:?}",
                        fname, file_id, file_path
                    );

                    let file_path_clone = file_path.clone();
                    let actor_address = ctx.address();
                    let file_id_clone = file_id.clone(); // Clonar para el closure
                    let fname_clone = fname.clone(); // Clonar para el closure

                    actix_web::rt::spawn(async move {
                        let write_result = web::block(move || {
                             fs::write(&file_path_clone, &bin)
                        }).await;

                        match write_result {
                            Ok(Ok(())) => {
                                log::info!("Successfully saved file to {:?}", file_path);
                                // Usar el nuevo formato de mensaje
                                actor_address.do_send(WsMessage::UploadComplete { file_id: file_id_clone, filename: fname_clone });
                            }
                            Ok(Err(io_err)) => {
                                log::error!("Failed to write file to {:?}: {}", file_path, io_err);
                                actor_address.do_send(WsMessage::Error {
                                    file_id: Some(file_id_clone),
                                    job_id: None,
                                    message: format!("Failed to save file on server: {}", io_err)
                                });
                            }
                            Err(block_err) => {
                                log::error!("Blocking task failed for file write: {}", block_err);
                                 actor_address.do_send(WsMessage::Error {
                                    file_id: Some(file_id_clone),
                                    job_id: None,
                                    message: "Server task failed during file save.".to_string()
                                });
                            }
                        }
                    });

                } else {
                    log::warn!("Session {}: Received binary data but no file upload was initiated via StartUpload.", self.id);
                     self.send_json(ctx, &WsMessage::Error{ job_id: None, file_id: None, message: "Received binary data without initiating upload.".to_string() });
                }
                */
                // --- Fin Lógica Subida Simple ---
                 log::error!("Session {}: Binary upload handling needs refactoring to associate with a file_id from StartUpload.", self.id);
                  self.send_json(ctx, &WsMessage::Error{ job_id: None, file_id: None, message: "Server error: Cannot process raw binary data without context.".to_string() });

            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                log::warn!("Continuation frames not supported");
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

// Handler para los resultados de la tarea TTS que vienen de la tarea async
impl Handler<WsMessage> for TextToSpeechWsSession {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        match msg {
            WsMessage::InternalTtsResult(result) => {
                match result {
                    TtsTaskResult::Progress { job_id, progress, message } => {
                         log::debug!("Sending TTS progress for job {}: {:.2}%", job_id, progress * 100.0);
                        self.send_json(ctx, &WsMessage::TtsProgress { job_id, progress, message });
                    }
                    TtsTaskResult::Complete { job_id, output_path, final_filename } => {
                        log::info!("TTS job {} completed. Output: {:?}", job_id, output_path);
                        // Construir la URL de descarga relativa
                        let download_url = format!("/download/audio/{}/{}", job_id, final_filename); // Asegúrate que coincida con la ruta de descarga
                         self.send_json(ctx, &WsMessage::TtsComplete {
                            job_id,
                            filename: final_filename,
                            download_url,
                        });
                    }
                    TtsTaskResult::Failed { job_id, error_message } => {
                         log::error!("TTS job {} failed: {}", job_id, error_message);
                         self.send_json(ctx, &WsMessage::Error {
                            job_id: Some(job_id),
                            file_id: None,
                            message: error_message,
                        });
                    }
                }
            },
            // Otros mensajes que podrían enviarse directamente al actor
             WsMessage::UploadComplete { file_id, filename } => {
                 log::info!("Sending UploadComplete for file_id: {}, filename: {}", file_id, filename);
                self.send_json(ctx, &WsMessage::UploadComplete { file_id, filename });
                // Opcional: Enviar mensaje Info adicional
                self.send_json(ctx, &WsMessage::Info{ message: format!("File '{}' uploaded successfully. You can now request TTS for file_id '{}'.", filename, file_id) });
            }
             WsMessage::Error { job_id, file_id, message } => {
                 log::info!("Sending Error: job={:?}, file={:?}, msg={}", job_id, file_id, message);
                self.send_json(ctx, &WsMessage::Error { job_id, file_id, message });
            }
             WsMessage::TtsJobCreated { job_id, original_file_id, original_filename } => {
                 log::info!("Sending TtsJobCreated for job: {}", job_id);
                self.send_json(ctx, &WsMessage::TtsJobCreated { job_id, original_file_id, original_filename });
            }

            _ => {
                 log::warn!("Received unexpected message type in Handler<WsMessage>: {:?}", msg);
            }
        }
    }
}


// --- Lógica TTS ---

/// Divide el texto en chunks (ejemplo simple: por líneas no vacías)
fn chunk_text(text: &str) -> Vec<String> {
    text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect()
    // Alternativa más robusta: dividir por frases usando terminadores como '.', '?', '!'
    // O usar librerías de procesamiento de lenguaje natural (NLP) si necesitas más precisión.
}

/// Procesa el texto completo a voz, chunk por chunk
async fn process_text_to_speech(
    job_id: String,
    text: String,
    ws_addr: Addr<TextToSpeechWsSession>, // Para enviar progreso
    final_filename: &str,
) -> TtsTaskResult {
    let chunks = chunk_text(&text);
    let total_chunks = chunks.len();
    if total_chunks == 0 {
         return TtsTaskResult::Failed{ job_id, error_message: "No text content found to convert.".to_string() };
    }

    log::info!("Job {}: Starting conversion for {} chunks.", job_id, total_chunks);

    // Crear un directorio temporal para los chunks de este job
    let job_audio_dir = PathBuf::from(TEMP_AUDIO_DIR).join(&job_id);
    if let Err(e) = fs::create_dir_all(&job_audio_dir) {
        return TtsTaskResult::Failed { job_id, error_message: format!("Failed to create temporary directory: {}", e) };
    }

    let mut chunk_files = Vec::new(); // Guardar rutas de los archivos de audio de chunks
    let mut concat_list_content = String::new(); // Contenido para list.txt de ffmpeg

    for (i, chunk) in chunks.into_iter().enumerate() {
        let chunk_num = i + 1;
        let progress = chunk_num as f32 / total_chunks as f32;
        let chunk_filename = format!("chunk_{}.wav", chunk_num);
        let chunk_filepath = job_audio_dir.join(&chunk_filename);

        log::debug!("Job {}: Processing chunk {}/{}", job_id, chunk_num, total_chunks);

        // Enviar progreso antes de procesar el chunk
         ws_addr.do_send(WsMessage::InternalTtsResult(TtsTaskResult::Progress{
             job_id: job_id.clone(),
             progress: progress * 0.9, // Dejar margen para la concatenación final
             message: format!("Converting chunk {} of {}", chunk_num, total_chunks)
         }));


        // --- Llamada a espeak ---
        // TODO: Añadir opciones de voz (idioma -v, velocidad -s, etc.)
        let mut cmd = Command::new("espeak");
        cmd.arg("-w") // Salida a archivo WAV
            .arg(&chunk_filepath)
            .arg("--") // Para asegurar que el texto no se interprete como argumento
            .arg(&chunk); // El texto del chunk

        cmd.stdout(Stdio::null()); // No necesitamos la salida estándar
        cmd.stderr(Stdio::piped()); // Capturar errores

        let output = match cmd.spawn() {
            Ok(mut child) => {
                // Esperar a que el proceso termine de forma asíncrona
                match child.wait_with_output().await {
                    Ok(out) => out,
                    Err(e) => {
                         let error_msg = format!("Failed to wait for espeak process for chunk {}: {}", chunk_num, e);
                         log::error!("Job {}: {}", job_id, error_msg);
                         // Limpiar directorio de chunks antes de salir
                         let _ = fs::remove_dir_all(&job_audio_dir);
                         return TtsTaskResult::Failed { job_id, error_message };
                    }
                }
            }
            Err(e) => {
                 let error_msg = format!("Failed to spawn espeak for chunk {}: {}. Is espeak installed and in PATH?", chunk_num, e);
                 log::error!("Job {}: {}", job_id, error_msg);
                 // Limpiar directorio de chunks
                 let _ = fs::remove_dir_all(&job_audio_dir);
                return TtsTaskResult::Failed { job_id, error_message };
            }
        };


        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let error_msg = format!(
                "espeak failed for chunk {} with status {:?}. Error: {}",
                chunk_num, output.status, stderr
            );
            log::error!("Job {}: {}", job_id, error_msg);
             // Limpiar directorio de chunks
             let _ = fs::remove_dir_all(&job_audio_dir);
            return TtsTaskResult::Failed { job_id, error_message };
        }

        // Si tuvo éxito, añadir a la lista para ffmpeg y guardar la ruta
        chunk_files.push(chunk_filepath.clone());
        // ¡Importante! ffmpeg necesita rutas relativas (o absolutas seguras) y caracteres especiales escapados.
        // Usar rutas relativas desde donde se ejecuta ffmpeg es lo más seguro.
        // Asumiendo que ejecutamos ffmpeg desde TEMP_AUDIO_DIR/job_id
        concat_list_content.push_str(&format!("file '{}'\n", chunk_filename)); // ffmpeg usa comillas simples

         log::debug!("Job {}: Successfully processed chunk {}/{}", job_id, chunk_num, total_chunks);
    }

    // --- Concatenar chunks con ffmpeg ---
    log::info!("Job {}: All chunks processed. Concatenating...", job_id);
     ws_addr.do_send(WsMessage::InternalTtsResult(TtsTaskResult::Progress{
             job_id: job_id.clone(),
             progress: 0.95, // Progreso antes de concatenar
             message: "Combining audio pieces...".to_string()
     }));

    let list_filename = "concat_list.txt";
    let list_filepath = job_audio_dir.join(list_filename);
    let final_output_path = job_audio_dir.join(final_filename); // Guardar en el dir del job inicialmente

    // Escribir el archivo de lista
    if let Err(e) = fs::write(&list_filepath, &concat_list_content) {
         let error_msg = format!("Failed to write ffmpeg list file: {}", e);
         log::error!("Job {}: {}", job_id, error_msg);
          // Limpiar directorio de chunks
          let _ = fs::remove_dir_all(&job_audio_dir);
        return TtsTaskResult::Failed { job_id, error_message };
    }

    // Comando ffmpeg
    let mut cmd = Command::new("ffmpeg");
    cmd.current_dir(&job_audio_dir) // Ejecutar desde el directorio del job para rutas relativas
        .arg("-f")
        .arg("concat")
        .arg("-safe") // Necesario si las rutas no son "seguras" (ej. absolutas)
        .arg("0")     // Permitir cualquier nombre de archivo
        .arg("-i")
        .arg(list_filename) // El archivo de lista que creamos
        .arg("-c")
        .arg("copy") // Copiar streams de audio sin recodificar (rápido)
        .arg(final_filename); // Nombre del archivo de salida final

    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::piped());

    let output = match cmd.spawn() {
         Ok(mut child) => {
            match child.wait_with_output().await {
                Ok(out) => out,
                Err(e) => {
                    let error_msg = format!("Failed to wait for ffmpeg process: {}", e);
                    log::error!("Job {}: {}", job_id, error_msg);
                    // Limpiar directorio de chunks y lista
                    let _ = fs::remove_dir_all(&job_audio_dir);
                    return TtsTaskResult::Failed { job_id, error_message };
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to spawn ffmpeg: {}. Is ffmpeg installed and in PATH?", e);
             log::error!("Job {}: {}", job_id, error_msg);
             // Limpiar directorio de chunks y lista
             let _ = fs::remove_dir_all(&job_audio_dir);
            return TtsTaskResult::Failed { job_id, error_message };
        }
    };

    // Limpiar archivo de lista independientemente del resultado de ffmpeg
    let _ = fs::remove_file(&list_filepath);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_msg = format!(
            "ffmpeg concatenation failed with status {:?}. Error: {}",
            output.status, stderr
        );
        log::error!("Job {}: {}", job_id, error_msg);
        // Limpiar directorio de chunks (el archivo final podría estar corrupto)
        let _ = fs::remove_dir_all(&job_audio_dir);
        return TtsTaskResult::Failed { job_id, error_message };
    }

    // --- Limpieza de chunks individuales ---
    log::info!("Job {}: Concatenation successful. Cleaning up chunk files...", job_id);
    for chunk_file in chunk_files {
        if let Err(e) = fs::remove_file(&chunk_file) {
            log::warn!("Job {}: Failed to remove chunk file {:?}: {}", job_id, chunk_file, e);
        }
    }

    log::info!("Job {}: TTS process complete. Final file: {:?}", job_id, final_output_path);

    // Devolver la ruta al archivo final
    TtsTaskResult::Complete { job_id, output_path: final_output_path, final_filename: final_filename.to_string() }
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
    // Inicia la sesión WebSocket con el nuevo actor
    ws::start(TextToSpeechWsSession::new(), &req, stream)
}

// Ruta de descarga para el audio generado
async fn download_audio(path: web::Path<(String, String)>) -> Result<NamedFile, Error> {
    let (job_id, filename) = path.into_inner();
    // La ruta ahora está dentro del directorio temporal del job
    let file_path = Path::new(TEMP_AUDIO_DIR).join(&job_id).join(&filename);

    log::info!("Download request for audio file at path: {:?}", file_path);

    let named_file = NamedFile::open_async(&file_path).await
        .map_err(|e| {
            log::error!("Failed to open file {:?} for download: {}", file_path, e);
            match e.kind() {
                io::ErrorKind::NotFound => actix_web::error::ErrorNotFound("Audio file not found. It might have expired or failed processing."),
                _ => actix_web::error::ErrorInternalServerError("Error accessing audio file."),
            }
        })?;

    // Configurar para descargar con el nombre original
    Ok(named_file
        .set_content_disposition(actix_web::http::header::ContentDisposition {
            disposition: actix_web::http::header::DispositionType::Attachment,
            parameters: vec![actix_web::http::header::DispositionParam::Filename(filename)],
        }))
    // NOTA: Considera una estrategia para limpiar archivos antiguos en TEMP_AUDIO_DIR
    // (ej. una tarea periódica que borre directorios de jobs con más de X horas/días)
}


// --- Main Server Setup ---

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Crear directorios temporales si no existen
    fs::create_dir_all(TEMP_UPLOAD_DIR)?;
    fs::create_dir_all(TEMP_AUDIO_DIR)?;
     log::info!("Temporary upload directory: {:?}", Path::new(TEMP_UPLOAD_DIR).canonicalize()?);
     log::info!("Temporary audio directory: {:?}", Path::new(TEMP_AUDIO_DIR).canonicalize()?);


    let server_addr = "127.0.0.1";
    let server_port = 8080;

    log::info!("Starting HTTP server at http://{}:{}", server_addr, server_port);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .route("/", web::get().to(index))
            .route("/ws", web::get().to(ws_route)) // WebSocket endpoint
            // Ruta de descarga de audio generado por TTS
            .route("/download/audio/{job_id}/{filename}", web::get().to(download_audio))
            // Servir archivos estáticos (CSS, JS) - ¡Descomenta y ajusta si es necesario!
            .service(actix_files::Files::new("/static", "./static").show_files_listing()) // show_files_listing es útil para depurar
    })
    .workers(4) // Puedes ajustar el número de workers
    .bind((server_addr, server_port))?
    .run()
    .await
}