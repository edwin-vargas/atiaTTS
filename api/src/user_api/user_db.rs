use rusqlite::{params, Connection, Result};
//use std::path::Path;
use uuid::Uuid;

#[derive()]
pub struct User {
    pub user_id: String,
    pub user_name: String,
    pub user_email: String,
    pub user_pass: String,
}

pub fn insert_user(user_name: &str, user_email: &str, user_pass: &str) -> Result<String> {
    let conn = Connection::open("app.db")?;
    let user_id = Uuid::new_v4().to_string();
    
    conn.execute(
        "
            INSERT INTO users 
            (user_id, user_name, user_email, user_pass) 
            VALUES (?1, ?2, ?3, ?4)
        ",
        params![
            user_id, 
            user_name, 
            user_email, 
            user_pass
        ],
    )?;
    
    Ok(user_id)
}

pub fn get_user_by_id(user_id: &str) -> Result<User> {
    let conn = Connection::open("app.db")?;
    
    let mut stmt = conn.prepare("
        SELECT user_id, user_name, user_email, user_pass 
        FROM users 
        WHERE user_id = ?1
    ")?;
    
    let user = stmt.query_row(params![user_id], |row| {
        Ok(User {
            user_id: row.get(0)?,
            user_name: row.get(1)?,
            user_email: row.get(2)?,
            user_pass: row.get(3)?,
        })
    })?;
    
    Ok(user)
}

pub fn get_user_by_email_pass(user_email: &str, user_pass: &str) -> Result<User> {
    let conn = Connection::open("app.db")?;
    
    let mut stmt = conn.prepare("
        SELECT user_id, user_name, user_email, user_pass 
        FROM users 
        WHERE user_email = ?1 
        AND user_pass = ?2
    ")?;

    let user = stmt.query_row(params![user_email, user_pass], |row| {
        Ok(User {
            user_id: row.get(0)?,
            user_name: row.get(1)?,
            user_email: row.get(2)?,
            user_pass: row.get(3)?,
        })
    })?;
    
    Ok(user)
}

pub fn update_user(user_id: &str, user_name: &str, user_email: &str, user_pass: &str) -> Result<()> {
    let mut conn = Connection::open("app.db")?;
    
    conn.execute(
        "
            UPDATE users 
            SET user_name = ?1, user_email = ?2, user_pass = ?3 
            WHERE user_id = ?4
        ",
        params![
            user_name, 
            user_email, 
            user_pass, 
            user_id
        ],
    )?;
    
    Ok(())
}

pub fn delete_user(user_id: &str) -> Result<()> {
    let mut conn = Connection::open("app.db")?;
    
    let tx = conn.transaction()?;
    
    tx.execute("DELETE FROM pro WHERE user_id = ?1", params![user_id])?;
    tx.execute("DELETE FROM plus WHERE user_id = ?1", params![user_id])?;
    tx.execute("DELETE FROM users WHERE user_id = ?1", params![user_id])?;
    
    tx.commit()?;
    
    Ok(())
}

pub fn add_user_to_plan(user_id: &str, plan: i32) -> Result<()> {
    let mut conn = Connection::open("app.db")?;
    
    {
        let mut stmt = conn.prepare("SELECT 1 FROM users WHERE user_id = ?1")?;
        let user_exists = stmt.exists(params![user_id])?;
        
        if !user_exists {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
    } // stmt is dropped here, releasing the immutable borrow on conn
    
    // Now we can create a transaction
    let tx = conn.transaction()?;
    
    // Remove from any existing plans first
    tx.execute("DELETE FROM plus WHERE user_id = ?1", params![user_id])?;
    tx.execute("DELETE FROM pro WHERE user_id = ?1", params![user_id])?;
    
    // Add to the appropriate plan
    match plan {
        1 => tx.execute("INSERT INTO plus (user_id) VALUES (?1)", params![user_id])?,
        2 => tx.execute("INSERT INTO pro (user_id) VALUES (?1)", params![user_id])?,
        _ => return Err(rusqlite::Error::InvalidParameterName("Invalid plan type".to_string())),
    };
    
    // Commit transaction
    tx.commit()?;
    
    Ok(())
}
