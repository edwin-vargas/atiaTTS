use rusqlite::{
    //params, 
    Connection, Result};
//use std::fs;
use std::path::Path;
//use uuid::Uuid;

// pub struct User {
//     pub user_id: String,
//     pub user_name: String,  
//     pub user_email: String,
//     pub user_pass: String,
// }

// Ensure the database exists, if not create it with all required tables
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
