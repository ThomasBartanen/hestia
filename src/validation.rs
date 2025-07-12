use crate::{database::DatabaseManager, models::{Property, Tenant, Unit}};

#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }
}

pub trait Validatable {
    fn validate(&self) -> ValidationResult;
}

// Tenant Validation
pub struct TenantValidator;

impl TenantValidator {
    pub fn validate_name(name: String) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        if name.trim().is_empty() {
            result.add_error("Tenant name cannot be empty".to_string());
        }
        
        // Basic name validation (letters, spaces, hyphens)
        if !name.chars().all(|c| c.is_ascii_alphabetic() || c.is_whitespace() || c == '-') {
            result.add_error("Invalid characters in tenant name".to_string());
        }
        
        result
    }

    pub fn validate_email(email: String) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        if email.trim().is_empty() {
            result.add_error("Email cannot be empty".to_string());
        }
        
        // Simple email validation
        if !(email.contains('@') && email.split('@').nth(1).unwrap_or("").contains('.')) {
            result.add_error("Invalid email format".to_string());
        }
        
        result
    }
    /*
    pub async fn validate_unique_email(
        db_manager: &DatabaseManager,
        email: String,
        exclude_id: Option<i32>
    ) -> Result<ValidationResult, Error> {
        let mut result = ValidationResult::new();
        
        let count = db_manager.conn.prepare(
            "SELECT COUNT(*) FROM tenants 
             WHERE id != COALESCE(?1, 0) AND LOWER(email) = LOWER(?2)"
        )?.query_row([&exclude_id, &email.to_lowercase()], |row| row.get(0))?;
        
        if count > 0 {
            result.add_error("Tenant with this email already exists".to_string());
        }
        
        Ok(result)
    }
    */
}

impl Validatable for Property {
    fn validate(&self) -> ValidationResult {
        let mut result = ValidationResult::new();

        /*
        pub async fn validate_unique_name(
            db_manager: &DatabaseManager,
            name: String,
            exclude_id: Option<i32>
        ) -> Result<ValidationResult, Error> {
            let mut result = ValidationResult::new();
            
            let count = db_manager.conn.prepare(
                "SELECT COUNT(*) FROM buildings 
                WHERE id != COALESCE(?1, 0) AND LOWER(name) = LOWER(?2)"
            )?.query_row([&exclude_id, &name.to_lowercase()], |row| row.get(0))?;
            
            if count > 0 {
                result.add_error("Building with this name already exists".to_string());
            }
            
            Ok(result)
        }
        */

        // ---- Validate Name ----
        if self.name.trim().is_empty() {
            result.add_error("Building name cannot be empty".to_string());
        }
        
        if self.name.len() > 100 {
            result.add_error("Building name must be less than 100 characters".to_string());
        }
        
        result
    }
}

impl Validatable for Unit {
    fn validate(&self) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // ---- Validate Unit Number ----
        if self.unit_number.trim().is_empty() {
            result.add_error("Unit number cannot be empty".to_string());
        }
        
        // Allow alphanumeric plus common unit separators
        if !self.unit_number.chars().all(|c| c.is_alphanumeric() || "-/#".contains(c)) {
            result.add_error("Invalid characters in unit number".to_string());
        }

        // ---- Validate Occupancy ----
        if self.is_occupied && self.tenant_id.is_none() {
            result.add_error("Occupied units must have a valid tenant assigned".to_string());
        }
        
        result
    }
}

impl Validatable for Tenant {
    fn validate(&self) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        let name_validation = TenantValidator::validate_name(self.name.clone());
        if !name_validation.is_valid {
            result.errors.extend(name_validation.errors);
        }
        
        let email_validation = TenantValidator::validate_email(self.email.clone());
        if !email_validation.is_valid {
            result.errors.extend(email_validation.errors);
        }
        
        result
    }
}