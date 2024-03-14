use std::result::Result;
use sqlx::{sqlite::{SqliteQueryResult, SqliteConnectOptions}, Connection, Sqlite, Executor, SqlitePool, migrate::MigrateDatabase};

use crate::{Expense, ExpenseType, MaintenanceType, UtilitiesType};

pub async fn initialize_database() -> sqlx::Pool<Sqlite> {
    let db_url = String::from("sqlite://sqlite.db");
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        Sqlite::create_database(&db_url).await.unwrap();
        match create_schema(&db_url).await{
            Ok(_) => println!("Database created successfully"),
            Err(e) => panic!("{}", e)
        }
    }

    SqlitePool::connect(&db_url).await.unwrap()
}

pub async fn create_schema(db_url:&str) -> Result<SqliteQueryResult, sqlx::Error> {
    let pool = SqlitePool::connect(&db_url).await?;
    let qry = "
    PRAGMA foreign_keys = ON;
    CREATE TABLE IF NOT EXISTS leases (
        lease_id            integer PRIMARY KEY AUTOINCREMENT,
        start_date          text,
        end_date            text,
        monthly_rent        real,
        payment_due_date    text,
        payment_method      text
    );
    
    CREATE TABLE IF NOT EXISTS properties (
        property_id         integer PRIMARY KEY AUTOINCREMENT,
        property_name       text,
        address             text,
        city                text,
        state               text,
        zip_code            text,
        num_units           integer
    );
    
    CREATE TABLE IF NOT EXISTS maintenance_requests (
        request_id          integer PRIMARY KEY AUTOINCREMENT,
        tenant_id           integer,
        request_date        text,
        description         text,
        status              text,
        completion_date     text null
    );
    
    CREATE TABLE IF NOT EXISTS tenants (
        tenant_id           integer PRIMARY KEY AUTOINCREMENT,
        first_name          text,
        last_name           text,
        email               text,
        phone_number        text,
        move_in_date        text
    );
    
    CREATE TABLE IF NOT EXISTS expenses (
        expense_id          integer PRIMARY KEY AUTOINCREMENT,
        expense_type        text,
        amount              real,
        date_incurred       text,
        description         text,
        receipt_url         text null
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