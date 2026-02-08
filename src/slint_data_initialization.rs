use crate::leaseholders::Leaseholder;
use crate::properties::Property;
use crate::{expenses::*, App};
use crate::{ExpenseData, LesseeData, PropertyData, ExpenseInput, PropertyInput, ValidIds};
use slint::{ModelRc, VecModel, ComponentHandle};
use sqlx::Sqlite;

pub async fn initialize_slint_expenses<'a>(ui: & App, pool: &'a sqlx::Pool<Sqlite>, max_ids: & ValidIds) -> &'a sqlx::Pool<Sqlite> {
    let (expenses, pool) = crate::database::get_all_expenses(pool).await;
    let expenses: Vec<ExpenseInput> = expenses
        .iter()
        .map(Expense::convert_to_slint)
        .collect();

    let converted_expenses = ModelRc::new(VecModel::from(expenses));
    ui.global::<ExpenseData>().set_potential_expense_id(max_ids.expense_id as i32);
    ui.global::<ExpenseData>().set_expenses(converted_expenses);
    pool
}

pub async fn initialize_slint_properties<'a>(ui: &App, pool: &'a sqlx::Pool<Sqlite>, max_ids: &ValidIds) -> &'a sqlx::Pool<Sqlite> {
    let (props, pool) = crate::database::get_properties(pool).await;
    let props: Vec<PropertyInput> = props
        .iter()
        .map(Property::convert_to_slint)
        .collect();

    let converted_props = ModelRc::new(VecModel::from(props));
    ui.global::<PropertyData>().set_potential_prop_id(max_ids.property_id as i32);
    ui.global::<PropertyData>().set_properties(converted_props);
    pool
}

pub async fn initialize_slint_leaseholders<'a>(
    ui: &App,
    pool: &'a sqlx::Pool<Sqlite>,
    max_ids: &ValidIds,
) -> &'a sqlx::Pool<Sqlite> {
    let (leaseholders, pool) = crate::database::get_leaseholders(pool).await;
    let leaseholders: Vec<crate::LeaseholderInput> = leaseholders
        .iter()
        .map(Leaseholder::convert_to_slint)
        .collect();

    let converted_leaseholders = ModelRc::new(VecModel::from(leaseholders));
    ui.global::<LesseeData>().set_potential_lessee_id(max_ids.leaseholder_id as i32);
    ui.global::<LesseeData>().set_lessees(converted_leaseholders);
    pool
}
