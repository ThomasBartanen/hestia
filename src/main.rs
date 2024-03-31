use chrono::NaiveDate;
use database::add_tenant;
use sqlx::Sqlite;
use statements::Statement;
use tenant::{CAMRates, InsuranceRate, Lease, PropertyTaxRate, Rent};

use crate::{
    database::{add_expense, add_property, get_current_expenses, initialize_database},
    expenses::*,
    properties::{Address, Property},
    statements::create_statement,
    tenant::{ContactInformation, Tenant},
};

mod database;
mod expenses;
mod pdf_formatting;
mod properties;
mod statements;
mod tenant;

#[async_std::main]
async fn main() {
    let instances = initialize_database().await;
    test_database(&instances).await;
    instances.close().await;
}

async fn test_database(instances: &sqlx::Pool<Sqlite>) {
    let mut property = Property::new(
        0,
        "name".to_string(),
        Address::new(
            "address".to_string(),
            "city".to_string(),
            "state".to_string(),
            "zip_code".to_string(),
        ),
        1000.0,
        950.0,
        10,
    );
    match add_property(instances, &property).await {
        Ok(r) => {
            //converting i64 to u16. This may cause issues. Keep an eye on this
            property.id = r.last_insert_rowid() as u16;
            println!("Successfully added PROPERTY");
        }
        Err(e) => println!("Error when adding PROPERTY: {}", e),
    };

    let contact = ContactInformation::new(
        "John".to_string(),
        "Smith".to_string(),
        "JohnSmith@gmail.com".to_string(),
        "2064445555".to_string(),
    );

    let lease = Lease::new(
        NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 2, 28).unwrap(),
        tenant::FeeStructure::TripleNet(
            Rent { base_rent: 1700.0 },
            PropertyTaxRate { property_tax: 0.2 },
            InsuranceRate {
                building_insurance: 0.15,
            },
            CAMRates {
                electicity: 0.3,
                recycling: 0.3,
                garbage: 0.3,
                water: 0.3,
                landscaping: 0.3,
                amenities: 0.3,
                misc: 0.1,
            },
        ),
        "Check".to_string(),
    );
    let mut tenant = Tenant::new(
        0,
        lease.clone(),
        property.id,
        contact,
        NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
    );
    match add_tenant(instances, &tenant, 1).await {
        Ok(t) => {
            tenant.id = t.last_insert_rowid() as u16;
            println!("Successfully added TENANT")
        }
        Err(e) => println!("Error when adding TENANT: {}", e),
    }

    let dt = NaiveDate::from_ymd_opt(2024, 3, 10);
    let expense = Expense::new(
        property.id,
        ExpenseType::Maintenance(MaintenanceType::Landscaping),
        100.0,
        dt.unwrap(),
        "Normal Maintenance".to_string(),
    );
    match add_expense(instances, &expense).await {
        Ok(_) => println!("Successfully added EXPENSE"),
        Err(e) => println!("Error when adding EXPENSE: {}", e),
    }

    let expense = Expense::new(
        property.id,
        ExpenseType::Utilities(UtilitiesType::Electricity),
        1920.0,
        dt.unwrap(),
        "Electricity Bill".to_string(),
    );
    match add_expense(instances, &expense).await {
        Ok(_) => println!("Successfully added EXPENSE"),
        Err(e) => println!("Error when adding EXPENSE: {}", e),
    }

    let expense = Expense::new(
        property.id,
        ExpenseType::Utilities(UtilitiesType::Water),
        450.0,
        dt.unwrap(),
        "Water Bill".to_string(),
    );
    match add_expense(instances, &expense).await {
        Ok(_) => println!("Successfully added EXPENSE"),
        Err(e) => println!("Error when adding EXPENSE: {}", e),
    }

    let expense = Expense::new(
        property.id,
        ExpenseType::Other,
        100.0,
        dt.unwrap(),
        "Rat Abatement".to_string(),
    );
    match add_expense(instances, &expense).await {
        Ok(_) => println!("Successfully added EXPENSE"),
        Err(e) => println!("Error when adding EXPENSE: {}", e),
    }

    println!(
        "Property ID: {}, Tenant's Property ID: {}, Lease ID: {}",
        property.id, tenant.property_id, tenant.lease.id
    );

    let statement = Statement::new(
        NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
        tenant,
        get_current_expenses(
            instances,
            property.id,
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
        )
        .await,
    );
    //println!("New Statement: {:#?}", statement);

    create_statement(statement, property);
}
