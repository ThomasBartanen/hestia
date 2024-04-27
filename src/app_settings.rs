use std::path::Path;

use async_std::fs;

pub const TESTING_STATEMENT_PATH: &str = "E:/!Coding/Rust/hestia/statements/";
pub const TESTING_DATABASE_PATH: &str = "E:/!Coding/Rust/hestia/";

pub struct PathSettings {
    pub statements_path: String,
    pub database_path: String,
}

impl Default for PathSettings {
    fn default() -> Self {
        PathSettings {
            statements_path: TESTING_STATEMENT_PATH.to_owned(),
            database_path: TESTING_DATABASE_PATH.to_owned(),
        }
    }
}

pub async fn initialize_data_paths() {
    match Path::new(TESTING_STATEMENT_PATH).try_exists() {
        Ok(o) => {
            if o {
                println!("Statement path already created")
            } else {
                match fs::create_dir(TESTING_STATEMENT_PATH).await {
                    Ok(_) => println!("Successfully Created Statement Directory"),
                    Err(e) => println!("Failed to create statement directory: {}", e),
                }
            }
        }
        Err(e) => println!("FAILED CHECKING STATEMENT PATH: {}", e),
    }
}
