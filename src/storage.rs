use crate::state::PacketMetadata;
use rusqlite::{params, Connection, Result};
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::time::{interval, Duration};

#[derive(Clone)]
pub struct Storage {
    conn: Arc<std::sync::Mutex<Connection>>,
}

impl Storage {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        // Enable WAL mode for concurrency (PRAGMA returns a result, so use query_row)
        let _: String = conn.query_row("PRAGMA journal_mode=WAL;", [], |row| row.get(0))?;
        conn.execute_batch("PRAGMA synchronous=NORMAL;")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS packets (
                id INTEGER PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                src_ip TEXT NOT NULL,
                dst_ip TEXT NOT NULL,
                src_port INTEGER,
                dst_port INTEGER,
                protocol TEXT,
                length INTEGER
            )",
            [],
        )?;
        
        conn.execute(
             "CREATE INDEX IF NOT EXISTS idx_timestamp ON packets(timestamp)",
             []
        )?;

        Ok(Self {
            conn: Arc::new(std::sync::Mutex::new(conn)),
        })
    }

    pub async fn run_writer(&self, mut rx: Receiver<PacketMetadata>) {
        let mut buffer = Vec::new();
        let mut ticker = interval(Duration::from_secs(2));

        loop {
            tokio::select! {
                Some(packet) = rx.recv() => {
                    buffer.push(packet);
                    if buffer.len() >= 1000 {
                         self.flush(&mut buffer);
                    }
                }
                _ = ticker.tick() => {
                    if !buffer.is_empty() {
                        self.flush(&mut buffer);
                    }
                }
            }
        }
    }

    fn flush(&self, buffer: &mut Vec<PacketMetadata>) {
         let mut conn = self.conn.lock().unwrap();
         let tx = match conn.transaction() {
             Ok(tx) => tx,
             Err(e) => {
                 eprintln!("Failed to start transaction: {}", e);
                 return;
             }
         };

         {
             let mut stmt = match tx.prepare(
                 "INSERT INTO packets (timestamp, src_ip, dst_ip, src_port, dst_port, protocol, length)
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
             ) {
                 Ok(stmt) => stmt,
                 Err(e) => {
                     eprintln!("Failed to prepare statement: {}", e);
                     return;
                 }
             };

             for packet in buffer.iter() {
                 if let Err(e) = stmt.execute(params![
                     packet.timestamp,
                     packet.src_ip,
                     packet.dst_ip,
                     packet.src_port,
                     packet.dst_port,
                     packet.protocol,
                     packet.length
                 ]) {
                     eprintln!("Failed to insert packet: {}", e);
                 }
             }
         } // stmt dropped here

         if let Err(e) = tx.commit() {
             eprintln!("Failed to commit transaction: {}", e);
         } else {
             buffer.clear();
         }
    }
    
    pub fn query_history(&self, limit: usize) -> Result<Vec<PacketMetadata>> {
         let conn = self.conn.lock().unwrap();
         let mut stmt = conn.prepare(
             "SELECT timestamp, src_ip, dst_ip, src_port, dst_port, protocol, length 
              FROM packets ORDER BY timestamp DESC LIMIT ?1"
         )?;
         
         let rows = stmt.query_map([limit], |row| {
             Ok(PacketMetadata {
                 timestamp: row.get(0)?,
                 src_ip: row.get(1)?,
                 dst_ip: row.get(2)?,
                 src_port: row.get(3)?,
                 dst_port: row.get(4)?,
                 protocol: row.get(5)?,
                 length: row.get(6)?,
             })
         })?;
         
         let mut result = Vec::new();
         for row in rows {
             result.push(row?);
         }
         Ok(result)
    }
}
