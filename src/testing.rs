use crate::{
    app_settings::PathSettings,
    database::{
        add_expense, add_leaseholders, add_property, add_statement, get_current_expenses,
        update_property,
    },
    lease::{self, *},
    leaseholders::*,
    properties::*,
    statements::{create_statement, Statement},
    Expense, ExpenseType, MaintenanceType, UtilitiesType,
};
use chrono::NaiveDate;
use sqlx::Sqlite;

pub async fn activate_test_mode(activate: bool, instances: &sqlx::Pool<Sqlite>) {
    if activate {
        let settings = test_settings().await;
        let (company, leaseholder, mut property) = test_database(instances).await;
        test_expenses(instances, &property).await;
        test_statements(instances, &mut property, leaseholder, company, settings).await;
    }
}

async fn test_settings() -> PathSettings {
    PathSettings::default()
}

async fn test_database(instances: &sqlx::Pool<Sqlite>) -> (Company, Leaseholder, Property) {
    println!("- - - Testing Database - - -");
    let company = Company::new("Company".to_owned(), 3241523);

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
            property.id = r.last_insert_rowid() as u32;
            println!("Successfully added PROPERTY");
        }
        Err(e) => println!("Error when adding PROPERTY: {}", e),
    };

    let contact = ContactInformation::new(
        Address::new(
            "3322 S 55th Street".to_string(),
            "Seattle".to_string(),
            "WA".to_string(),
            "97132".to_string(),
        ),
        "JohnSmith@gmail.com".to_string(),
        "2064445555".to_string(),
    );

    let lease = Lease::new(
        NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 2, 28).unwrap(),
        lease::FeeStructure::TripleNet(
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
    let mut leaseholder = Leaseholder::new(
        0,
        lease.clone(),
        property.id,
        LeaseholderType::IndividualLeaseholder(Individual {
            first_name: "John".to_owned(),
            last_name: "Example".to_owned(),
        }),
        contact,
        NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
    );
    match add_leaseholders(instances, &leaseholder, property.id).await {
        Ok(t) => {
            leaseholder.id = t.last_insert_rowid() as u32;
            println!("Successfully added LEASEHOLDER")
        }
        Err(e) => println!("Error when adding LEASEHOLDER: {}", e),
    };
    (company, leaseholder, property)
}

pub async fn test_expenses(instances: &sqlx::Pool<Sqlite>, property: &Property) {
    println!("- - - Testing Expenses - - -");
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
}

pub async fn test_statements(
    instances: &sqlx::Pool<Sqlite>,
    property: &mut Property,
    leaseholder: Leaseholder,
    company: Company,
    settings: PathSettings,
) {
    println!("- - - Testing Statements - - -");
    let statement = Statement::new(
        NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
        leaseholder,
        get_current_expenses(
            instances,
            property.id,
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
        )
        .await,
    );
    match add_statement(instances, &statement).await {
        Ok(_) => println!("Successfully added STATEMENT"),
        Err(e) => println!("Error when adding STATEMENT: {}", e),
    }
    //println!("New Statement: {:#?}", statement);

    create_statement(statement, property.clone(), company, settings);

    property.business_insurance += 100.0;
    match update_property(instances, property).await {
        Ok(_) => println!("Successfully updated PROPERTY. ID: {}", property.id),
        Err(e) => println!("Error when adding STATEMENT: {}", e),
    }
}
