//#![windows_subsystem = "windows"]

mod generated_code {
    slint::include_modules!();
}

use chrono::NaiveDate;
use expenses::calculate_expense_totals;
pub use generated_code::*;
use slint::{Model, ModelRc, Weak};
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
mod time;

#[async_std::main]
async fn main() {
    //println!("{:?}", std::env::current_exe());
    app_settings::initialize_data_paths().await;
    let instances = database::initialize_database().await;

    let mut valid_ids = get_ids(&instances).await;

    testing::activate_test_mode(true, &instances, &mut valid_ids).await;
    let app = App::new().unwrap();
    let weak_app = app.as_weak();

    initialize_slint_properties(&weak_app, &instances, &valid_ids).await;

    let worker_instances = instances.clone();
    let expense_worker = expenses::ExpenseWorker::new(&worker_instances);
    let property_worker = properties::PropertyWorker::new(&worker_instances);
    let lessee_worker = leaseholders::LeaseholderWorker::new(&worker_instances);
    let statement_worker = statements::StatementWorker::new(&worker_instances);

    intialize_slint_callbacks(
        &app,
        valid_ids.clone(),
        &expense_worker,
        &property_worker,
        &lessee_worker,
        &statement_worker,
    );

    app.run().unwrap();

    instances.close().await;
    let _expense_result = expense_worker.join();
    let _property_result = property_worker.join();
    let _lessee_result = lessee_worker.join();
    let _statement_result = statement_worker.join();
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
    instances: &sqlx::Pool<Sqlite>,
    valid_ids: &ValidIds,
) {
    slint_conversion::initialize_slint_properties(
        &weak_app.upgrade().unwrap(),
        instances,
        valid_ids,
    )
    .await;
    slint_conversion::initialize_slint_expenses(
        &weak_app.upgrade().unwrap(), 
        instances, 
        valid_ids
    )
    .await;
    slint_conversion::initialize_slint_leaseholders(
        &weak_app.upgrade().unwrap(),
        instances,
        valid_ids,
    )
    .await;
}

