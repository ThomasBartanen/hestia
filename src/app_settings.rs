pub const TESTING_STATEMENT_PATH: &str = "E:/!Coding/Rust/hestia/statements/";

pub struct PathSettings {
    pub statements_path: String,
}

impl Default for PathSettings {
    fn default() -> Self {
        PathSettings {
            statements_path: TESTING_STATEMENT_PATH.to_owned(),
        }
    }
}
