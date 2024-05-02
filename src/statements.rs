use crate::{
    app_settings::PathSettings,
    database::add_statement,
    expenses::*,
    lease::FeeStructure,
    leaseholders::{Company, Leaseholder},
    pdf_formatting::write_with_printpdf,
    properties::Property,
};
use chrono::NaiveDate;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Debug, Clone)]
pub struct Statement {
    pub date: NaiveDate,
    pub leaseholder: Leaseholder,
    pub rates: FeeStructure,
    pub fees: Vec<Expense>,
    pub total: f32,
}

impl Statement {
    pub fn new(date: NaiveDate, tenant: Leaseholder, fees: Vec<Expense>) -> Statement {
        let tenant_clone = tenant.clone();
        Statement {
            date,
            leaseholder: tenant,
            rates: tenant_clone.clone().lease.fee_structure,
            fees,
            total: calculate_total(tenant_clone.lease.fee_structure, 1000.0),
        }
    }
}

pub fn calculate_total(fee_structure: FeeStructure, building_fees: f32) -> f32 {
    match fee_structure {
        FeeStructure::Gross(rent) => rent.base_rent,
        FeeStructure::SingleNet(rent, tax) => {
            rent.base_rent + calculate_share(tax.property_tax, building_fees)
        }
        FeeStructure::DoubleNet(rent, tax, insurance) => {
            rent.base_rent
                + calculate_share(tax.property_tax, building_fees)
                + calculate_share(insurance.building_insurance, building_fees)
        }
        FeeStructure::TripleNet(rent, tax, insurance, cam) => {
            rent.base_rent
                + calculate_share(tax.property_tax, building_fees)
                + calculate_share(insurance.building_insurance, building_fees)
                + calculate_share(cam.amenities, building_fees)
                + calculate_share(cam.electicity, building_fees)
                + calculate_share(cam.garbage, building_fees)
                + calculate_share(cam.landscaping, building_fees)
                + calculate_share(cam.misc, building_fees)
                + calculate_share(cam.recycling, building_fees)
                + calculate_share(cam.water, building_fees)
        }
    }
}

pub fn calculate_share(rate: f32, total: f32) -> f32 {
    total * rate
}

pub fn create_statement(
    statement: Statement,
    property: Property,
    company: Company,
    settings: PathSettings,
) {
    write_with_printpdf(statement, property, company, settings);
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
        println!("Create new Statement Worker");
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
                StatementMessage::StatementCreated(create) => {}
                StatementMessage::StatementUpdate(update) => {}
                StatementMessage::StatementDelete(remove) => {}
                StatementMessage::Quit => {
                    println!("Quitting");
                    continue;
                }
            },
            None => continue,
        };
    }
}
