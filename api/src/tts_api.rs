use actix_web::{rt, web, Error, HttpRequest, HttpResponse, Responder};
use actix_ws::AggregatedMessage;
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct ProTTSMessage {
    pub text: String,
    pub voice: Option<String>
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
                            let file_id = Uuid::new_v4().to_string();
                            let file_name = format!("{}.wav", file_id);
                            let voice = tts_request.voice.unwrap_or("default".to_string());
                            let status = Command::new("espeak")
                                .arg("-v")
                                .arg(&voice)
                                .arg("-s")
                                .arg("130") // Velocidad
                                .arg("-p")
                                .arg("50")  // Tono
                                .arg(&tts_request.text)
                                .arg("-w")
                                .arg(&file_name)
                                .status();
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

#[derive(Deserialize, Serialize, Debug)]
pub struct PlusTTS {
    pub text: String
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