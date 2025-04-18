use rusqlite::{
    params, 
    Connection, 
    Result
};
use std::path::Path;

pub struct User {
    pub user_id: String,
    pub user_name: String,
    pub user_email: String,
    pub user_pass: String,
}

pub fn ensure_db_exists() -> Result<Connection> {
    let db_path = "app.db";
    let db_exists = Path::new(db_path).exists();
    
    let conn = Connection::open(db_path)?;
    
    if !db_exists {
        // Create Users table
        conn.execute(
            "CREATE TABLE users (
                user_id TEXT PRIMARY KEY,
                user_name TEXT NOT NULL,
                user_email TEXT NOT NULL UNIQUE,
                user_pass TEXT NOT NULL
            )",
            [],
        )?;

        // Create PRO table
        conn.execute(
            "CREATE TABLE pro (
                user_id TEXT PRIMARY KEY,
                FOREIGN KEY (user_id) REFERENCES users (user_id)
            )",
            [],
        )?;

        // Create PLUS table
        conn.execute(
            "CREATE TABLE plus (
                user_id TEXT PRIMARY KEY,
                FOREIGN KEY (user_id) REFERENCES users (user_id)
            )",
            [],
        )?;
    }
    
    Ok(conn)
}

pub fn get_user_plan_type(user_id: &str) -> Result<String> {
    let conn = ensure_db_exists()?;
    
    let mut stmt = conn.prepare("SELECT 1 FROM pro WHERE user_id = ?1")?;
    let is_pro = stmt.exists(params![user_id])?;
    
    if is_pro {
        return Ok("PRO".to_string());
    }
    
    let mut stmt = conn.prepare("SELECT 1 FROM plus WHERE user_id = ?1")?;
    let is_plus = stmt.exists(params![user_id])?;
    
    if is_plus {
        return Ok("PLUS".to_string());
    }
    
    Ok("FREE".to_string())
}
