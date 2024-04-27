//#![windows_subsystem = "windows"]

mod generated_code {
    slint::include_modules!();
}

pub use generated_code::*;
use slint::Model;

use crate::{
    app_settings::initialize_data_paths, expenses::*, slint_conversion::initialize_slint_expenses,
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

    let expense_instances = instances.clone();
    let expense_worker = ExpenseWorker::new(&app, &expense_instances);

    app.on_new_expense({
        let expense_channel = expense_worker.channel.clone();
        move |input| {
            let input_clone = input.clone();
            let res = expense_channel.send(ExpenseMessage::ExpenseCreated(input));
            match res {
                Ok(_) => println!("expense successfully sent"),
                Err(_e) => println!("expense send failed"),
            }
            let res = weak_app.upgrade_in_event_loop(move |handle| {
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

    app.run().unwrap();

    let _expense_result = expense_worker.join();
    instances.close().await;
}
