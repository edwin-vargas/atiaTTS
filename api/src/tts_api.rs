use actix_web::{rt, web, Error, HttpRequest, HttpResponse, Responder};
use actix_ws::AggregatedMessage;
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Deserialize, Serialize, Debug)]
pub struct ProTTSMessage {
    pub text: String,
    pub voice: Option<String>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PlusTTS {
    pub text: String
}
pub async fn pro_tts(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;
    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));
    
    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                
                Ok(AggregatedMessage::Text(text)) => {
                    match serde_json::from_str::<ProTTSMessage>(&text) {
                        Ok(tts_request) => {
                            let voice = tts_request.voice.unwrap_or("default".to_string());
                            let (file_name, status) = espeak_pro(&tts_request.text, &voice);
                            match status {
                                Ok(_) => {
                                    
                                    match fs::read(&file_name) {
                                        Ok(file_content) => {
                                            let _ = session.binary(file_content).await;
                                            let _ = fs::remove_file(&file_name);
                                        },
                                        Err(e) => {
                                            let error_msg = format!("Error al leer el archivo: {}", e);
                                            let _ = session.text(error_msg).await;
                                        }
                                    }
                                },
                                Err(e) => {
                                    let error_msg = format!("Error al ejecutar espeak: {}", e);
                                    let _ = session.text(error_msg).await;
                                }
                            }
                        },
                        Err(e) => {
                            let error_msg = format!("Error al procesar el mensaje: {}", e);
                            let _ = session.text(error_msg).await;
                        }
                    }
                },
                Ok(AggregatedMessage::Ping(msg)) => {
                    let _ = session.pong(&msg).await;
                },
                Ok(AggregatedMessage::Close(_)) => {
                    break;
                },
                _ => {}
            }
        }
    });
    
    Ok(res)
}

fn espeak_pro(text: &str, voice: &str) -> (String, std::io::Result<std::process::ExitStatus>) {
    let file_id = Uuid::new_v4().to_string();
    let file_name = format!("{}.wav", file_id);
    let file_name_clone = file_name.clone();
    
    // Create thread-safe reference to status result
    let status_result = Arc::new(Mutex::new(None));
    let status_result_clone = Arc::clone(&status_result);
    
    // Spawn a separate thread to run the TTS conversion
    let text_owned = text.to_string();
    let voice_owned = voice.to_string();
    
    let handle = thread::spawn(move || {
        let status = Command::new("espeak")
            .arg("-v")
            .arg(&voice_owned)
            .arg("-s")
            .arg("80")
            .arg("-p")
            .arg("50")
            .arg(&text_owned)
            .arg("-w")
            .arg(&file_name_clone)
            .status();
            
        // Store the result in the shared state
        let mut result = status_result_clone.lock().unwrap();
        *result = Some(status);
    });
    
    // Wait for the thread to complete
    handle.join().unwrap();
    
    // Extract the result
    let result = status_result.lock().unwrap().take().unwrap();
    
    (file_name, result)
}

pub async fn plus_tts(req: web::Json<PlusTTS>) -> impl Responder {
    let file_id = Uuid::new_v4().to_string();
    let file_name = format!("{}.wav", file_id);
    
    let _ = Command::new("espeak")
        .arg(&req.text)
        .arg("-w")
        .arg(&file_name)
        .status(); 

    let file_content = fs::read(&file_name).unwrap();

    
    let mut response = HttpResponse::Ok();
    response.content_type("audio/wav")
        .append_header(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![DispositionParam::Filename(String::from("speech.wav"))],
        });    
    let response_body = response.body(file_content);
    let _ = fs::remove_file(&file_name);
    response_body
}