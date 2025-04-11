use std::{path::Path, sync::Arc};
use sqlx::{postgres::PgPoolOptions, Connection, Error};
use tokio::sync::{mpsc::{UnboundedReceiver, UnboundedSender}, Mutex};


use crate::{models::{Property, Tenant}, AppState};

pub const DATABASE_NAME: &str = "test_db.db3";

use sqlx::{PgPool, Pool, Postgres};
use std::time::Duration;

#[derive(Debug)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl DatabaseConfig {
    pub fn new(url: String, max_connections: u32) -> Self {
        Self { url, max_connections }
    }
}

pub async fn create_pool(config: DatabaseConfig) -> sqlx::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&config.url)
        .await
}

#[derive(Clone)]
pub struct DatabaseManager {
    pub pool: PgPool,
}
impl DatabaseManager {
    pub async fn new(pool: PgPool) -> Result<Self, Error> {
        Self::initialize_schema(&pool).await?;
        
        Ok(Self { pool })
    }

    async fn initialize_schema(pool: &PgPool) -> Result<(), Error> {
        let schema = r#"
            -- Properties Table            
            CREATE TABLE IF NOT EXISTS properties (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                address TEXT NOT NULL
            );
            
            -- Units Table
            CREATE TABLE IF NOT EXISTS units (
                id SERIAL PRIMARY KEY,
                property_id INTEGER,
                unit_number TEXT NOT NULL,
                is_occupied BOOLEAN DEFAULT FALSE,
                FOREIGN KEY (property_id) REFERENCES properties(id)
            );
            
            -- Tenants Table
            CREATE TABLE IF NOT EXISTS tenants (
                id SERIAL PRIMARY KEY,
                unit_id INTEGER,
                name TEXT NOT NULL,
                email TEXT,
                phone TEXT
            );
            
            -- Expenses Table
            CREATE TABLE IF NOT EXISTS expenses (
                id SERIAL PRIMARY KEY,
                amount REAL NOT NULL,
                description TEXT,
                date TEXT
            );
        "#;

        sqlx::query(schema)
            .execute(pool)
            .await?;

        Ok(())
    }

    // ====================================
    // ========= Insert ===================
    // ====================================

    pub async fn insert_property(&self, property: Property) -> Result<u64, Error> {
        let res = sqlx::query(
            "INSERT INTO properties (name, address) VALUES ($1, $2)"
        )
        .bind(property.name)
        .bind(property.address)
        .execute(&self.pool)
        .await?;
        
        Ok(res.rows_affected())
    }

    pub async fn insert_tenant(&self, tenant: Tenant) -> Result<u64, Error> {
        let res = sqlx::query(
            "INSERT INTO tenants (name, email, phone) VALUES (?, ?, ?)"
        )
        .bind(tenant.name)
        .bind(tenant.email)
        .bind(tenant.phone)
        .execute(&self.pool)
        .await?;

        Ok(res.rows_affected())
    }

    pub async fn assign_tenant_to_unit(&self, tenant_id: i64, unit_id: i64) -> Result<(), Error> {
        sqlx::query(
            "INSERT OR REPLACE INTO tenants (unit_id) VALUES (?) WHERE id = ?"
        )
        .bind(unit_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;
        
        sqlx::query(
            "UPDATE units SET is_occupied = TRUE WHERE id = ?"
        )
        .bind(unit_id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}

pub enum DatabaseOperation {
    Create,
    Query,
    Update,
    Delete,
    Close
}

pub struct DatabaseWorker {    
    pub channel: UnboundedSender<DatabaseOperation>,
    pub worker_thread: std::thread::JoinHandle<()>,
}

impl DatabaseWorker {
    pub fn new(pool: DatabaseManager) -> Self {
        //println!("Create new Expense Worker");
        let (sender, r) = tokio::sync::mpsc::unbounded_channel();
        let worker_thread = std::thread::spawn({
            let new_pool = pool;
            move || {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(database_worker_loop(new_pool, r))
            }
        });
        Self {
            channel: sender,
            worker_thread,
        }
    }
    pub fn join(self) -> std::thread::Result<()> {
        let _ = self.channel.send(DatabaseOperation::Close);
        self.worker_thread.join()
    }
}

async fn database_worker_loop(
    conn: DatabaseManager,
    mut r: UnboundedReceiver<DatabaseOperation>,
) {

}