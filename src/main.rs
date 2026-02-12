//#![windows_subsystem = "windows"]

mod generated_code {
    slint::include_modules!();
}

use chrono::NaiveDate;
use expenses::calculate_expense_totals;
pub use generated_code::*;
use slint::{ComponentHandle, Model, ModelRc, Weak, WindowPosition};
use sqlx::Sqlite;
use tokio::sync::{Mutex, mpsc::UnboundedSender};
use std::sync::Arc;
use crate::{database_worker::{DatabaseManager, DatabaseOperation, DatabaseTable}, leaseholders::Leaseholder};

mod app_settings;
mod database_worker;
mod database;
mod expenses;
mod lease;
mod leaseholders;
mod pdf_formatting;
mod properties;
mod slint_data_initialization;
mod statements;
mod testing;
mod time;

#[tokio::main]
async fn main() {
    //println!("{:?}", std::env::current_exe());
    app_settings::initialize_data_paths().await;

    let app_state = Arc::new(Mutex::new(AppState::new(database::initialize_database().await).await));
    let (init_data_result, _instances) = app_state.lock().await.load_initial_data().await;

    match init_data_result {
        Ok(_) => (), //println!("Data Initialized into memory"),
        Err(e) => println!("Error initializing data into memory: {e}"),
    }

    let mut valid_ids = get_ids(&app_state.lock().await.db_manager.db_pool).await;

    testing::activate_test_mode(true, &app_state.lock().await.db_manager.db_pool, &mut valid_ids).await;
    let app = App::new().unwrap();
    app.window().set_position(slint::WindowPosition::Logical(slint::LogicalPosition::new(0.,0.)));
    let weak_app = app.as_weak();

    let app_state = initialize_slint_properties(&weak_app, app_state, &valid_ids).await;

    let db_worker = database_worker::DatabaseWorker::new(app_state.lock().await.db_manager.clone());

    let app_state = intialize_slint_callbacks(
        &app,
        app_state,
        valid_ids.clone(),
        db_worker.channel.clone(),
    );

    app.run().unwrap();

    app_state.lock().await.db_manager.db_pool.close().await;
    let _db_result = db_worker.join();
}

struct AppState {
    db_manager: DatabaseManager,
    props_dirty: bool,
    properties: Vec<properties::Property>,
    tenants_dirty: bool,
    tenants: Vec<leaseholders::Leaseholder>,
    expenses_dirty: bool,
    expenses: Vec<expenses::Expense>,
    selected_building: Option<i32>,
    selected_unit: Option<i32>,
    selected_tenant: Option<i32>,
    selected_expense: Option<i32>,
}

impl AppState {
    async fn new(pool: sqlx::Pool<Sqlite>) -> Self {
        Self {
            db_manager: DatabaseManager::new(pool).await,
            props_dirty: false,
            properties: Vec::new(),
            tenants_dirty: false,
            tenants: Vec::new(),
            expenses_dirty: false,
            expenses: Vec::new(),
            selected_building: None,
            selected_unit: None,
            selected_tenant: None,
            selected_expense: None,
        }
    }
    async fn load_initial_data(&mut self) -> (Result<(), sqlx::Error>, &sqlx::Pool<Sqlite>) {
        let mut pool = &self.db_manager.db_pool;
        (self.properties, pool) = crate::database::get_properties(&pool).await;

        (self.tenants, pool) = crate::database::get_leaseholders(&pool).await;

        (self.expenses, pool) = crate::database::get_all_expenses(&pool).await;

        (Ok(()), pool)
    }
}

#[derive(Debug, Clone)]
struct ValidIds {
    expense_id: u32,
    property_id: u32,
    leaseholder_id: u32,
    lease_id: u32,
    statement_id: u32,
}

impl ValidIds {
    pub fn get_id(&mut self, id_type: IdType) -> u32 {
        let id: u32;
        match id_type {
            IdType::Expense => {
                id = self.expense_id;
                self.expense_id += 1;
            }
            IdType::Property => {
                id = self.property_id;
                self.property_id += 1;
            }
            IdType::Leaseholder => {
                id = self.leaseholder_id;
                self.leaseholder_id += 1;
            }
            IdType::Lease => {
                id = self.lease_id;
                self.lease_id += 1;
            }
            IdType::Statement => {
                id = self.statement_id;
                self.statement_id += 1;
            }
        };
        id
    }
}

async fn get_ids(pool: &sqlx::Pool<Sqlite>) -> ValidIds {    
    let db_url = String::from("sqlite://sqlite.db");
    if !<Sqlite as sqlx::migrate::MigrateDatabase>::database_exists(&db_url).await.unwrap_or(false) {
        let ids = ValidIds {
            expense_id: 0,
            property_id: 0,
            leaseholder_id: 0,
            lease_id: 0,
            statement_id: 0,
        };
        ids
    }
    else {
        let ids = ValidIds {
            expense_id: database::get_max_expense_id(pool).await,
            property_id: database::get_max_property_id(pool).await,
            leaseholder_id: database::get_max_leaseholder_id(pool).await,
            lease_id: 0,
            statement_id: database::get_max_statement_id(pool).await,
        };
        //println!("Created ID Struct: {:#?}", ids);
        ids
    }
}