fn intialize_slint_callbacks(
    app: &App,
    valid_ids: ValidIds,
    expense_worker: &expenses::ExpenseWorker,
    property_worker: &properties::PropertyWorker,
    lessee_worker: &leaseholders::LeaseholderWorker,
    statement_worker: &statements::StatementWorker,
) {
    let weak_app = app.as_weak();
    app.global::<Validation>().on_get_valid_id({
        let mut id_clone = valid_ids;
        move |input| {
            id_clone.get_id(input) as i32
        }}
    );

    app.global::<ExpenseData>().on_new_expense({
        let expense_channel = expense_worker.channel.clone();
        let local_app = weak_app.clone();
        move |input| {
            let input_clone = input.clone();
            let upgrade_res = local_app.upgrade_in_event_loop({
                let internal_channel = expense_channel.clone();
                move |handle| {
                    let prev_expense = handle.global::<ExpenseData>().get_expenses();
                    let new_expenses = prev_expense
                        .as_any()
                        .downcast_ref::<slint::VecModel<ExpenseInput>>()
                        .expect("Expenses failed to downcast");
                    let message = match input_clone.message {
                        MessageType::Create => {
                            new_expenses.push(input_clone);
                            expenses::ExpenseMessage::ExpenseCreated(input)
                        }
                        MessageType::Update => {
                            let index = match new_expenses
                                .iter()
                                .position(|r| {
                                    //println!("r.id: {}. input_clone.id: {}", r.id, input_clone.id);
                                    r.id == input_clone.id
                                }) {
                                    Some(i) => i,
                                    None => panic!("Failed to find correct index for Expense in collection"),
                                };
                            new_expenses.remove(index);
                            new_expenses.insert(index, input_clone);
                            expenses::ExpenseMessage::ExpenseUpdate(input)
                        }
                        MessageType::Delete => {
                            let index = new_expenses
                                .iter()
                                .position(|r| {
                                    //println!("r.id: {}. input_clone.id: {}", r.id, input_clone.id);
                                    r.id == input_clone.id
                                })
                                .unwrap();
                            new_expenses.remove(index);
                            expenses::ExpenseMessage::ExpenseDelete(input)
                        }
                    };
                    let res = internal_channel.send(message);
                    match res {
                        Ok(_) => (), //println!("expense successfully sent"),
                        Err(_e) => println!("expense send failed"),
                    }
                }
            });
            match upgrade_res {
                Ok(_) => (),
                Err(e) => println!("Failed to upgrade ui: {e}"),
            };
        }
    });

    app.global::<PropertyData>().on_new_property({
        let property_channel = property_worker.channel.clone();
        let local_app = weak_app.clone();
        move |input| {
            let input_clone = input.clone();
            let upgrade_res = local_app.upgrade_in_event_loop({
                let internal_channel = property_channel.clone();
                move |handle| {
                    let prev_property = handle.global::<PropertyData>().get_properties();
                    let new_properties = prev_property
                        .as_any()
                        .downcast_ref::<slint::VecModel<PropertyInput>>()
                        .expect("Properties failed to downcast");
                    let message = match input_clone.message {
                        crate::MessageType::Create => {
                            new_properties.push(input_clone);
                            properties::PropertyMessage::PropertyCreated(input)
                        }
                        crate::MessageType::Update => {
                            let index = new_properties
                                .iter()
                                .position(|r| {
                                    //println!("r.id: {}. input_clone.id: {}", r.id, input_clone.id);
                                    r.id == input_clone.id
                                })
                                .unwrap();
                            new_properties.remove(index);
                            new_properties.insert(index, input_clone);
                            properties::PropertyMessage::PropertyUpdate(input)
                        }
                        crate::MessageType::Delete => {
                            let index = new_properties
                                .iter()
                                .position(|r| {
                                    //println!("r.id: {}. input_clone.id: {}", r.id, input_clone.id);
                                    r.id == input_clone.id
                                })
                                .unwrap();
                            new_properties.remove(index);
                            properties::PropertyMessage::PropertyRemove(input)
                        }
                    };
                    let res = internal_channel.send(message);
                    match res {
                        Ok(_) => (), //println!("property successfully sent"),
                        Err(_e) => println!("property send failed"),
                    };
                }
            });
            match upgrade_res {
                Ok(_) => (),
                Err(e) => println!("Failed to upgrade ui: {e}"),
            };
        }
    });

    app.global::<LesseeData>().on_new_lessee({
        let lessee_channel = lessee_worker.channel.clone();
        let local_app = weak_app.clone();
        move |input| {
            let input_clone = input.clone();
            let upgrade_res = local_app.upgrade_in_event_loop({
                let internal_channel = lessee_channel.clone();
                move |handle| {
                    let prev_lessees = handle.global::<LesseeData>().get_lessees();
                    let new_lessees = prev_lessees
                        .as_any()
                        .downcast_ref::<slint::VecModel<LeaseholderInput>>()
                        .expect("Properties failed to downcast");
                    let message = match input_clone.message {
                        crate::MessageType::Create => {
                            new_lessees.push(input_clone);
                            leaseholders::LeaseholderMessage::LeaseholderCreated(input)
                        }
                        crate::MessageType::Update => {
                            let index = new_lessees
                                .iter()
                                .position(|r| {
                                    //println!("r.id: {}. input_clone.id: {}", r.id, input_clone.id);
                                    r.id == input_clone.id
                                })
                                .unwrap();
                            new_lessees.remove(index);
                            new_lessees.insert(index, input_clone);
                            leaseholders::LeaseholderMessage::LeaseholderUpdate(input)
                        }
                        crate::MessageType::Delete => {
                            let index = new_lessees
                                .iter()
                                .position(|r| {
                                    //println!("r.id: {}. input_clone.id: {}", r.id, input_clone.id);
                                    r.id == input_clone.id
                                })
                                .unwrap();
                            new_lessees.remove(index);
                            leaseholders::LeaseholderMessage::LeaseholderDelete(input)
                        }
                    };
                    let res = internal_channel.send(message);
                    match res {
                        Ok(_) => (), //println!("Leaseholder successfully sent"),
                        Err(_e) => println!("Leaseholder send failed"),
                    };
                }
            });
            match upgrade_res {
                Ok(_) => (),
                Err(e) => println!("Failed to upgrade ui: {e}"),
            };
        }
    });

    app.global::<StatementData>().on_new_statement({
        let statement_channel = statement_worker.channel.clone();
        let local_app = weak_app.clone();
        move |input| {
            let input_clone = input.clone();
            let upgrade_res = local_app.upgrade_in_event_loop({
                let internal_channel = statement_channel.clone();
                move |handle| {
                    let prev_statements = handle.global::<StatementData>().get_statements();
                    let new_statements = prev_statements
                        .as_any()
                        .downcast_ref::<slint::VecModel<StatementInput>>()
                        .expect("Statements failed to downcast");
                    
                    let message = match input.message {
                        crate::MessageType::Create => {
                            new_statements.push(input_clone);
                            statements::StatementMessage::StatementCreated(input)
                        }
                        crate::MessageType::Delete => {
                            let index = new_statements
                                .iter()
                                .position(|r| {
                                    //println!("r.id: {}. input_clone.id: {}", r.id, input_clone.id);
                                    r.id == input_clone.id
                                })
                                .unwrap();
                            new_statements.remove(index);
                            statements::StatementMessage::StatementDelete(input)
                        }
                        crate::MessageType::Update => {
                            let index = new_statements
                                .iter()
                                .position(|r| {
                                    //println!("r.id: {}. input_clone.id: {}", r.id, input_clone.id);
                                    r.id == input_clone.id
                                })
                                .unwrap();
                            new_statements.remove(index);
                            new_statements.insert(index, input_clone);
                            statements::StatementMessage::StatementUpdate(input)
                        }
                    };
                    let res = internal_channel.send(message);
                    match res {
                        Ok(_) => (), //println!("Statement successfully sent"),
                        Err(_e) => println!("Statement send failed"),
                    };
                }
            });
            match upgrade_res {
                Ok(_) => (),
                Err(e) => println!("Failed to upgrade ui: {e}"),
            }
        }
    });
}
