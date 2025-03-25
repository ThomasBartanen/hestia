use rusqlite::{self, Connection, Error};
use std::path::Path;

use crate::models::Property;

pub const DATABASE_NAME: &str = "test_db.db3";

pub struct DatabaseManager {
    pub conn: Connection,
}
impl DatabaseManager {
    pub fn new(db_path: &str) -> Result<Self, Error> {
        let conn = Connection::open(format!("{db_path}test_db.db3"))?;
        
        Self::initialize_schema(&conn)?;
        
        Ok(Self { conn })
    }

    fn initialize_schema(conn: &Connection) -> Result<(), Error> {
        conn.execute_batch("
            BEGIN TRANSACTION;
            
            CREATE TABLE IF NOT EXISTS properties (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                address TEXT NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS units (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                property_id INTEGER,
                unit_number TEXT NOT NULL,
                is_occupied BOOLEAN DEFAULT FALSE,
                FOREIGN KEY (property_id) REFERENCES properties(id)
            );
            
            CREATE TABLE IF NOT EXISTS tenants (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                unit_id INTEGER,
                name TEXT NOT NULL,
                email TEXT,
                phone TEXT
            );
            
            CREATE TABLE IF NOT EXISTS expenses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                amount REAL NOT NULL,
                description TEXT,
                date TEXT
            );
            
            COMMIT;
        ")?;
        
        Ok(())
    }

    pub fn insert_property(&self, property: Property) -> Result<i64, Error> {
        self.conn.execute(
            "INSERT INTO properties (name, address) VALUES (?, ?)",
            &[&property.name, &property.address],
        )?;
        
        Ok(self.conn.last_insert_rowid())
    }

    pub fn assign_tenant_to_unit(&self, tenant_id: i64, unit_id: i64) -> Result<(), Error> {
        self.conn.execute(
            "INSERT OR REPLACE INTO tenants (unit_id)
             VALUES (?) WHERE id = ?",
            &[&unit_id, &tenant_id],
        )?;
        
        self.conn.execute(
            "UPDATE units SET is_occupied = TRUE WHERE id = ?",
            &[&unit_id],
        )?;
        
        Ok(())
    }
}

