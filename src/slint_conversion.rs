use crate::leaseholders::Leaseholder;
use crate::properties::Property;
use crate::{expenses::*, App};
use crate::{ExpenseInput, PropertyInput, ValidIds};
use slint::{ModelRc, VecModel};
use sqlx::Sqlite;

pub async fn initialize_slint_expenses(
    ui: &App,
    pool: &sqlx::Pool<Sqlite>,
    max_ids: &ValidIds,
    property_id: u32,
) {
    let expenses: Vec<ExpenseInput> = crate::database::get_expenses(pool, property_id)
        .await
        .iter()
        .map(Expense::convert_to_slint)
        .collect();

    let converted_expenses = ModelRc::new(VecModel::from(expenses));
    ui.set_potential_expense_id(max_ids.expense_id as i32);
    ui.set_expenses(converted_expenses);
}

pub async fn initialize_slint_properties(ui: &App, pool: &sqlx::Pool<Sqlite>, max_ids: &ValidIds) {
    let expenses: Vec<PropertyInput> = crate::database::get_properties(pool)
        .await
        .iter()
        .map(Property::convert_to_slint)
        .collect();

    let converted_expenses = ModelRc::new(VecModel::from(expenses));
    ui.set_potential_prop_id(max_ids.property_id as i32);
    ui.set_properties(converted_expenses);
}

pub async fn initialize_slint_leaseholders(
    ui: &App,
    pool: &sqlx::Pool<Sqlite>,
    max_ids: &ValidIds,
) {
    let leaseholders: Vec<crate::LeaseholderInput> = crate::database::get_leaseholders(pool)
        .await
        .iter()
        .map(Leaseholder::convert_to_slint)
        .collect();

    let converted_leaseholders = ModelRc::new(VecModel::from(leaseholders));
    ui.set_potential_lessee_id(max_ids.leaseholder_id as i32);
    ui.set_lessees(converted_leaseholders);
}
