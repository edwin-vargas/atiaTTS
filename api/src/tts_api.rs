
use actix_web::{
    web, 
    HttpResponse, 
    Responder, 
    Error
};
use actix_web::http::header::{
    ContentDisposition, 
    DispositionType, 
    DispositionParam
};
use actix_multipart::Multipart;
use futures::StreamExt;
use std::fs as std_fs;
use std::process::Command;
use uuid::Uuid;

mod tts_mod;
use tts_mod::{
    TtsRequest, 
    PlanRequest, 
    ErrorResponse
};
mod tts_db;



// Text > WAV
pub async fn text_to_speech(data: web::Json<TtsRequest>) -> impl Responder {
    // Verify user has PLUS or PRO plan
    let plan = match tts_db::get_user_plan_type(&data.user_id) {
        Ok(plan) => plan,
        Err(_) => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: "User not found".to_string(),
            });
        }
    };
    
    if plan != "PLUS" && plan != "PRO" {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "This feature requires PLUS or PRO plan".to_string(),
        });
    }
    
    // Generate a unique filename
    let file_id = Uuid::new_v4().to_string();
    let output_file = format!("{}.wav", file_id);
    
    // Run espeak command to convert text to speech
    let status = Command::new("espeak")
        .arg(&data.text)
        .arg("-w")
        .arg(&output_file)
        .status();
        
    match status {
        Ok(_) => {
            // Read the generated file
            match std_fs::read(&output_file) {
                Ok(file_content) => {
                    // Create response with WAV file
                    let mut response = HttpResponse::Ok();
                    
                    // Set appropriate headers
                    response.content_type("audio/wav")
                        .append_header(ContentDisposition {
                            disposition: DispositionType::Attachment,
                            parameters: vec![DispositionParam::Filename(String::from("speech.wav"))],
                        });
                    
                    // Delete file after reading
                    let response_body = response.body(file_content);
                    let _ = std_fs::remove_file(&output_file); // Ignore deletion errors
                    
                    response_body
                }
                Err(e) => {
                    // Clean up on error
                    let _ = std_fs::remove_file(&output_file);
                    HttpResponse::InternalServerError().json(ErrorResponse {
                        error: format!("Failed to read audio file: {}", e),
                    })
                }
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Text to speech conversion failed: {}", e),
            })
        }
    }
}

// Process uploaded text files for PRO users and convert to speech
pub async fn process_files(mut payload: Multipart, user_id: String) -> Result<HttpResponse, Error> {
    // Verify user has PRO plan
    let plan = match tts_db::get_user_plan_type(&user_id) {
        Ok(plan) => plan,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(ErrorResponse {
                error: "User not found".to_string(),
            }));
        }
    };
    
    if plan != "PRO" {
        return Ok(HttpResponse::Forbidden().json(ErrorResponse {
            error: "This feature requires PRO plan".to_string(),
        }));
    }
    
    let mut file_count = 0;
    let mut audio_files = Vec::new();
    
    // Process each uploaded file
    while let Some(item) = payload.next().await {
        let mut field = item?;
        
        // Limit to 5 files
        if file_count >= 5 {
            break;
        }
        
        // Get filename from field
        let content_disposition = field.content_disposition().clone();
        let filename = content_disposition
            .get_filename()
            .unwrap_or("unnamed.txt")
            .to_string();
        
        // Read file content
        let mut content = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            content.extend_from_slice(&data);
        }
        
        // Convert content to string
        if let Ok(text) = String::from_utf8(content) {
            // Generate unique filename for audio
            let output_filename = format!("{}-{}", Uuid::new_v4(), filename.replace(".txt", ".wav"));
            
            // Run espeak command to convert text to speech
            let status = Command::new("espeak")
                .arg(&text)
                .arg("-w")
                .arg(&output_filename)
                .status();
                
            if let Ok(_) = status {
                // Read the generated audio file
                if let Ok(audio_content) = std_fs::read(&output_filename) {
                    audio_files.push((output_filename.clone(), audio_content, filename));
                }
            }
        }
        
        file_count += 1;
    }
    
    // If no files were processed successfully
    if audio_files.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "Failed to process any files".to_string(),
        }));
    }
    
    // Create a multipart response
    let boundary = "multipartboundary";
    let mut response_body = Vec::new();
    
    for (output_filename, audio_content, original_filename) in &audio_files {
        // Add multipart headers
        let header = format!(
            "--{}\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Type: audio/wav\r\n\r\n",
            boundary,
            original_filename.replace(".txt", ".wav")
        );
        
        // Add header and content to response body
        response_body.extend_from_slice(header.as_bytes());
        response_body.extend_from_slice(audio_content);
        response_body.extend_from_slice(b"\r\n");
        
        // Clean up temporary file
        let _ = std_fs::remove_file(output_filename);
    }
    
    // Add closing boundary
    let closing = format!("--{}--\r\n", boundary);
    response_body.extend_from_slice(closing.as_bytes());
    
    // Return response
    Ok(HttpResponse::Ok()
        .content_type(format!("multipart/mixed; boundary={}", boundary))
        .body(response_body))
}

// Handle multipart form data for file uploads
pub async fn upload_files(payload: Multipart, query: web::Query<PlanRequest>) -> Result<HttpResponse, Error> {
    process_files(payload, query.user_id.clone()).await
}
