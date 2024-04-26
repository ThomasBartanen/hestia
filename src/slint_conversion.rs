use crate::ExpenseInput;
use crate::{database::get_expenses, App, Expense};
use slint::{ModelRc, VecModel};
use sqlx::Sqlite;

pub async fn initialize_slint_expenses(ui: &App, pool: &sqlx::Pool<Sqlite>, property_id: u16) {
    let expenses: Vec<ExpenseInput> = get_expenses(pool, property_id)
        .await
        .iter()
        .map(Expense::convert_to_slint)
        .collect();

    let converted_expenses = ModelRc::new(VecModel::from(expenses));

    ui.set_expenses(converted_expenses);
}
