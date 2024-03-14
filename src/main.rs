use std::result::Result;
use chrono::{DateTime, Utc, Local};
use sqlx::{sqlite::{SqliteQueryResult, SqliteConnectOptions}, Connection, Sqlite, Executor, SqlitePool, migrate::MigrateDatabase};

use crate::database::{add_expense, initialize_database};


mod database;

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
    let dt = Utc::now();
    let expense = Expense::new(0, ExpenseType::Maintenance(MaintenanceType::Landscaping), 100.0, dt, "Normal Maintenance".to_string());
    let _result = add_expense(&instances, &expense).await;

    instances.close().await;
}
