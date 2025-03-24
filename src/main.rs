use std::{sync::Arc, vec};
use rusqlite::Error;
use async_std::sync::Mutex;
use models::{Property, Tenant};
use database::DatabaseManager;

mod app_settings;
mod database;
mod models;
mod conversion;
mod validation;

slint::include_modules!();

#[async_std::main]
async fn main() {    
    app_settings::initialize_data_paths().await;
    let app_state = Arc::new(Mutex::new(AppState::default()));
    let _ = app_state.lock().await.load_initial_data().await;
    let app = App::new().unwrap();
    app.window().set_position(slint::WindowPosition::Logical(slint::LogicalPosition::new(0.,0.)));
    let weak_app = app.as_weak();

    let app = initialize_slint_properties(&app, app_state.clone()).await;
    setup_event_handlers(&app, app_state.clone());
    app.run().unwrap();
}

struct AppState {
    db_manager: DatabaseManager,
    properties: Vec<Property>,
    tenants: Vec<Tenant>,
    selected_building: Option<i32>,
    selected_unit: Option<i32>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            db_manager: DatabaseManager::new(app_settings::TESTING_DATABASE_PATH).unwrap(),
            properties: Vec::new(),
            tenants: Vec::new(),
            selected_building: None,
            selected_unit: None,
        }
    }
}

impl AppState {
    async fn load_initial_data(&mut self) -> Result<(), Error> {
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
        
        Ok(())
    }
}

async fn initialize_slint_properties(app_ref: &App, app_state: Arc<Mutex<AppState>>) -> &App {
    //app_ref.global::<TenantData>().set_tenants(app_state.lock().await.tenants); // need to convert
    app_ref
}

fn setup_event_handlers(app_ref: &App, app_state: Arc<Mutex<AppState>>) { 
    

}