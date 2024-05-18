use std::str::FromStr;

use crate::{
    app_settings::PathSettings,
    database,
    expenses::*,
    lease::FeeStructure,
    leaseholders::{Company, Leaseholder},
    pdf_formatting::write_with_printpdf,
    properties::Property,
};
use chrono::NaiveDate;
use serde::Serialize;
use sqlx::{sqlite::SqliteRow, Row, FromRow, Sqlite};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Debug, Clone)]
pub struct Statement {
    pub id: u32,
    pub date: NaiveDate,
    pub leaseholder_id: u32,
    pub statement_name: String,
    pub rates: FeeStructure,
    pub fees: Vec<Expense>,
    pub total: f32,
    pub amount_paid: f32
}

impl Statement {
    pub async fn new(pool: &sqlx::Pool<Sqlite>, date: NaiveDate, tenant: Leaseholder) -> Statement {
        let tenant_clone = tenant.clone();
        let property = database::get_property(pool, tenant.property_id).await;
        let fees = database::get_current_property_expenses(
            &pool,
            property.id,
            date,
        ).await;
        Statement {
            id: 0,
            date,
            statement_name: tenant_clone.contact_info.name.to_owned(),
            leaseholder_id: tenant.id,
            rates: tenant_clone.clone().lease.fee_structure,
            fees: fees.clone(),
            total: calculate_total(tenant_clone, property, fees),
            amount_paid: 0.0,
        }
    }
}

impl<'r> FromRow<'r, SqliteRow> for Statement {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let id: u32 = row.try_get("statement_id")?;
        let leaseholder_id: u32 = row.try_get("leaseholder_id")?;
        let date_string: String = row.try_get("statement_id")?;
        let total: f32 = row.try_get("amount_due")?;
        let amount_paid: f32 = row.try_get("amount_paid")?;
        let statement_name: String = row.try_get("filename")?;
        let rates_string: String = row.try_get("fee_structure")?;
        let fee_list: String = row.try_get("expense_list")?;

        let date = NaiveDate::from_str(&date_string).unwrap();

        Ok(Statement {
            id,
            date,
            statement_name,
            leaseholder_id,
            rates: serde_json::from_str(&rates_string).unwrap(),
            fees: serde_json::from_str(&fee_list).unwrap(),
            total,
            amount_paid,
        })
    }
}

pub fn calculate_total(tenant: Leaseholder, property: Property, expenses: Vec<Expense>) -> f32 {
    let fee_structure = tenant.lease.fee_structure;
    let mut elect_total: f32 = 0.0;
    let mut garb_recycl_total: f32 = 0.0;
    let mut water_total: f32 = 0.0;
    let mut gas_total: f32 = 0.0;
    let mut landscaping_total: f32 = 0.0;
    let mut repairs_total: f32 = 0.0;
    let mut misc_total: f32 = 0.0;

    expenses.iter().for_each(|x|
        match &x.expense_type {
            ExpenseType::Maintenance(m) => match m {
                MaintenanceType::Repairs => repairs_total += x.amount,
                MaintenanceType::Cleaning => misc_total += x.amount,
                MaintenanceType::Landscaping => landscaping_total += x.amount,
                MaintenanceType::Other => misc_total += x.amount,
            },
            ExpenseType::Utilities(u) => match u {
                UtilitiesType::Water => water_total += x.amount,
                UtilitiesType::Electricity => elect_total += x.amount,
                UtilitiesType::Garbage => garb_recycl_total += x.amount,
                UtilitiesType::Gas => gas_total += x.amount,
                UtilitiesType::Other => misc_total += x.amount,
            },
            ExpenseType::Other => misc_total += x.amount,
        }
    );

    match fee_structure {
        FeeStructure::Gross(rent) =>  rent.base_rent,
        FeeStructure::SingleNet(rent, tax) => {
            rent.base_rent + calculate_share(tax.property_tax, property.property_tax)
        }
        FeeStructure::DoubleNet(rent, tax, insurance) => {
            rent.base_rent
                + calculate_share(tax.property_tax, property.property_tax)
                + calculate_share(insurance.building_insurance, property.business_insurance)
        }
        FeeStructure::TripleNet(rent, tax, insurance, cam) => {
            rent.base_rent
                + calculate_share(tax.property_tax, property.property_tax)
                + calculate_share(insurance.building_insurance, property.business_insurance)
                + calculate_share(cam.electicity, elect_total)
                + calculate_share(cam.garbage, garb_recycl_total)
                + calculate_share(cam.landscaping, landscaping_total)
                + calculate_share(cam.repairs, repairs_total)
                + calculate_share(cam.misc, misc_total)
                + calculate_share(cam.water, water_total)
        }
    }
}

pub fn calculate_share(rate: f32, total: f32) -> f32 {
    total * rate
}

pub async fn create_statement(
    pool: &sqlx::Pool<Sqlite>,
    statement: Statement,
    property: Property,
    company: Company,
    settings: PathSettings,
) {
    let tenant = database::get_leaseholder(&pool, statement.leaseholder_id).await;
    write_with_printpdf(statement, tenant, property, company, settings);
}

pub enum StatementMessage {
    StatementCreated(Statement),
    StatementUpdate(Statement),
    StatementDelete(Statement),
    Quit,
}

pub struct StatementWorker {
    pub channel: UnboundedSender<StatementMessage>,
    pub worker_thread: std::thread::JoinHandle<()>,
}

impl StatementWorker {
    pub fn new(pool: &sqlx::Pool<sqlx::Sqlite>) -> Self {
        //println!("Create new Statement Worker");
        let (sender, r) = tokio::sync::mpsc::unbounded_channel();
        let worker_thread = std::thread::spawn({
            let new_pool = pool.clone();
            move || {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(statement_worker_loop(new_pool, r))
            }
        });
        Self {
            channel: sender,
            worker_thread,
        }
    }
    pub fn join(self) -> std::thread::Result<()> {
        let _ = self.channel.send(StatementMessage::Quit);
        self.worker_thread.join()
    }
}

async fn statement_worker_loop(
    pool: sqlx::Pool<sqlx::Sqlite>,
    mut r: UnboundedReceiver<StatementMessage>,
) {
    loop {
        let m = r.recv().await;

        match m {
            Some(s) => match s {
                StatementMessage::StatementCreated(create) => {
                    match database::add_statement(&pool, &create).await {
                        Ok(_) => (),
                        Err(e) => println!("Failed to add statement: {}", e),
                    }
                }
                StatementMessage::StatementUpdate(update) => {
                    match database::update_statement(&pool, &update).await {
                        Ok(_) => (),
                        Err(e) => println!("Failed to update statement: {}", e),
                    }
                }
                StatementMessage::StatementDelete(remove) => {
                    match database::remove_statement(&pool, &remove).await {
                        Ok(_) => (),
                        Err(e) => println!("Failed to remove statement: {}", e),
                    }
                }
                StatementMessage::Quit => {
                    println!("Quitting");
                    continue;
                }
            },
            None => continue,
        };
    }
}
