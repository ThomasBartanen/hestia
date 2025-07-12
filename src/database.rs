use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgRow},
    Connection, Error, Executor, Row,
};
use std::{path::Path, str::FromStr, sync::Arc};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};

use crate::{
    models::{Transaction, Property, Tenant, Unit},
    AppState,
};

pub const DATABASE_NAME: &str = "test_db.db3";

use chrono::NaiveDate;
use sqlx::{PgPool, Pool, Postgres};
use std::time::Duration;

#[derive(Debug)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub acquire_timeout: u32,
}

impl DatabaseConfig {
    pub fn new(url: String, max_connections: u32, acquire_timeout: u32) -> Self {
        Self {
            url,
            max_connections,
            acquire_timeout,
        }
    }
}

pub async fn create_pool(config: DatabaseConfig) -> sqlx::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(config.acquire_timeout.into()))
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
        let mut conn = pool.acquire().await?;
        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS properties (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                address TEXT NOT NULL
            );",
        )
        .execute(pool)
        .await?;
        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS units (
                id SERIAL PRIMARY KEY,
                property_id INTEGER,
                unit_number TEXT NOT NULL,
                tenant_id INTEGER,
                is_occupied BOOLEAN DEFAULT FALSE,
                FOREIGN KEY (property_id) REFERENCES properties(id)
            );",
        )
        .execute(pool)
        .await?;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS tenants (
                id SERIAL PRIMARY KEY,
                unit_id INTEGER,
                name TEXT NOT NULL,
                email TEXT,
                phone TEXT
            );",
        )
        .execute(pool)
        .await?;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS transactions (
                id SERIAL PRIMARY KEY,
                prop_id INTEGER,
                amount REAL NOT NULL,
                date TEXT,
                description TEXT
            );",
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    // ====================================
    // ========= Insert ===================
    // ====================================

    pub async fn insert_property(&self, property: Property) -> Result<u64, Error> {
        let res = sqlx::query("INSERT INTO properties (name, address) VALUES ($1, $2)")
            .bind(property.name)
            .bind(property.address)
            .execute(&self.pool)
            .await?;

        Ok(res.rows_affected())
    }

    pub async fn insert_transaction(&self, transaction: Transaction) -> Result<u64, Error> {
        let res = sqlx::query(
            "INSERT INTO transactions (prop_id, amount, date, description) VALUES ($1, $2, $3, $4)",
        )
        .bind(transaction.property_id)
        .bind(transaction.amount)
        .bind(transaction.date.to_string())
        .bind(transaction.description)
        .execute(&self.pool)
        .await?;

        Ok(res.rows_affected())
    }

    pub async fn insert_unit(&self, unit: Unit) -> Result<u64, Error> {
        let res = sqlx::query("INSERT INTO units (property_id, unit_number) VALUES (?, ?)")
            .bind(unit.building_id)
            .bind(unit.unit_number)
            .execute(&self.pool)
            .await?;

        Ok(res.rows_affected())
    }

    pub async fn insert_tenant(&self, tenant: Tenant) -> Result<u64, Error> {
        let res = sqlx::query("INSERT INTO tenants (name, email, phone) VALUES (?, ?, ?)")
            .bind(tenant.name)
            .bind(tenant.email)
            .bind(tenant.phone)
            .execute(&self.pool)
            .await?;

        Ok(res.rows_affected())
    }

    pub async fn assign_tenant_to_unit(&self, tenant_id: i64, unit_id: i64) -> Result<(), Error> {
        sqlx::query("INSERT OR REPLACE INTO tenants (unit_id) VALUES (?) WHERE id = ?")
            .bind(unit_id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;

        sqlx::query("UPDATE units SET is_occupied = TRUE, tenant_id = ? WHERE id = ?")
            .bind(tenant_id)
            .bind(unit_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ====================================
    // ========= SELECT INDIVIDUAL ========
    // ====================================
    /*
    pub async fn select_property(&self, id: i32) -> Property {
        let res =
            sqlx::query("SELECT * FROM properties WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await;

        match res {
            Ok(o) => {
                println!("Successfully retreived property. {o}");
                return Property::from(o)
            },
            Err(e) => todo!(),
        }
    }
    */

    // ====================================
    // ========= SELECT ALL ===============
    // ====================================
    pub async fn select_all_properties(&self) -> Vec<Property> {
        self.pool
            .fetch_all("SELECT * FROM properties ORDER BY name")
            .await
            .unwrap()
            .iter()
            .map(|row| Property {
                id: PgRow::get(row, 0),
                name: PgRow::get(row, 1),
                address: PgRow::get(row, 2),
                units: Vec::new(),
            })
            .collect::<Vec<Property>>()
    }

    pub async fn select_all_tenants(&self) -> Vec<Tenant> {
        self.pool
            .fetch_all("SELECT * FROM tenants")
            .await
            .unwrap()
            .iter()
            .map(|row| Tenant {
                id: PgRow::get(row, 0),
                name: PgRow::get(row, 2),
                email: PgRow::get(row, 3),
                phone: PgRow::get(row, 4),
            })
            .collect::<Vec<Tenant>>()
    }

    pub async fn select_all_transactions(&self) -> Vec<Transaction> {
        self.pool
            .fetch_all("SELECT * FROM transactions")
            .await
            .unwrap()
            .iter()
            .map(|row| Transaction {
                id: PgRow::get(row, 0),
                property_id: PgRow::get(row, 1),
                transaction_type: String::from(""),
                amount: PgRow::get(row, 2),
                date: NaiveDate::from_str(PgRow::get(row, 3)).unwrap(),
                description: PgRow::get(row, 4),
            })
            .collect::<Vec<Transaction>>()
    }
}

pub enum DatabaseOperation {
    Create(DatabaseTable),
    Query(DatabaseTable),
    Update(DatabaseTable),
    Delete(DatabaseTable),
    Close,
}

pub enum DatabaseTable {
    Property(Property),
    Unit(Unit),
    Tenant(Tenant),
    Transaction(Transaction),
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

async fn database_worker_loop(conn: DatabaseManager, mut r: UnboundedReceiver<DatabaseOperation>) {
    loop {
        match r.recv().await.unwrap() {
            DatabaseOperation::Create(t) => match t {
                DatabaseTable::Property(property) => match conn.insert_property(property).await {
                    Ok(o) => println!("Successfully inserted new property. Row: {o}"),
                    Err(e) => println!("Error inserting new property: {e}"),
                },
                DatabaseTable::Unit(unit) => match conn.insert_unit(unit).await{
                    Ok(o) => println!("Successfully inserted new unit. Row: {o}"),
                    Err(e) => println!("Error inserting new unit: {e}"),
                },
                DatabaseTable::Tenant(tenant) => match conn.insert_tenant(tenant).await {
                    Ok(o) => println!("Successfully inserted new tenant. Row: {o}"),
                    Err(e) => println!("Error inserting new tenant: {e}"),
                },
                DatabaseTable::Transaction(expense) => match conn.insert_transaction(expense).await {
                    Ok(o) => println!("Successfully inserted new expense. Row: {o}"),
                    Err(e) => println!("Error inserting new expense: {e}"),
                },
            },
            DatabaseOperation::Query(t) => match t {
                /*DatabaseTable::Property(property) => match conn.select_property(property.id) {
                    Ok(o) => println!("Successfully selected property."),
                    Err(e) => println!("Error selecting property: {e}"),
                }*/
                DatabaseTable::Property(property) => todo!(),
                DatabaseTable::Unit(unit) => todo!(),
                DatabaseTable::Tenant(tenant) => todo!(),
                DatabaseTable::Transaction(expense) => todo!(),
            },
            DatabaseOperation::Update(t) => match t {
                DatabaseTable::Property(property) => todo!(),
                DatabaseTable::Unit(unit) => todo!(),
                DatabaseTable::Tenant(tenant) => todo!(),
                DatabaseTable::Transaction(expense) => todo!(),
            },
            DatabaseOperation::Delete(t) => match t {
                DatabaseTable::Property(property) => todo!(),
                DatabaseTable::Unit(unit) => todo!(),
                DatabaseTable::Tenant(tenant) => todo!(),
                DatabaseTable::Transaction(expense) => todo!(),
            },
            DatabaseOperation::Close => println!("Closing app"),
        };
    }
}
