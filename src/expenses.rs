use std::fmt;

use chrono::NaiveDate;
use sqlx::{sqlite::SqliteRow, FromRow, Row};

#[derive(Debug, Clone)]
pub enum ExpenseType {
    Maintenance(MaintenanceType),
    Utilities(UtilitiesType),
    Other,
}

#[derive(Debug, Clone)]
pub enum MaintenanceType {
    Repairs,
    Cleaning,
    Landscaping,
    Other,
}

#[derive(Debug, Clone)]
pub enum UtilitiesType {
    Water,
    Electricity,
    Garbage,
    Gas,
    Other,
}

#[derive(Debug, Clone)]
pub enum RequestStatus {
    Received,
    InProgress,
    Completed,
    Cancelled,
    OnHold,
}

impl fmt::Display for RequestStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RequestStatus::Received => write!(f, "RequestStatus: Received"),
            RequestStatus::InProgress => write!(f, "RequestStatus: In Progress"),
            RequestStatus::Completed => write!(f, "RequestStatus: Completed"),
            RequestStatus::Cancelled => write!(f, "RequestStatus: Cancelled"),
            RequestStatus::OnHold => write!(f, "RequestStatus: On Hold"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaintenanceRequest {
    pub request_id: u16,
    pub tenant_id: u16,
    pub request_date: NaiveDate,
    pub request_type: MaintenanceType,
    pub description: String,
    pub status: RequestStatus,
    pub completion_date: Option<NaiveDate>,
}

#[derive(Debug, Clone)]
pub struct Expense {
    pub property_id: u16,
    pub expense_type: ExpenseType,
    pub amount: f32,
    pub date: NaiveDate,
    pub description: String,
}

impl Expense {
    pub fn new(
        property_id: u16,
        expense_type: ExpenseType,
        amount: f32,
        date: NaiveDate,
        description: String,
    ) -> Expense {
        Expense {
            property_id,
            expense_type,
            amount,
            date,
            description,
        }
    }
}
impl<'r> FromRow<'r, SqliteRow> for Expense {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let property_id = row.try_get("property_id")?;
        let expense_type: String = row.try_get("expense_type")?;
        let amount = row.try_get("amount")?;
        let date: String = row.try_get("date_incurred")?;
        let description = row.try_get("description")?;

        let mut parts = expense_type.split(':');
        let expense_main_type = parts.next().unwrap_or("").trim();
        let expense_sub_type = parts.next().unwrap_or("").trim();

        let expense_type = match expense_main_type {
            "Maintenance" => match expense_sub_type {
                "Repairs" => ExpenseType::Maintenance(MaintenanceType::Repairs),
                "Cleaning" => ExpenseType::Maintenance(MaintenanceType::Cleaning),
                "Landscaping" => ExpenseType::Maintenance(MaintenanceType::Landscaping),
                _ => ExpenseType::Maintenance(MaintenanceType::Other),
            },
            "Utilities" => match expense_sub_type {
                "Water" => ExpenseType::Utilities(UtilitiesType::Water),
                "Electricity" => ExpenseType::Utilities(UtilitiesType::Electricity),
                "Garbage" => ExpenseType::Utilities(UtilitiesType::Garbage),
                "Gas" => ExpenseType::Utilities(UtilitiesType::Gas),
                _ => ExpenseType::Utilities(UtilitiesType::Other),
            },
            _ => ExpenseType::Other,
        };

        let naive_date = NaiveDate::parse_from_str(date.as_str(), "%Y-%m-%d")
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        Ok(Expense {
            property_id,
            expense_type,
            amount,
            date: naive_date,
            description,
        })
    }
}
