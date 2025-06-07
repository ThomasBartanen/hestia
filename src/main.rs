use database::{
    create_pool, DatabaseConfig, DatabaseManager, DatabaseOperation, DatabaseTable, DatabaseWorker,
};
use models::{Expense, Property, Tenant};
use slint::{Model, ModelRc, VecModel};
use sqlx::{postgres::PgRow, Executor, PgPool, Row};
use std::{fmt::Error, sync::Arc, vec};
use tokio::sync::{mpsc::UnboundedSender, Mutex};

mod app_settings;
mod conversion;
mod database;
mod models;
mod validation;

slint::include_modules!();

#[tokio::main]
async fn main() {
    app_settings::initialize_data_paths().await;

    let config = DatabaseConfig::new(
        format!("postgres://postgres:RRC1@localhost/postgres"),
        30,
        30,
    );

    let pool = match create_pool(config).await {
        Ok(p) => p,
        Err(e) => panic!("Failed to create pool due to: {e}"),
    };

    let app_state = Arc::new(Mutex::new(AppState::new(pool).await));
    let _ = app_state.lock().await.load_initial_data().await;

    let app = App::new().unwrap();
    app.window()
        .set_position(slint::WindowPosition::Logical(slint::LogicalPosition::new(
            0., 0.,
        )));
    let weak_app = app.as_weak();

    let (app, app_state) = initialize_slint_properties(&app, app_state.clone()).await;
    let db_worker = DatabaseWorker::new(app_state.lock().await.db_manager.clone());
    setup_event_handlers(&app, db_worker.channel.clone()).await;
    app.run().unwrap();
    let _ = db_worker.join();
}

struct AppState {
    db_manager: DatabaseManager,
    properties: Vec<Property>,
    tenants: Vec<Tenant>,
    expenses: Vec<Expense>,
    selected_building: Option<i32>,
    selected_unit: Option<i32>,
    selected_tenant: Option<i32>,
    selected_expense: Option<i32>,
}

impl AppState {
    async fn new(pool: PgPool) -> Self {
        Self {
            db_manager: DatabaseManager::new(pool).await.unwrap(),
            properties: Vec::new(),
            tenants: Vec::new(),
            expenses: Vec::new(),
            selected_building: None,
            selected_unit: None,
            selected_tenant: None,
            selected_expense: None,
        }
    }
    async fn load_initial_data(&mut self) -> Result<(), Error> {
        self.properties = self.db_manager.select_all_properties().await;

        self.tenants = self.db_manager.select_all_tenants().await;

        /*
        for element in self.properties.iter_mut() {
            //probably a very inefficient way of doing it
            let mut unit_stmt = self.db_manager.pool.fetch_all(
                "SELECT id FROM units WHERE property_id = ?1",
            ).await;
            let _ = unit_stmt.unwrap().iter().map(|row| Ok(element.units.push(PgRow::get(row, 0))));
        }
        */

        self.expenses = self.db_manager.select_all_expenses().await;

        Ok(())
    }
}

async fn initialize_slint_properties(
    app_ref: &App,
    app_state: Arc<Mutex<AppState>>,
) -> (&App, Arc<Mutex<AppState>>) {
    let tenants = app_state.lock().await.tenants.clone();
    let converted_tenants: Vec<_> = tenants.into_iter().map(|t| Tenant::to_slint(&t)).collect();
    app_ref
        .global::<TenantData>()
        .set_tenants(ModelRc::new(VecModel::from(converted_tenants)));

    let properties = app_state.lock().await.properties.clone();
    let converted_properties: Vec<_> = properties
        .into_iter()
        .map(|p| Property::to_slint(&p))
        .collect();
    app_ref
        .global::<PropertyData>()
        .set_props(ModelRc::new(VecModel::from(converted_properties)));

    (app_ref, app_state)
}

async fn setup_event_handlers(app_ref: &App, tx: UnboundedSender<DatabaseOperation>) {
    app_ref.global::<TenantData>().on_new_tenant({
        let local_channel = tx.clone();
        move |message_type, input| {
            let message_res = match message_type {
                MessageType::Create => local_channel.send(DatabaseOperation::Create(
                    DatabaseTable::Tenant(Tenant::from_slint(input)),
                )),
                MessageType::Update => local_channel.send(DatabaseOperation::Update(
                    DatabaseTable::Tenant(Tenant::from_slint(input)),
                )),
                MessageType::Delete => local_channel.send(DatabaseOperation::Delete(
                    DatabaseTable::Tenant(Tenant::from_slint(input)),
                )),
            };
            match message_res {
                Ok(_) => (),
                Err(e) => println!("Failed to send tenant message: {e}"),
            }
        }
    });
    app_ref.global::<ExpenseData>().on_new_expense({
        let local_channel = tx.clone();
        move |message_type, input| {
            let message_res = match message_type {
                MessageType::Create => local_channel.send(DatabaseOperation::Create(
                    DatabaseTable::Expense(Expense::from_slint(input)),
                )),
                MessageType::Update => local_channel.send(DatabaseOperation::Update(
                    DatabaseTable::Expense(Expense::from_slint(input)),
                )),
                MessageType::Delete => local_channel.send(DatabaseOperation::Delete(
                    DatabaseTable::Expense(Expense::from_slint(input)),
                )),
            };
            match message_res {
                Ok(_) => (),
                Err(e) => println!("Failed to send expense message: {e}"),
            }
        }
    });
}

