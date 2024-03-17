use std::result::Result;
use chrono::{DateTime, Local, NaiveDate, Utc};
use database::add_tenant;
use sqlx::{sqlite::{SqliteQueryResult, SqliteConnectOptions}, Connection, Sqlite, Executor, SqlitePool, migrate::MigrateDatabase};
use tenant::Lease;

use crate::{
    database::{add_expense, add_property, initialize_database}, 
    properties::Property,
    tenant::Tenant
};


mod database;
mod properties;
mod tenant;

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
    test_database(&instances).await;
    instances.close().await;
}

async fn test_database(instances: &sqlx::Pool<Sqlite>) {
    let property = Property::new(0, "name".to_string(), "address".to_string(), "city".to_string(), "state".to_string(), "zip_code".to_string(), 10);
    match add_property(&instances, &property).await {
        Ok(_) => println!("Successfully added PROPERTY"),
        Err(e) => println!("Error when adding PROPERTY: {}", e),
    }

    let lease = Lease::new(NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(), NaiveDate::from_ymd_opt(2025, 2, 28).unwrap(), 1700.0, 1, "Check".to_string());
    let tenant = Tenant::new(0,1,"John".to_string(), "Smith".to_string(), "JohnSmith@gmail.com".to_string(), "2064445555".to_string(), NaiveDate::from_ymd_opt(2024, 3, 1).unwrap());
    match add_tenant(&instances, &tenant, &lease, 1).await {
        Ok(_) => println!("Successfully added TENANT"),
        Err(e) => println!("Error when adding TENANT: {}", e),
    }

    let dt = Utc::now();
    let expense = Expense::new(1, ExpenseType::Maintenance(MaintenanceType::Landscaping), 100.0, dt, "Normal Maintenance".to_string());
    match add_expense(&instances, &expense).await {
        Ok(_) => println!("Successfully added EXPENSE"),
        Err(e) => println!("Error when adding EXPENSE: {}", e),
    }
}
