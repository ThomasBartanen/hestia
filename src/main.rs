use std::result::Result;
use chrono::{DateTime, Local, NaiveDate, Utc};
use database::add_tenant;
use sqlx::{sqlite::{SqliteQueryResult, SqliteConnectOptions}, Connection, Sqlite, Executor, SqlitePool, migrate::MigrateDatabase};
use statements::Statement;
use tenant::{CAMRates, InsuranceRate, Lease, PropertyTaxRate, Rent};

use crate::{
    database::{add_expense, add_property, get_current_expenses, initialize_database}, expenses::*, properties::Property, statements::create_statement, tenant::Tenant
};

mod database;
mod properties;
mod tenant;
mod expenses;
mod statements;
mod pdf_formatting;

#[async_std::main]
async fn main() {
    let instances = initialize_database().await;
    test_database(&instances).await;
    instances.close().await;
}

async fn test_database(instances: &sqlx::Pool<Sqlite>) {
    let property = Property::new(
        0, 
        "name".to_string(), 
        "address".to_string(), 
        "city".to_string(), 
        "state".to_string(), 
        "zip_code".to_string(), 
        10);
    match add_property(&instances, &property).await {
        Ok(_) => println!("Successfully added PROPERTY"),
        Err(e) => println!("Error when adding PROPERTY: {}", e),
    }

    let lease = Lease::new(
        NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 2, 28).unwrap(),
        tenant::FeeStructure::TripleNet(
            Rent{ base_rent: 1700.0 },
            PropertyTaxRate{ property_tax: 0.2},
            InsuranceRate{ building_insurance: 0.15},
            CAMRates{
                electicity: 0.3,
                recycling: 0.3,
                garbage: 0.3,
                water: 0.3,
                landscaping: 0.3,
                amenities: 0.3,
                misc: 0.1,
            }),
        "Check".to_string());
    let tenant = Tenant::new(lease.clone(), property.id,"John".to_string(), "Smith".to_string(), "JohnSmith@gmail.com".to_string(), "2064445555".to_string(), NaiveDate::from_ymd_opt(2024, 3, 1).unwrap());
    match add_tenant(&instances, &tenant, 1).await {
        Ok(_) => println!("Successfully added TENANT"),
        Err(e) => println!("Error when adding TENANT: {}", e),
    }

    let dt = NaiveDate::from_ymd_opt(2024, 3, 10);
    let expense = Expense::new(1, ExpenseType::Maintenance(MaintenanceType::Landscaping), 100.0, dt.unwrap(), "Normal Maintenance".to_string());
    match add_expense(&instances, &expense).await {
        Ok(_) => println!("Successfully added EXPENSE"),
        Err(e) => println!("Error when adding EXPENSE: {}", e),
    }


    let statement = Statement::new(
        NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(), 
        tenant,
        get_current_expenses(&instances, property.id, NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()).await
    );
    println!("New Statement: {:#?}", statement);

    create_statement(statement);
}
