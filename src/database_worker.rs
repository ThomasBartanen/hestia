use sqlx::Sqlite;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

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
            },
            DatabaseOperation::Query(t) => match t {
                /*DatabaseTable::Property(property) => match conn.select_property(property.id) {
                    Ok(o) => println!("Successfully selected property."),
                    Err(e) => println!("Error selecting property: {e}"),
                }*/
                DatabaseTable::Property(property) => todo!(),
                DatabaseTable::Leaseholder(leaseholder) => todo!(),
                DatabaseTable::Expense(expense) => todo!(),
            },
            DatabaseOperation::Update(t) => match t {
                DatabaseTable::Property(property) => todo!(),
                DatabaseTable::Leaseholder(leaseholder) => todo!(),
                DatabaseTable::Expense(expense) => todo!(),
            },
            DatabaseOperation::Delete(t) => match t {
                DatabaseTable::Property(property) => todo!(),
                DatabaseTable::Leaseholder(leaseholder) => todo!(),
                DatabaseTable::Expense(expense) => todo!(),
            },
            DatabaseOperation::Close => println!("Closing app"),
        };
    }
}