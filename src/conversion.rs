use chrono::NaiveDate;
use slint::{ModelRc, ToSharedString, VecModel};

use crate::{models::*, ExpenseInfo, TenantInfo};

impl Tenant {
    pub fn to_slint(&self) -> crate::slint_generatedApp::TenantInfo {
        crate::slint_generatedApp::TenantInfo {
            id: self.id,
            name: self.name.clone().into(),
            lease: 0,
            property_id: 0,
            email: self.email.clone().into(),
            phone_number: self.phone.clone().into(),
            move_in_date: format!("Now").into(),
        }
    }
    pub fn from_slint(input: TenantInfo) -> Tenant {
        Tenant { 
            id: input.id, 
            name: String::from(input.name), 
            email: String::from(input.email), 
            phone: String::from(input.phone_number) 
        }
    }
}

impl Property {
    pub fn to_slint(&self) -> crate::slint_generatedApp::PropertyInfo {
        crate::slint_generatedApp::PropertyInfo {
            address: self.address.to_shared_string(),
            id: self.id,
            name: self.name.to_shared_string(),
            unit_ids: ModelRc::new(VecModel::from(self.units.clone()))
        }
    }
}

impl Expense {
    pub fn to_slint(&self) -> crate::slint_generatedApp::ExpenseInfo {
        crate::slint_generatedApp::ExpenseInfo {
            id: self.id,
            prop_id: {
                match self.property_id {
                    Some(n) => n,
                    None => -1
                }
            },
            expense_type: self.expense_type.to_shared_string(),
            amount: self.amount,
            date: self.date.to_shared_string(),
            description: self.description.to_shared_string(),
        }
    }
    pub fn from_slint(input: ExpenseInfo) -> Expense {
        Expense {
            id: input.id,
            property_id: Some(input.prop_id),
            expense_type: input.expense_type.to_string(),
            amount: input.amount,
            date: NaiveDate::parse_from_str(&input.date.to_string(), "%m/%d/%Y").unwrap_or(chrono::Utc::now().naive_utc().date()),
            description: input.description.to_string(),
        }
    }
}