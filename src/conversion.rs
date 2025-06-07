use chrono::{Datelike, NaiveDate};
use slint::{ModelRc, ToSharedString, VecModel};

use crate::{models::*, slint_generatedApp, TransactionInfo, TenantInfo};

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

impl Transaction {
    pub fn to_slint(&self) -> crate::slint_generatedApp::TransactionInfo {
        crate::slint_generatedApp::TransactionInfo {
            id: self.id,
            prop_id: {
                match self.property_id {
                    Some(n) => n,
                    None => -1
                }
            },
            expense_type: self.transaction_type.to_shared_string(),
            amount: self.amount,
            date: slint_generatedApp::Date {
                month: self.date.month() as i32,
                day: self.date.day0() as i32,
                year: self.date.year_ce().1 as i32
            },
            description: self.description.to_shared_string(),
        }
    }
    pub fn from_slint(input: TransactionInfo) -> Transaction {
        Transaction {
            id: input.id,
            property_id: Some(input.prop_id),
            transaction_type: input.expense_type.to_string(),
            amount: input.amount,
            date: NaiveDate::from_ymd_opt(input.date.year, input.date.month as u32, input.date.day as u32).unwrap(),
            description: input.description.to_string(),
        }
    }
}