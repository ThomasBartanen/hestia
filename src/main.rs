//#![windows_subsystem = "windows"]

mod generated_code {
    slint::include_modules!();
}

pub use generated_code::*;
use slint::Model;
use sqlx::Sqlite;

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
    app_settings::initialize_data_paths().await;
    let instances = database::initialize_database().await;

    testing::activate_test_mode(true, &instances).await;
    let app = App::new().unwrap();
    let weak_app = app.as_weak();

    let valid_ids = get_ids(&instances).await;

    slint_conversion::initialize_slint_properties(
        &weak_app.upgrade().unwrap(),
        &instances,
        &valid_ids,
    )
    .await;
    slint_conversion::initialize_slint_expenses(
        &weak_app.upgrade().unwrap(),
        &instances,
        &valid_ids,
        1,
    )
    .await;
    slint_conversion::initialize_slint_leaseholders(
        &weak_app.upgrade().unwrap(),
        &instances,
        &valid_ids,
    )
    .await;

    let worker_instances = instances.clone();
    let expense_worker = expenses::ExpenseWorker::new(&worker_instances);
    let property_worker = properties::PropertyWorker::new(&worker_instances);
    let lessee_worker = leaseholders::LeaseholderWorker::new(&worker_instances);

    intialize_slint_callbacks(&app, &expense_worker, &property_worker);

    app.run().unwrap();

    instances.close().await;
    let _expense_result = expense_worker.join();
    let _property_result = property_worker.join();
    let _lessee_result = lessee_worker.join();
}

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

fn intialize_slint_callbacks(
    app: &App,
    expense_worker: &expenses::ExpenseWorker,
    property_worker: &properties::PropertyWorker,
) {
    let weak_app = app.as_weak();

    //app.global::<Validation>().on_get_valid_id(move |input| {});
    app.on_new_expense({
        let expense_channel = expense_worker.channel.clone();
        let local_app = weak_app.clone();
        move |input| {
            let input_clone = input.clone();
            let message = match input_clone.message {
                crate::MessageType::Create => expenses::ExpenseMessage::ExpenseCreated(input),
                crate::MessageType::Update => expenses::ExpenseMessage::ExpenseUpdate(input),
                crate::MessageType::Delete => expenses::ExpenseMessage::ExpenseDelete(input),
            };
            let res = expense_channel.send(message);
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
                match input_clone.message {
                    MessageType::Create => new_expenses.push(input_clone),
                    MessageType::Update => {
                        let index = new_expenses
                            .iter()
                            .position(|r| {
                                println!("r.id: {}. input_clone.id: {}", r.id, input_clone.id);
                                r.id == input_clone.id
                            })
                            .unwrap();
                        new_expenses.remove(index);
                        new_expenses.insert(index, input_clone);
                    }
                    MessageType::Delete => {
                        let index = new_expenses
                            .iter()
                            .position(|r| {
                                println!("r.id: {}. input_clone.id: {}", r.id, input_clone.id);
                                r.id == input_clone.id
                            })
                            .unwrap();
                        new_expenses.remove(index);
                    }
                }
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
                crate::MessageType::Delete => properties::PropertyMessage::PropertyRemove(input),
            };
            println!("New Property message sent: {}", message);
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
}
