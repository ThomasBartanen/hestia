use chrono::NaiveDate;
use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, FromRow, Sqlite, SqlitePool};
use std::result::Result;

use crate::{
    expenses::*, lease::Lease, leaseholders::Leaseholder, properties::Property,
    statements::Statement,
};

pub async fn initialize_database() -> sqlx::Pool<Sqlite> {
    let db_url = String::from("sqlite://sqlite.db");
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        Sqlite::create_database(&db_url).await.unwrap();
        match create_schema(&db_url).await {
            Ok(_) => println!("Database created successfully"),
            Err(e) => panic!("{}", e),
        }
    } else {
        println!("Database already exists");
    }

    SqlitePool::connect(&db_url).await.unwrap()
}

pub async fn create_schema(db_url: &str) -> Result<SqliteQueryResult, sqlx::Error> {
    let pool = SqlitePool::connect(db_url).await?;
    let qry = "
    PRAGMA foreign_keys = ON;
    CREATE TABLE IF NOT EXISTS leases (
        lease_id            INTEGER PRIMARY KEY AUTOINCREMENT,
        start_date          TEXT,
        end_date            TEXT,
        fee_structure       TEXT,
        payment_method      TEXT
    );  
    CREATE TABLE IF NOT EXISTS properties (
        property_id         INTEGER PRIMARY KEY AUTOINCREMENT,
        property_name       TEXT,
        property_tax        TEXT,
        business_insurance  TEXT,
        address             TEXT,
        city                TEXT,
        state               TEXT,
        zip_code            TEXT,
        num_units           INTEGER
    );
    CREATE TABLE IF NOT EXISTS maintenance_requests (
        request_id          INTEGER PRIMARY KEY AUTOINCREMENT,
        leaseholder_id      INTEGER,
        request_date        TEXT,
        maintenance_type    TEXT,
        description         TEXT,
        status              TEXT,
        completion_date     TEXT null,
        FOREIGN KEY (leaseholder_id) REFERENCES leaseholders(leaseholder_id) ON DELETE SET NULL
    );
    CREATE TABLE IF NOT EXISTS leaseholders (
        leaseholder_id      INTEGER PRIMARY KEY AUTOINCREMENT,
        lease_id            INTEGER,
        property_id         INTEGER,
        name                TEXT,
        address             TEXT,
        city                TEXT,
        state               TEXT,
        zip_code            TEXT,
        email               TEXT,
        phone_number        TEXT,
        move_in_date        TEXT,
        FOREIGN KEY (lease_id) REFERENCES leases(lease_id) ON DELETE SET NULL
        FOREIGN KEY (property_id) REFERENCES properties(property_id) ON DELETE CASCADE
    );    
    CREATE TABLE IF NOT EXISTS expenses (
        expense_id          INTEGER PRIMARY KEY AUTOINCREMENT,
        property_id         INTEGER,
        expense_type        TEXT,
        amount              REAL,
        date_incurred       TEXT,
        description         TEXT,
        receipt_url         TEXT null,
        FOREIGN KEY (property_id) REFERENCES properties(property_id) ON DELETE SET NULL
    );
    CREATE TABLE IF NOT EXISTS statements (
        statement_id        INTEGER PRIMARY KEY AUTOINCREMENT,
        leaseholder_id      INTEGER,
        amount_due          INTEGER,
        amount_paid         INTEGER,
        statement_path      TEXT,
        FOREIGN KEY (leaseholder_id) REFERENCES leaseholders(leaseholder_id) ON DELETE CASCADE
    )";
    let result = sqlx::query(qry).execute(&pool).await;
    pool.close().await;
    result
}

// -------------------------------------- ADD ---------------------------------------------

