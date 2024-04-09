use chrono::NaiveDate;

use crate::{lease::Lease, properties::Address};

#[derive(Debug, Clone)]
pub struct ContactInformation {
    pub remittence_address: Address,
    pub email: String,
    pub phone_number: String,
}

impl ContactInformation {
    pub fn new(
        remittence_address: Address,
        email: String,
        phone_number: String,
    ) -> ContactInformation {
        ContactInformation {
            remittence_address,
            email,
            phone_number,
        }
    }
    pub fn get_address_string(&self) -> String {
        format!(
            "{} {}, {} {}",
            self.remittence_address.street_address,
            self.remittence_address.city,
            self.remittence_address.state,
            self.remittence_address.zip_code
        )
    }
}

#[derive(Debug, Clone)]
pub struct Leaseholder {
    pub id: u16,
    pub lease: Lease,
    pub property_id: u16,
    pub leaseholder_type: LeaseholderType,
    pub contact_info: ContactInformation,
    pub move_in_date: NaiveDate,
}

impl Leaseholder {
    pub fn new(
        id: u16,
        lease: Lease,
        property_id: u16,
        leaseholder_type: LeaseholderType,
        contact_info: ContactInformation,
        move_in_date: NaiveDate,
    ) -> Leaseholder {
        Leaseholder {
            id,
            lease,
            property_id,
            leaseholder_type,
            contact_info,
            move_in_date,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Individual {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone)]
pub struct Company {
    pub name: String,
    pub tax_id_number: u32,
}

impl Company {
    pub fn new(name: String, tax_id_number: u32) -> Company {
        Company {
            name,
            tax_id_number,
        }
    }
}

#[derive(Debug, Clone)]
pub enum LeaseholderType {
    CompanyLeaseholder(Company),
    IndividualLeaseholder(Individual),
}

impl LeaseholderType {
    pub fn get_name(&self) -> String {
        match self {
            LeaseholderType::CompanyLeaseholder(c) => c.name.to_string(),
            LeaseholderType::IndividualLeaseholder(i) => {
                format!("{} {}", i.first_name, i.last_name)
            }
        }
    }
}
