use std::result::Result;
use sqlx::{sqlite::{SqliteQueryResult, SqliteConnectOptions}, Connection, Sqlite, Executor, SqlitePool, migrate::MigrateDatabase};

use crate::{
    properties::Property, 
    tenant::{Lease, Tenant}, 
    expenses::*
};

pub async fn initialize_database() -> sqlx::Pool<Sqlite> {
    let db_url = String::from("sqlite://sqlite.db");
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        Sqlite::create_database(&db_url).await.unwrap();
        match create_schema(&db_url).await{
            Ok(_) => println!("Database created successfully"),
            Err(e) => panic!("{}", e)
        }
    } else {
        println!("Database already exists");
    }

    SqlitePool::connect(&db_url).await.unwrap()
}

pub async fn create_schema(db_url:&str) -> Result<SqliteQueryResult, sqlx::Error> {
    let pool = SqlitePool::connect(&db_url).await?;
    let qry = "
    PRAGMA foreign_keys = ON;
    CREATE TABLE IF NOT EXISTS leases (
        lease_id            INTEGER PRIMARY KEY AUTOINCREMENT,
        start_date          TEXT,
        end_date            TEXT,
        monthly_rent        REAL,
        payment_due_date    TEXT,
        payment_method      TEXT
    );  
    CREATE TABLE IF NOT EXISTS properties (
        property_id         INTEGER PRIMARY KEY AUTOINCREMENT,
        property_name       TEXT,
        address             TEXT,
        city                TEXT,
        state               TEXT,
        zip_code            TEXT,
        num_units           INTEGER
    );
    CREATE TABLE IF NOT EXISTS maintenance_requests (
        request_id          INTEGER PRIMARY KEY AUTOINCREMENT,
        tenant_id           INTEGER,
        request_date        TEXT,
        description         TEXT,
        status              TEXT,
        completion_date     TEXT null,        
        FOREIGN KEY (tenant_id) REFERENCES tenants(tenant_id)
    );
    CREATE TABLE IF NOT EXISTS tenants (
        tenant_id           INTEGER PRIMARY KEY AUTOINCREMENT,
        lease_id            INTEGER,
        property_id         INTEGER,
        first_name          TEXT,
        last_name           TEXT,
        email               TEXT,
        phone_number        TEXT,
        move_in_date        TEXT,
        FOREIGN KEY (lease_id) REFERENCES leases(lease_id)
        FOREIGN KEY (property_id) REFERENCES properties(property_id)
    );    
    CREATE TABLE IF NOT EXISTS expenses (
        expense_id          INTEGER PRIMARY KEY AUTOINCREMENT,
        property_id         INTEGER,
        expense_type        TEXT,
        amount              REAL,
        date_incurred       TEXT,
        description         TEXT,
        receipt_url         TEXT null,
        FOREIGN KEY (property_id) REFERENCES properties(property_id)
    );";
    //maintenance_id      integer FOREIGN KEY REFERENCES maintenance_requests(request_id) null,
    let result = sqlx::query(&qry).execute(&pool).await;
    pool.close().await;
    return result;
}

pub async fn add_expense(pool: &sqlx::Pool<Sqlite>, expense: &Expense) -> Result<(), sqlx::Error> {
    let expense_type_str = match &expense.expense_type {
        ExpenseType::Maintenance(maintenance_type) => {
            match maintenance_type {
                MaintenanceType::Repairs => String::from("Maintenance: Repairs"),
                MaintenanceType::Cleaning => String::from("Maintenance: Cleaning"),
                MaintenanceType::Landscaping => String::from("Maintenance: Landscaping"),
                MaintenanceType::Other => String::from("Maintenance: Other"),
            }
        }
        ExpenseType::Utilities(utilities_type) => {
            match utilities_type {
                UtilitiesType::Water => String::from("Utilities: Water"),
                UtilitiesType::Electricity => String::from("Utilities: Electricity"),
                UtilitiesType::Gas => String::from("Utilities: Gas"),
                UtilitiesType::Other => String::from("Utilities: Other"),
            }
        }
        ExpenseType::Other => String::from("Other"),
    };

    sqlx::query(
        "INSERT INTO expenses (property_id, expense_type, amount, date_incurred, description) VALUES (?, ?, ?, ?, ?)")
        .bind(&expense.property_id)
        .bind(&expense_type_str)
        .bind(expense.amount)
        .bind(expense.date.to_string())
        .bind(&expense.description)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_property(pool: &sqlx::Pool<Sqlite>, property: &Property) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO properties (property_name, address, city, state, zip_code, num_units) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&property.address)
        .bind(&property.city)
        .bind(&property.state)
        .bind(&property.zip_code)
        .bind(property.num_units)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_tenant(pool: &sqlx::Pool<Sqlite>, tenant: &Tenant, lease: &Lease, property_id: u16) -> Result<(), sqlx::Error> {
    let x = sqlx::query(
        "INSERT INTO leases (start_date, end_date, monthly_rent, payment_due_date) VALUES (?, ?, ?, ?)")
        .bind(lease.start_date.to_string())
        .bind(lease.end_date.to_string())
        .bind(lease.monthly_rent)
        .bind(lease.payment_due_date)
        .execute(pool)
        .await?
        .last_insert_rowid();

    sqlx::query(
        "INSERT INTO tenants (lease_id, property_id, first_name, last_name, email, phone_number, move_in_date) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(x)
        .bind(property_id)
        .bind(&tenant.first_name)
        .bind(&tenant.last_name)
        .bind(&tenant.email)
        .bind(&tenant.phone_number)
        .bind(&tenant.move_in_date.to_string())
        .execute(pool)
        .await?;
    Ok(())
}