pub async fn add_maint_request(
    pool: &sqlx::Pool<Sqlite>,
    request: &MaintenanceRequest,
) -> Result<(), sqlx::Error> {
    println!("Adding Maintenence Request");
    let maint_type_str = match request.request_type {
        MaintenanceType::Repairs => String::from("Maintenance: Repairs"),
        MaintenanceType::Cleaning => String::from("Maintenance: Cleaning"),
        MaintenanceType::Landscaping => String::from("Maintenance: Landscaping"),
        MaintenanceType::Other => String::from("Maintenance: Other"),
    };
    sqlx::query("INSERT INTO maintenance_requests (leaseholder_id, request_date, maintenance_type, description, status, completion_date) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(request.leaseholder_id)
        .bind(request.request_date.to_string())
        .bind(maint_type_str)
        .bind(&request.description)
        .bind(RequestStatus::Received.to_string())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_expense(pool: &sqlx::Pool<Sqlite>, expense: &Expense) -> Result<(), sqlx::Error> {
    println!("Adding Expense");
    let expense_type_str = &expense.expense_type.to_string();
    sqlx::query(
        "INSERT INTO expenses (property_id, expense_type, amount, date_incurred, description) VALUES (?, ?, ?, ?, ?)")
        .bind(expense.property_id)
        .bind(expense_type_str)
        .bind(expense.amount)
        .bind(expense.date.to_string())
        .bind(&expense.description)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_property(
    pool: &sqlx::Pool<Sqlite>,
    property: &Property,
) -> Result<SqliteQueryResult, sqlx::Error> {
    println!("Adding Property");
    let x = sqlx::query(
        "INSERT INTO properties (property_name, property_tax, business_insurance, address, city, state, zip_code, num_units) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&property.name)
        .bind(property.property_tax)
        .bind(property.business_insurance)
        .bind(&property.address.street_address)
        .bind(&property.address.city)
        .bind(&property.address.state)
        .bind(&property.address.zip_code)
        .bind(property.num_units)
        .execute(pool)
        .await?;
    Ok(x)
}

pub async fn add_leaseholders(
    pool: &sqlx::Pool<Sqlite>,
    leaseholder: &Leaseholder,
    property_id: u32,
) -> Result<SqliteQueryResult, sqlx::Error> {
    println!("Adding Leaseholder");
    let lease = &leaseholder.lease;

    let lease_id =
        sqlx::query("INSERT INTO leases (start_date, end_date, fee_structure) VALUES (?, ?, ?)")
            .bind(lease.start_date.to_string())
            .bind(lease.end_date.to_string())
            .bind(leaseholder.lease.fee_structure.encode_to_database_string())
            .execute(pool)
            .await?
            .last_insert_rowid();

    let leaseholder_result = sqlx::query(
        "INSERT INTO leaseholders (lease_id, property_id, name, address, city, state, zip_code, email, phone_number, move_in_date) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(lease_id)
        .bind(property_id)
        .bind(&leaseholder.contact_info.name)
        .bind(&leaseholder.contact_info.remittence_address.street_address)
        .bind(&leaseholder.contact_info.remittence_address.city)
        .bind(&leaseholder.contact_info.remittence_address.state)
        .bind(&leaseholder.contact_info.remittence_address.zip_code)
        .bind(&leaseholder.contact_info.email)
        .bind(&leaseholder.contact_info.phone_number)
        .bind(&leaseholder.move_in_date.to_string())
        .execute(pool)
        .await?;
    Ok(leaseholder_result)
}

pub async fn add_statement(
    pool: &sqlx::Pool<Sqlite>,
    statement: &Statement,
) -> Result<SqliteQueryResult, sqlx::Error> {
    println!("Adding Statement");
    let x = sqlx::query(
        "INSERT INTO statements (leaseholder_id, amount_due, amount_paid, statement_path) VALUES (?, ?, ?, ?)")
        .bind(statement.leaseholder.id)
        .bind(statement.total)
        .bind(0)
        .bind("test_statement")
        .execute(pool)
        .await?;

    Ok(x)
}

// -------------------------------------- GET ---------------------------------------------
pub async fn get_properties(pool: &sqlx::Pool<Sqlite>) -> Vec<Property> {
    let mut properties: Vec<Property> = vec![];

    let property_rows = sqlx::query("SELECT * FROM properties")
        .fetch_all(pool)
        .await;
    for row in property_rows.unwrap() {
        let property = Property::from_row(&row);
        properties.push(property.unwrap());
    }
    properties
}

pub async fn get_leaseholders(pool: &sqlx::Pool<Sqlite>) -> Vec<Leaseholder> {
    let mut leaseholders: Vec<Leaseholder> = vec![];

    let leaseholder_rows = sqlx::query("SELECT * FROM leaseholders")
        .fetch_all(pool)
        .await;

    for row in leaseholder_rows.unwrap() {
        let leaseholder = Leaseholder::from_row(&row);
        leaseholders.push(leaseholder.unwrap());
    }
    leaseholders
}

pub async fn get_all_expenses(pool: &sqlx::Pool<Sqlite>) -> Vec<Expense> {
    let mut expenses: Vec<Expense> = vec![];

    let expense_rows = sqlx::query("SELECT * FROM expenses").fetch_all(pool).await;
    for row in expense_rows.unwrap() {
        let expense = Expense::from_row(&row);
        expenses.push(expense.unwrap());
    }
    expenses
}

pub async fn get_property_expenses(pool: &sqlx::Pool<Sqlite>, property_id: u32) -> Vec<Expense> {
    let mut expenses: Vec<Expense> = vec![];

    let expense_rows = sqlx::query("SELECT * FROM expenses WHERE property_id = ?")
        .bind(property_id)
        .fetch_all(pool)
        .await;
    for row in expense_rows.unwrap() {
        let expense = Expense::from_row(&row);
        expenses.push(expense.unwrap());
    }
    expenses
}

pub async fn get_current_property_expenses(
    pool: &sqlx::Pool<Sqlite>,
    property_id: u32,
    cutoff_date: NaiveDate,
) -> Vec<Expense> {
    let mut expenses: Vec<Expense> = vec![];

    let expense_rows =
        sqlx::query("SELECT * FROM expenses WHERE property_id = ? AND date_incurred > ?")
            .bind(property_id)
            .bind(cutoff_date.to_string())
            .fetch_all(pool)
            .await;
    for row in expense_rows.unwrap() {
        let expense = Expense::from_row(&row);
        expenses.push(expense.unwrap());
    }
    expenses
}

// -------------------------------------- UPDATE ---------------------------------------------

pub async fn update_property(
    pool: &sqlx::Pool<Sqlite>,
    property: &Property,
) -> Result<SqliteQueryResult, sqlx::Error> {
    let x = sqlx::query("UPDATE properties SET (property_name, property_tax, business_insurance, address, city, state, zip_code, num_units) = (?, ?, ?, ?, ?, ?, ?, ?) WHERE property_id == ?")
        .bind(&property.name)
        .bind(property.property_tax)
        .bind(property.business_insurance)
        .bind(&property.address.street_address)
        .bind(&property.address.city)
        .bind(&property.address.state)
        .bind(&property.address.zip_code)
        .bind(property.num_units)
        .bind(property.id)
        .execute(pool)
        .await?;
    Ok(x)
}

pub async fn update_expense(
    pool: &sqlx::Pool<Sqlite>,
    expense: &Expense,
) -> Result<SqliteQueryResult, sqlx::Error> {
    let expense_type_str = &expense.expense_type.to_string();

    let x = sqlx::query(
        "UPDATE expenses SET (property_id, expense_type, amount, date_incurred, description) = (?, ?, ?, ?, ?) WHERE expense_id == ?")
        .bind(expense.property_id)
        .bind(expense_type_str)
        .bind(expense.amount)
        .bind(expense.date.to_string())
        .bind(&expense.description)
        .bind(expense.id)
        .execute(pool)
        .await?;
    Ok(x)
}

pub async fn update_leaseholder(
    pool: &sqlx::Pool<Sqlite>,
    leaseholder: &Leaseholder,
) -> Result<SqliteQueryResult, sqlx::Error> {
    let x = sqlx::query(
        "UPDATE leaseholders SET (lease_id, property_id, name, remittence_address, email, phone_number, move_in_date) = (?, ?, ?, ?, ?, ?, ?) WHERE leaseholder_id == ?"
    )
        .bind(leaseholder.lease.id)
        .bind(leaseholder.property_id)
        .bind(&leaseholder.contact_info.name)
        .bind(&leaseholder.contact_info.remittence_address.street_address)
        .bind(&leaseholder.contact_info.remittence_address.city)
        .bind(&leaseholder.contact_info.remittence_address.state)
        .bind(&leaseholder.contact_info.remittence_address.zip_code)
        .bind(&leaseholder.contact_info.email)
        .bind(&leaseholder.contact_info.phone_number)
        .bind(&leaseholder.move_in_date.to_string())
        .bind(leaseholder.id)
        .execute(pool)
        .await?;
    Ok(x)
}

pub async fn update_lease(
    pool: &sqlx::Pool<Sqlite>,
    new_lease: &Lease,
) -> Result<SqliteQueryResult, sqlx::Error> {
    let x = sqlx::query(
        "UPDATE leases SET (start_date, end_date, fee_structure) = (?, ?, ?) WHERE lease_id == ?",
    )
    .bind(new_lease.start_date.to_string())
    .bind(new_lease.end_date.to_string())
    .bind(new_lease.fee_structure.encode_to_database_string())
    .execute(pool)
    .await?;
    Ok(x)
}

// -------------------------------------- REMOVE ---------------------------------------------
pub async fn remove_expense(
    pool: &sqlx::Pool<Sqlite>,
    expense: &Expense,
) -> Result<SqliteQueryResult, sqlx::Error> {
    println!("Removing Expense with id: {}", expense.id);
    let x = sqlx::query("DELETE FROM expenses WHERE expense_id == ?")
        .bind(expense.id)
        .execute(pool)
        .await?;
    Ok(x)
}
pub async fn remove_property(
    pool: &sqlx::Pool<Sqlite>,
    property: &Property,
) -> Result<SqliteQueryResult, sqlx::Error> {
    println!("Removing Property with id: {}", property.id);
    let x = sqlx::query("DELETE FROM properties WHERE property_id == ?")
        .bind(property.id)
        .execute(pool)
        .await?;
    Ok(x)
}
pub async fn remove_leaseholder(
    pool: &sqlx::Pool<Sqlite>,
    lessee: &Leaseholder,
) -> Result<SqliteQueryResult, sqlx::Error> {
    println!("Removing Leaseholder with id: {}", lessee.id);
    let x = sqlx::query("DELETE FROM leaseholders WHERE leaseholder_id == ?")
        .bind(lessee.id)
        .execute(pool)
        .await?;
    Ok(x)
}

// -------------------------------------- Get Max ID ---------------------------------------------
pub async fn get_max_expense_id(pool: &sqlx::Pool<Sqlite>) -> u32 {
    let res = sqlx::query("SELECT * FROM expenses ORDER BY expense_id DESC LIMIT 1;")
        .fetch_one(pool)
        .await;
    match res {
        Ok(r) => match Expense::from_row(&r) {
            Ok(o) => o.id + 1,
            Err(e) => {
                println!("Error parsing expense record for expense id: {}", e);
                0
            }
        },
        Err(e) => {
            println!("Error getting max expense id: {}", e);
            0
        }
    }
}
pub async fn get_max_property_id(pool: &sqlx::Pool<Sqlite>) -> u32 {
    let res = sqlx::query("SELECT * FROM properties ORDER BY property_id DESC LIMIT 1;")
        .fetch_one(pool)
        .await;
    match res {
        Ok(r) => match Property::from_row(&r) {
            Ok(o) => o.id + 1,
            Err(e) => {
                println!("Error parsing property record for property id: {}", e);
                0
            }
        },
        Err(e) => {
            println!("Error getting max property id: {}", e);
            0
        }
    }
}

pub async fn get_max_leaseholder_id(pool: &sqlx::Pool<Sqlite>) -> u32 {
    let res = sqlx::query("SELECT * FROM leaseholders ORDER BY leaseholder_id DESC LIMIT 1;")
        .fetch_one(pool)
        .await;
    match res {
        Ok(r) => match Leaseholder::from_row(&r) {
            Ok(o) => o.id + 1,
            Err(e) => {
                println!("Error parsing leaseholder record for leaseholder id: {}", e);
                0
            }
        },
        Err(e) => {
            println!("Error getting max leaseholder id: {}", e);
            0
        }
    }
}
