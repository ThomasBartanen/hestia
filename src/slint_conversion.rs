use crate::properties::Property;
use crate::{database::get_expenses, App, Expense};
use crate::{ExpenseInput, PropertyInput};
use slint::{ModelRc, VecModel};
use sqlx::Sqlite;

pub async fn initialize_slint_expenses(ui: &App, pool: &sqlx::Pool<Sqlite>, property_id: u32) {
    let expenses: Vec<ExpenseInput> = get_expenses(pool, property_id)
        .await
        .iter()
        .map(Expense::convert_to_slint)
        .collect();

    let converted_expenses = ModelRc::new(VecModel::from(expenses));

    ui.set_expenses(converted_expenses);
}

pub async fn initialize_slint_properties(ui: &App, pool: &sqlx::Pool<Sqlite>) {
    let expenses: Vec<PropertyInput> = crate::database::get_properties(pool)
        .await
        .iter()
        .map(Property::convert_to_slint)
        .collect();

    let converted_expenses = ModelRc::new(VecModel::from(expenses));

    ui.set_properties(converted_expenses);
}
