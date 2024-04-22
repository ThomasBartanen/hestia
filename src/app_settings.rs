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
