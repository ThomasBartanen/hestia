use std::fmt;

use crate::{database::add_expense, App, ExpenseInput};
use chrono::NaiveDate;
use sqlx::{sqlite::SqliteRow, FromRow, Row};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

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

impl fmt::Display for ExpenseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res = match self {
            ExpenseType::Maintenance(maintenance_type) => match maintenance_type {
                MaintenanceType::Repairs => String::from("Maintenance: Repairs"),
                MaintenanceType::Cleaning => String::from("Maintenance: Cleaning"),
                MaintenanceType::Landscaping => String::from("Maintenance: Landscaping"),
                MaintenanceType::Other => String::from("Maintenance: Other"),
            },
            ExpenseType::Utilities(utilities_type) => match utilities_type {
                UtilitiesType::Water => String::from("Utilities: Water"),
                UtilitiesType::Electricity => String::from("Utilities: Electricity"),
                UtilitiesType::Garbage => String::from("Utilities: Garbage/Recycle"),
                UtilitiesType::Gas => String::from("Utilities: Gas"),
                UtilitiesType::Other => String::from("Utilities: Other"),
            },
            ExpenseType::Other => String::from("Other"),
        };
        write!(f, "{res}")
    }
}

impl ExpenseType {
    pub fn parse_string(maintype: &str, subtype: &str) -> ExpenseType {
        match maintype {
            "Maintenance" => match subtype {
                "Repairs" => ExpenseType::Maintenance(MaintenanceType::Repairs),
                "Cleaning" => ExpenseType::Maintenance(MaintenanceType::Cleaning),
                "Landscaping" => ExpenseType::Maintenance(MaintenanceType::Landscaping),
                _ => ExpenseType::Maintenance(MaintenanceType::Other),
            },
            "Utilities" => match subtype {
                "Water" => ExpenseType::Utilities(UtilitiesType::Water),
                "Electricity" => ExpenseType::Utilities(UtilitiesType::Electricity),
                "Garbage" => ExpenseType::Utilities(UtilitiesType::Garbage),
                "Gas" => ExpenseType::Utilities(UtilitiesType::Gas),
                _ => ExpenseType::Utilities(UtilitiesType::Other),
            },
            _ => ExpenseType::Other,
        }
    }
    pub fn to_split_strings(&self) -> (String, String) {
        match self {
            ExpenseType::Maintenance(maintenance_type) => match maintenance_type {
                MaintenanceType::Repairs => (String::from("Maintenance"), String::from("Repairs")),
                MaintenanceType::Cleaning => {
                    (String::from("Maintenance"), String::from("Cleaning"))
                }
                MaintenanceType::Landscaping => {
                    (String::from("Maintenance"), String::from("Landscaping"))
                }
                MaintenanceType::Other => (String::from("Maintenance"), String::from("Other")),
            },
            ExpenseType::Utilities(utilities_type) => match utilities_type {
                UtilitiesType::Water => (String::from("Utilities"), String::from("Water")),
                UtilitiesType::Electricity => {
                    (String::from("Utilities"), String::from("Electricity"))
                }
                UtilitiesType::Garbage => (String::from("Utilities"), String::from("Garbage")),
                UtilitiesType::Gas => (String::from("Utilities"), String::from("Gas")),
                UtilitiesType::Other => (String::from("Utilities"), String::from("Other")),
            },
            ExpenseType::Other => (String::from("Other"), String::from("")),
        }
    }
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
    pub request_id: u32,
    pub leaseholder_id: u32,
    pub request_date: NaiveDate,
    pub request_type: MaintenanceType,
    pub description: String,
    pub status: RequestStatus,
    pub completion_date: Option<NaiveDate>,
}

#[derive(Debug, Clone)]
pub struct Expense {
    pub id: u32,
    pub property_id: u32,
    pub expense_type: ExpenseType,
    pub amount: f32,
    pub date: NaiveDate,
    pub description: String,
}

impl Expense {
    pub fn new(
        property_id: u32,
        expense_type: ExpenseType,
        amount: f32,
        date: NaiveDate,
        description: String,
    ) -> Expense {
        Expense {
            id: 0,
            property_id,
            expense_type,
            amount,
            date,
            description,
        }
    }
    pub fn convert_from_slint(input: ExpenseInput) -> Expense {
        Expense::new(
            1,
            ExpenseType::parse_string(input.expense_type.as_str(), input.expense_subtype.as_str()),
            input.amount,
            NaiveDate::from_ymd_opt(2022, 3, 3).unwrap(),
            input.description.to_string(),
        )
    }

    pub fn convert_to_slint(&self) -> ExpenseInput {
        let (main, sub) = ExpenseType::to_split_strings(&self.expense_type);
        let cur_expense = self.clone();
        ExpenseInput {
            amount: cur_expense.amount,
            date: cur_expense.date.to_string().into(),
            description: cur_expense.description.into(),
            expense_subtype: sub.into(),
            expense_type: main.into(),
        }
    }
}
impl<'r> FromRow<'r, SqliteRow> for Expense {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let id = row.try_get("expense_id")?;
        let property_id = row.try_get("property_id")?;
        let expense_type: String = row.try_get("expense_type")?;
        let amount = row.try_get("amount")?;
        let date: String = row.try_get("date_incurred")?;
        let description = row.try_get("description")?;

        let mut parts = expense_type.split(':');
        let expense_main_type = parts.next().unwrap_or("").trim();
        let expense_sub_type = parts.next().unwrap_or("").trim();

        let expense_type = ExpenseType::parse_string(expense_main_type, expense_sub_type);

        let naive_date = NaiveDate::parse_from_str(date.as_str(), "%Y-%m-%d")
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        Ok(Expense {
            id,
            property_id,
            expense_type,
            amount,
            date: naive_date,
            description,
        })
    }
}

pub enum ExpenseMessage {
    ExpenseCreated(ExpenseInput),
    ExpenseUpdate(ExpenseInput),
    Quit,
}

pub struct ExpenseWorker {
    pub channel: UnboundedSender<ExpenseMessage>,
    pub worker_thread: std::thread::JoinHandle<()>,
}

impl ExpenseWorker {
    pub fn new(pool: &sqlx::Pool<sqlx::Sqlite>) -> Self {
        println!("Create new Expense Worker");
        let (sender, r) = tokio::sync::mpsc::unbounded_channel();
        let worker_thread = std::thread::spawn({
            let new_pool = pool.clone();
            move || {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(expense_worker_loop(new_pool, r))
            }
        });
        Self {
            channel: sender,
            worker_thread,
        }
    }
    pub fn join(self) -> std::thread::Result<()> {
        let _ = self.channel.send(ExpenseMessage::Quit);
        self.worker_thread.join()
    }
}

async fn expense_worker_loop(
    pool: sqlx::Pool<sqlx::Sqlite>,
    mut r: UnboundedReceiver<ExpenseMessage>,
) {
    loop {
        let m = r.recv().await;

        let res = match m {
            Some(s) => match s {
                ExpenseMessage::ExpenseCreated(create) => create,
                ExpenseMessage::ExpenseUpdate(update) => update,
                ExpenseMessage::Quit => {
                    println!("Quitting");
                    continue;
                }
            },
            None => continue,
        };

        let converted_expense = Expense::convert_from_slint(res);

        match add_expense(&pool, &converted_expense).await {
            Ok(_) => println!("Successfully added expense via slint"),
            Err(e) => println!("Failed to add expense via slint: {e}"),
        }
    }
}
