use chrono::{DateTime, NaiveDate, Utc};

#[derive(Debug)]
pub enum ExpenseType {
    Maintenance(MaintenanceType),
    Utilities(UtilitiesType),
    Other,
}

#[derive(Debug)]
pub enum MaintenanceType {
    Repairs,
    Cleaning,
    Landscaping,
    Other,
}

#[derive(Debug)]
pub enum UtilitiesType {
    Water,
    Electricity,
    Gas,
    Other,
}

pub enum RequestStatus {
    Received,
    InProgress,
    Completed,
    Cancelled,
    OnHold
}

pub struct MaintenanceRequest {
    pub request_id: u16,
    pub tenant_id: u16,
    pub request_date: NaiveDate,
    pub request_type: MaintenanceType,
    pub description: String,
    pub status: RequestStatus,
    pub completion_date: Option<NaiveDate>
}

pub struct Expense {
    pub property_id: u16,
    pub expense_type: ExpenseType,
    pub amount: f64,
    pub date: DateTime<Utc>,
    pub description: String
}

impl Expense {
    pub fn new(property_id: u16, expense_type: ExpenseType, amount: f64, date: DateTime<Utc>, description: String) -> Expense {
        Expense {
            property_id,
            expense_type,
            amount,
            date,
            description
        }
    }
}