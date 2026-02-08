use sqlx::{Sqlite, FromRow};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::properties::Property;


#[derive(Clone)]
pub struct DatabaseManager {
    pub db_pool: sqlx::Pool<Sqlite>
}

impl DatabaseManager {
    pub async fn new(pool: sqlx::Pool<Sqlite>) -> DatabaseManager {
        DatabaseManager {
            db_pool: pool
        }
    }
}

pub enum DatabaseOperation {
    Create(DatabaseTable),
    Query(DatabaseTable),
    Update(DatabaseTable),
    Delete(DatabaseTable),
    Close,
}

pub enum DatabaseTable {
    Property(crate::properties::Property),
    Leaseholder(crate::leaseholders::Leaseholder),
    Expense(crate::expenses::Expense),
    Statement(crate::statements::Statement),
}

pub struct DatabaseWorker {
    pub channel: UnboundedSender<DatabaseOperation>,
    pub worker_thread: std::thread::JoinHandle<()>,
}

impl DatabaseWorker {
    pub fn new(pool: DatabaseManager) -> Self {
        //println!("Create new Expense Worker");
        let (sender, r) = tokio::sync::mpsc::unbounded_channel();
        let worker_thread = std::thread::spawn({
            let new_pool = pool;
            move || {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(database_worker_loop(new_pool, r))
            }
        });
        Self {
            channel: sender,
            worker_thread,
        }
    }
    pub fn join(self) -> std::thread::Result<()> {
        let _ = self.channel.send(DatabaseOperation::Close);
        self.worker_thread.join()
    }
}

async fn database_worker_loop(conn: DatabaseManager, mut r: UnboundedReceiver<DatabaseOperation>) {
    loop {
        match r.recv().await.unwrap() {
            DatabaseOperation::Create(t) => match t {
                DatabaseTable::Property(property) => match crate::database::add_property(&conn.db_pool, &property).await {
                    Ok(o) => println!("Successfully inserted new property. Row: {:?}", o),
                    Err(e) => println!("Error inserting new property: {e}"),
                },
                DatabaseTable::Leaseholder(leaseholder) => match crate::database::add_leaseholders(&conn.db_pool, &leaseholder).await {
                    Ok(o) => println!("Successfully inserted new leaseholder. Row: {:?}", o),
                    Err(e) => println!("Error inserting new leaseholder: {e}"),
                },
                DatabaseTable::Expense(expense) => match crate::database::add_expense(&conn.db_pool, &expense).await {
                    Ok(o) => println!("Successfully inserted new expense. Row: {:?}", o),
                    Err(e) => println!("Error inserting new expense: {e}"),
                },
                DatabaseTable::Statement(statement) => match crate::database::add_statement(&conn.db_pool, &statement).await {
                    Ok(o) => println!("Successfully inserted new statement. Row: {:?}", o),
                    Err(e) => println!("Error inserting new statement: {e}"),
                },
            },
            DatabaseOperation::Query(t) => match t {
                DatabaseTable::Property(property) => match crate::database::get_property(&conn.db_pool, property.id).await {
                    Ok(o) => match Property::from_row(&o) {
                        Ok(p) => println!("Successfully selected property. Row: {:?}", p),
                        Err(e) => println!("Error converting property from row: {e}"),
                    }
                    Err(e) => println!("Error selecting property: {e}"),
                },
                DatabaseTable::Leaseholder(leaseholder) => match crate::database::get_leaseholder(&conn.db_pool, leaseholder.id).await {
                    Ok(o) => match crate::leaseholders::Leaseholder::from_row(&o) {
                        Ok(l) => println!("Successfully selected leaseholder. Row: {:?}", l),
                        Err(e) => println!("Error converting leaseholder from row: {e}"),
                    }
                    Err(e) => println!("Error selecting leaseholder: {e}"),
                },
                DatabaseTable::Expense(expense) => match crate::database::get_expense(&conn.db_pool, expense.id).await {
                    Ok(o) => match crate::expenses::Expense::from_row(&o) {
                        Ok(e) => println!("Successfully selected expense. Row: {:?}", e),
                        Err(e) => println!("Error converting expense from row: {e}"),
                    }
                    Err(e) => println!("Error selecting expense: {e}"),
                },
                DatabaseTable::Statement(statement) => match crate::database::get_statement(&conn.db_pool, statement.id).await {
                    Ok(o) => match crate::statements::Statement::from_row(&o) {
                        Ok(s) => println!("Successfully selected statement. Row: {:?}", s),
                        Err(e) => println!("Error converting statement from row: {e}"),
                    }
                    Err(e) => println!("Error selecting statement: {e}"),
                },
            },
            DatabaseOperation::Update(t) => match t {
                DatabaseTable::Property(property) => match crate::database::update_property(&conn.db_pool, &property).await {
                    Ok(o) => println!("Successfully updated property. Row: {:?}", o),
                    Err(e) => println!("Error updating property: {e}"),
                },
                DatabaseTable::Leaseholder(leaseholder) => match crate::database::update_leaseholder(&conn.db_pool, &leaseholder).await {
                    Ok(o) => println!("Successfully updated leaseholder. Row: {:?}", o),
                    Err(e) => println!("Error updating leaseholder: {e}"),
                },
                DatabaseTable::Expense(expense) => match crate::database::update_expense(&conn.db_pool, &expense).await {
                    Ok(o) => println!("Successfully updated expense. Row: {:?}", o),
                    Err(e) => println!("Error updating expense: {e}"),
                },
                DatabaseTable::Statement(statement) => match crate::database::update_statement(&conn.db_pool, &statement).await {
                    Ok(o) => println!("Successfully updated statement. Row: {:?}", o),
                    Err(e) => println!("Error updating statement: {e}"),
                },
            },
            DatabaseOperation::Delete(t) => match t {
                DatabaseTable::Property(property) => match crate::database::remove_property(&conn.db_pool, &property).await {
                    Ok(o) => println!("Successfully deleted property. Row: {:?}", o),
                    Err(e) => println!("Error deleting property: {e}"),
                },
                DatabaseTable::Leaseholder(leaseholder) => match crate::database::remove_leaseholder(&conn.db_pool, &leaseholder).await {
                    Ok(o) => println!("Successfully deleted leaseholder. Row: {:?}", o),
                    Err(e) => println!("Error deleting leaseholder: {e}"),
                },
                DatabaseTable::Expense(expense) => match crate::database::remove_expense(&conn.db_pool, &expense).await {
                    Ok(o) => println!("Successfully deleted expense. Row: {:?}", o),
                    Err(e) => println!("Error deleting expense: {e}"),
                },
                DatabaseTable::Statement(statement) => match crate::database::remove_statement(&conn.db_pool, &statement).await {
                    Ok(o) => println!("Successfully deleted statement. Row: {:?}", o),
                    Err(e) => println!("Error deleting statement: {e}"),
                },
            },
            DatabaseOperation::Close => println!("Closing app"),
        };
    }
}