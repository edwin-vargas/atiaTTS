use actix_multipart::Multipart;
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use actix_web::{web, App, Error as ActixError, HttpResponse, HttpServer, Responder};
use bytes::Bytes;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::fs as std_fs;
use std::io::Write; // Required for write_all
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;

// --- Configuration ---
const UPLOAD_DIR: &str = "./uploads"; // Define your upload directory

// --- Endpoint for Multiple File Upload ---

pub async fn upload_multiple_files(mut payload: Multipart) -> Result<HttpResponse, ActixError> {

    std_fs::create_dir_all(UPLOAD_DIR)?;

    let mut file_count = 0;
    let mut saved_files = Vec::new();

    // Ensure upload directory exists
    std_fs::create_dir_all(UPLOAD_DIR).map_err(ActixError::from)?;

    while let Some(mut field) = payload.try_next().await? {
        let content_disposition = field.content_disposition();
        let raw_filename = content_disposition
            .get_filename()
            .map_or_else(|| Uuid::new_v4().to_string(), |name| name.to_string()); // Use UUID if no filename

        // Sanitize the filename to prevent path traversal and invalid characters
        let filename = sanitize_filename::sanitize(&raw_filename);

        if filename.is_empty() || filename == "." || filename == ".." {
             eprintln!("Skipping field with invalid sanitized filename: {}", raw_filename);
             continue; // Skip potentially dangerous or empty filenames after sanitization
        }

        println!("Processing file: {}", filename);

        let file_path = PathBuf::from(UPLOAD_DIR).join(&filename);

        // Prevent overwriting existing files (optional, consider adding unique prefix/suffix)
        // if file_path.exists() {
        //     return Err(actix_web::error::ErrorInternalServerError(format!("File already exists: {}", filename)));
        // }

        let file_path_clone = file_path.clone(); // Clone for the blocking closure
        let mut f = web::block(move || std_fs::File::create(file_path_clone))
            .await??; // Use web::block for blocking I/O

        while let Some(chunk) = field.try_next().await? {
            f = web::block(move || -> Result<std_fs::File, std::io::Error> {
                f.write_all(&chunk)?;
                Ok(f) // Return the file handle for the next iteration
            })
            .await??; // Use web::block for blocking I/O
        }

        saved_files.push(filename);
        file_count += 1;
    }

    if file_count > 0 {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Successfully uploaded {} files.", file_count),
            "files": saved_files
        })))
    } else {
        Ok(HttpResponse::BadRequest().body("No files were uploaded or processed."))
    }
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

    let file_content = std_fs::read(&file_name).unwrap();

    
    let mut response = HttpResponse::Ok();
    response.content_type("audio/wav")
        .append_header(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![DispositionParam::Filename(String::from("speech.wav"))],
        });    
    let response_body = response.body(file_content);
    let _ = std_fs::remove_file(&file_name);
    response_body
}
