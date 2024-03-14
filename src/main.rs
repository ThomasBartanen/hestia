use std::result::Result;
use chrono::{DateTime, Utc, Local};
use sqlx::{sqlite::{SqliteQueryResult, SqliteConnectOptions}, Connection, Sqlite, Executor, SqlitePool, migrate::MigrateDatabase};

use crate::{database::{add_expense, add_property, initialize_database}, properties::Property};


mod database;
mod properties;

#[derive(Debug)]
enum ExpenseType {
    Maintenance(MaintenanceType),
    Utilities(UtilitiesType),
    Other,
}

#[derive(Debug)]
enum MaintenanceType {
    Repairs,
    Cleaning,
    Landscaping,
    Other,
}

#[derive(Debug)]
enum UtilitiesType {
    Water,
    Electricity,
    Gas,
    Other,
}

struct Expense {
    property_id: u16,
    expense_type: ExpenseType,
    amount: f64,
    date: DateTime<Utc>,
    description: String
}

impl Expense {
    fn new(property_id: u16, expense_type: ExpenseType, amount: f64, date: DateTime<Utc>, description: String) -> Expense {
        Expense {
            property_id,
            expense_type,
            amount,
            date,
            description
        }
    }
}
#[async_std::main]
async fn main() {
    let instances = initialize_database().await;
    let property = Property::new(0, "name".to_string(), "address".to_string(), "city".to_string(), "state".to_string(), "zip_code".to_string(), 10);
    match add_property(&instances, &property).await {
        Ok(_) => println!("Successfully added PROPERTY"),
        Err(e) => println!("Error when adding PROPERTY: {}", e),
    }

    let dt = Utc::now();
    let expense = Expense::new(0, ExpenseType::Maintenance(MaintenanceType::Landscaping), 100.0, dt, "Normal Maintenance".to_string());
    match add_expense(&instances, &expense).await {
        Ok(_) => println!("Successfully added EXPENSE"),
        Err(e) => println!("Error when adding EXPENSE: {}", e),
    }

    instances.close().await;
}
