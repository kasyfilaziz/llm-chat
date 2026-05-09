use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

impl Db {
    pub fn new(path: &str) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn get_conn(&self) -> Arc<Mutex<Connection>> {
        self.conn.clone()
    }
}