async fn initialize_slint_properties(
    weak_app: &Weak<App>,
    app_state: Arc<Mutex<AppState>>,
    valid_ids: &ValidIds,
) -> Arc<Mutex<AppState>> {
    slint_data_initialization::initialize_slint_properties(
        &weak_app.upgrade().unwrap(),
        &app_state.lock().await.db_manager.db_pool,
        valid_ids,
    )
    .await;
    slint_data_initialization::initialize_slint_expenses(
        &weak_app.upgrade().unwrap(), 
        &app_state.lock().await.db_manager.db_pool, 
        valid_ids
    )
    .await;
    slint_data_initialization::initialize_slint_leaseholders(
        &weak_app.upgrade().unwrap(),
        &app_state.lock().await.db_manager.db_pool,
        valid_ids,
    )
    .await;

    app_state
}

fn intialize_slint_callbacks(
    app: &App,
    app_state: Arc<Mutex<AppState>>,
    valid_ids: ValidIds,
    tx: UnboundedSender<DatabaseOperation>,
) -> Arc<Mutex<AppState>> {
    let weak_app = app.as_weak();
    app.global::<Validation>().on_get_valid_id({
        let mut id_clone = valid_ids;
        move |input| {
            id_clone.get_id(input) as i32
        }}
    );

    app.global::<LesseeData>().on_new_lessee({
        let local_channel = tx.clone();
        move |message_type, input| {
            let message_res = match message_type {
                MessageType::Create => local_channel.send(DatabaseOperation::Create(
                    DatabaseTable::Leaseholder(Leaseholder::convert_from_slint(input)),
                )),
                MessageType::Update => local_channel.send(DatabaseOperation::Update(
                    DatabaseTable::Leaseholder(Leaseholder::convert_from_slint(input)),
                )),
                MessageType::Delete => local_channel.send(DatabaseOperation::Delete(
                    DatabaseTable::Leaseholder(Leaseholder::convert_from_slint(input)),
                )),
            };
            match message_res {
                Ok(_) => (),
                Err(e) => println!("Failed to send tenant message: {e}"),
            }
        }
    });
    /*
    app_ref.global::<RegisterData>().on_new_transaction({
        let local_channel = tx.clone();
        move |message_type, input| {
            let message_res = match message_type {
                MessageType::Create => local_channel.send(DatabaseOperation::Create(
                    DatabaseTable::Transaction(Transaction::from_slint(input)),
                )),
                MessageType::Update => local_channel.send(DatabaseOperation::Update(
                    DatabaseTable::Transaction(Transaction::from_slint(input)),
                )),
                MessageType::Delete => local_channel.send(DatabaseOperation::Delete(
                    DatabaseTable::Transaction(Transaction::from_slint(input)),
                )),
            };
            match message_res {
                Ok(_) => (),
                Err(e) => println!("Failed to send transaction message: {e}"),
            }
        }
    });
    */
    app.global::<WindowCallbacks>().on_open_window({
        let local_app = weak_app.clone().upgrade().unwrap();
        move |input| {
            match input {
                WindowType::Lessee => { 
                    println!("Opening Lessee Edit Window");
                    let menu = AddLeaseholderMenu::new().unwrap();
                    menu.show().unwrap();
                    menu.invoke_open_lessee(local_app.global::<LesseeData>().get_selected_lessee());
                }
                WindowType::Property => (),
                WindowType::Settings => ()
            };
        }
    });

    app.global::<ExpenseData>().on_new_expense({
        let expense_channel = tx.clone();
        move |message_type, input| {
            let message_res = match message_type { 
                MessageType::Create => expense_channel.send(DatabaseOperation::Create(
                    DatabaseTable::Expense(expenses::Expense::convert_from_slint(input.clone())),
                )),
                MessageType::Update => expense_channel.send(DatabaseOperation::Update(
                    DatabaseTable::Expense(expenses::Expense::convert_from_slint(input.clone())),
                )),
                MessageType::Delete => expense_channel.send(DatabaseOperation::Delete(
                    DatabaseTable::Expense(expenses::Expense::convert_from_slint(input.clone())),
                )),
            };
            match message_res {
                Ok(_) => (),
                Err(e) => println!("Failed to send expense message: {e}"),
            }
        }
    });

    app.global::<PropertyData>().on_new_property({
        let property_channel = tx.clone();
        move |message_type, input| {
            let message_res = match message_type {
                MessageType::Create => property_channel.send(DatabaseOperation::Create(
                    DatabaseTable::Property(properties::Property::convert_from_slint(input.clone())),
                )),
                MessageType::Update => property_channel.send(DatabaseOperation::Update(
                    DatabaseTable::Property(properties::Property::convert_from_slint(input.clone())),
                )),
                MessageType::Delete => property_channel.send(DatabaseOperation::Delete(
                    DatabaseTable::Property(properties::Property::convert_from_slint(input.clone())),
                )),
            };
            match message_res {
                Ok(_) => (),
                Err(e) => println!("Failed to send property message: {e}"),
            }
        }
    });

    app.global::<StatementData>().on_new_statement({
        let statement_channel = tx.clone();
        move |message_type, input| {
            let message_res = match message_type {
                MessageType::Create => statement_channel.send(DatabaseOperation::Create(
                    DatabaseTable::Statement(statements::Statement::convert_from_slint(input.clone())),
                )),
                MessageType::Update => statement_channel.send(DatabaseOperation::Update(
                    DatabaseTable::Statement(statements::Statement::convert_from_slint(input.clone())),
                )),
                MessageType::Delete => statement_channel.send(DatabaseOperation::Delete(
                    DatabaseTable::Statement(statements::Statement::convert_from_slint(input.clone())),
                )),
            };
            match message_res {
                Ok(_) => (),
                Err(e) => println!("Failed to send statement message: {e}"),
            }
        }
    });
    
    app_state
}
