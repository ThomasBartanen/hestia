//#![windows_subsystem = "windows"]

mod generated_code {
    slint::include_modules!();
}

pub use generated_code::*;
use properties::PropertyWorker;
use slint::Model;

use crate::{
    app_settings::initialize_data_paths,
    expenses::*,
    slint_conversion::{initialize_slint_expenses, initialize_slint_properties},
};

mod app_settings;
mod database;
mod expenses;
mod lease;
mod leaseholders;
mod pdf_formatting;
mod properties;
mod slint_conversion;
mod statements;
mod testing;

#[async_std::main]
async fn main() {
    //println!("{:?}", std::env::current_exe());
    let instances = database::initialize_database().await;
    initialize_data_paths().await;

    testing::activate_test_mode(false, &instances).await;
    let app = App::new().unwrap();
    let weak_app = app.as_weak();

    initialize_slint_expenses(&weak_app.upgrade().unwrap(), &instances, 1).await;
    initialize_slint_properties(&weak_app.upgrade().unwrap(), &instances).await;
    let valid_ids = get_ids(&instances).await;

    let worker_instances = instances.clone();
    let expense_worker = ExpenseWorker::new(&worker_instances);
    let property_worker = PropertyWorker::new(&worker_instances);
#[derive(Debug)]
struct ValidIds {
    expense_id: u32,
    property_id: u32,
    leaseholder_id: u32,
}

async fn get_ids(pool: &sqlx::Pool<Sqlite>) -> ValidIds {
    let ids = ValidIds {
        expense_id: database::get_max_expense_id(pool).await,
        property_id: database::get_max_property_id(pool).await,
        leaseholder_id: database::get_max_leaseholder_id(pool).await,
    };
    println!("Created ID Struct: {:#?}", ids);
    ids
}

    app.on_new_expense({
        let expense_channel = expense_worker.channel.clone();
        let local_app = weak_app.clone();
        move |input| {
            let input_clone = input.clone();
            let res = expense_channel.send(ExpenseMessage::ExpenseCreated(input));
            match res {
                Ok(_) => println!("expense successfully sent"),
                Err(_e) => println!("expense send failed"),
            }
            let res = local_app.upgrade_in_event_loop(move |handle| {
                let prev_expense = handle.get_expenses();
                let new_expenses = prev_expense
                    .as_any()
                    .downcast_ref::<slint::VecModel<ExpenseInput>>()
                    .expect("Expenses failed to downcast");
                new_expenses.push(input_clone);
            });
            match res {
                Ok(_) => (),
                Err(e) => println!("Failed to upgrade ui: {e}"),
            };
        }
    });

    app.on_new_property({
        let property_channel = property_worker.channel.clone();
        let local_app = weak_app.clone();
        move |input| {
            let input_clone = input.clone();
            let message = match input_clone.message {
                crate::MessageType::Create => properties::PropertyMessage::PropertyCreated(input),
                crate::MessageType::Update => properties::PropertyMessage::PropertyUpdate(input),
                crate::MessageType::Delete => todo!(),
            };
            let res = property_channel.send(message);
            match res {
                Ok(_) => println!("property successfully sent"),
                Err(_e) => println!("property send failed"),
            }
            let res = local_app.upgrade_in_event_loop(move |handle| {
                let prev_property = handle.get_properties();
                let new_properties = prev_property
                    .as_any()
                    .downcast_ref::<slint::VecModel<PropertyInput>>()
                    .expect("Properties failed to downcast");
                new_properties.push(input_clone);
            });
            match res {
                Ok(_) => (),
                Err(e) => println!("Failed to upgrade ui: {e}"),
            };
        }
    });

    app.run().unwrap();

    let _expense_result = expense_worker.join();
    let _property_result = property_worker.join();
    instances.close().await;
}
