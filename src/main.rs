use std::{fmt::Error, sync::Arc, vec};
use models::{Property, Tenant};
use database::{create_pool, DatabaseConfig, DatabaseManager, DatabaseWorker};
use slint::{Model, ModelRc, VecModel};
use sqlx::PgPool;
use tokio::sync::Mutex;

mod app_settings;
mod database;
mod models;
mod conversion;
mod validation;

slint::include_modules!();

#[tokio::main]
async fn main() {
    app_settings::initialize_data_paths().await;
    
    let config = DatabaseConfig::new(
        "postgres://username:password@localhost/database".to_string(),
        5,
    );
    
    let pool = create_pool(config).await.unwrap();

    let app_state = Arc::new(Mutex::new(AppState::new(pool).await));
    let _ = app_state.lock().await.load_initial_data().await;

    let app = App::new().unwrap();
    app.window().set_position(slint::WindowPosition::Logical(slint::LogicalPosition::new(0.,0.)));
    let weak_app = app.as_weak();

    let (app, app_state) = initialize_slint_properties(&app, app_state.clone()).await;
    let db_worker = DatabaseWorker::new(app_state.lock().await.db_manager.clone());
    setup_event_handlers(&app, db_worker).await;
    app.run().unwrap();
}

struct AppState {
    db_manager: DatabaseManager,
    properties: Vec<Property>,
    tenants: Vec<Tenant>,
    selected_building: Option<i32>,
    selected_unit: Option<i32>,
}


impl AppState {
    async fn new(pool: PgPool) -> Self {
        Self {
            db_manager: DatabaseManager::new(pool).await.unwrap(),
            properties: Vec::new(),
            tenants: Vec::new(),
            selected_building: None,
            selected_unit: None,
        }
    }
    async fn load_initial_data(&mut self) -> Result<(), Error> {
        /*
        let mut prop_stmt = self.db_manager.conn.prepare(
            "SELECT * FROM properties ORDER BY name"
        )?;
        
        let rows = prop_stmt.query_map([], |row| {
            Ok(Property {
                id: row.get(0)?,
                name: row.get(1)?,
                address: row.get(2)?,
                units: Vec::new()
            })
        })?;
        
        self.properties = rows.collect::<Result<Vec<_>, _>>()?;

        for element in self.properties.iter_mut() {
            //probably a very inefficient way of doing it
            let mut unit_stmt = self.db_manager.conn.prepare(
                "SELECT id FROM units WHERE property_id = ?1",
            )?;
            let _ = unit_stmt.query_map([element.id], |row| Ok(element.units.push(row.get::<usize, i32>(0).unwrap())))?;
        }
        */
        Ok(())
    }
}

async fn initialize_slint_properties(app_ref: &App, app_state: Arc<Mutex<AppState>>) -> (&App, Arc<Mutex<AppState>>) {
    let tenants = app_state.lock().await.tenants.clone();
    let converted_tenants: Vec<_> = tenants.into_iter().map(|t| Tenant::to_slint(&t)).collect();
    app_ref.global::<TenantData>().set_tenants(ModelRc::new(VecModel::from(converted_tenants)));

    let properties = app_state.lock().await.properties.clone();
    let converted_properties: Vec<_> = properties.into_iter().map(|p| Property::to_slint(&p)).collect();
    app_ref.global::<PropertyData>().set_props(ModelRc::new(VecModel::from(converted_properties)));
    
    (app_ref, app_state)
}

async fn setup_event_handlers(app_ref: &App, worker: DatabaseWorker) { 
    app_ref.global::<TenantData>().on_new_tenant({
        move |message_type, input| {
            let message = match message_type {
                MessageType::Create => worker.channel.send(database::DatabaseOperation::Create),
                MessageType::Update => worker.channel.send(database::DatabaseOperation::Update),
                MessageType::Delete => worker.channel.send(database::DatabaseOperation::Delete),
            };
    }});
}