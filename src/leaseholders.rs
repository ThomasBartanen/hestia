use chrono::{Datelike, NaiveDate};
use sqlx::{sqlite::SqliteRow, FromRow, Row};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    database::{add_leaseholders, remove_leaseholder, update_leaseholder},
    lease::{self, CAMRates, FeeStructure, InsuranceRate, Lease, PropertyTaxRate, Rent},
    properties::Address,
    LeaseholderInput,
};

#[derive(Debug, Clone)]
pub struct ContactInformation {
    pub name: String,
    pub remittence_address: Address,
    pub email: String,
    pub phone_number: String,
}

impl ContactInformation {
    pub fn new(
        name: String,
        remittence_address: Address,
        email: String,
        phone_number: String,
    ) -> ContactInformation {
        ContactInformation {
            name,
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
    pub id: u32,
    pub lease: Lease,
    pub property_id: u32,
    pub contact_info: ContactInformation,
    pub move_in_date: NaiveDate,
}

impl Leaseholder {
    pub fn new(
        id: u32,
        lease: Lease,
        property_id: u32,
        contact_info: ContactInformation,
        move_in_date: NaiveDate,
    ) -> Leaseholder {
        Leaseholder {
            id,
            lease,
            property_id,
            contact_info,
            move_in_date,
        }
    }
    pub fn convert_to_slint(&self) -> LeaseholderInput {
        let copy = self.clone();
        LeaseholderInput {
            id: self.id as i32,
            property_id: self.property_id as i32,
            lease: self.lease.id as i32,
            message: crate::MessageType::Update,
            name: copy.contact_info.name.into(),
            city: copy.contact_info.remittence_address.city.into(),
            email: copy.contact_info.email.into(),
            move_in_date: copy.move_in_date.to_string().into(),
            phone_number: copy.contact_info.phone_number.into(),
            state: copy.contact_info.remittence_address.state.into(),
            street_address: copy.contact_info.remittence_address.street_address.into(),
            zip_code: copy.contact_info.remittence_address.zip_code.into(),
        }
    }

    pub fn convert_from_slint(lessee: LeaseholderInput) -> Leaseholder {
        Leaseholder {
            id: lessee.id as u32,
            lease: Lease {
                id: lessee.lease as u32,
                start_date: NaiveDate::from_ymd_opt(2022, 3, 3).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2023, 3, 3).unwrap(),
                payment_method: "Check".to_string(),
                fee_structure: lease::FeeStructure::TripleNet(
                    Rent { base_rent: 1700.0 },
                    PropertyTaxRate { property_tax: 10.0 },
                    InsuranceRate {
                        building_insurance: 10.0,
                    },
                    CAMRates::default(),
                ),
            },
            property_id: lessee.property_id as u32,
            contact_info: ContactInformation {
                name: lessee.name.into(),
                remittence_address: Address {
                    street_address: lessee.street_address.into(),
                    city: lessee.city.into(),
                    state: lessee.state.into(),
                    zip_code: lessee.zip_code.into(),
                },
                email: lessee.email.into(),
                phone_number: lessee.phone_number.into(),
            },
            move_in_date: NaiveDate::from_ymd_opt(2022, 3, 3).unwrap(),
        }
    }
}

impl<'r> FromRow<'r, SqliteRow> for Leaseholder {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let id: u32 = row.try_get("leaseholder_id")?;
        let lease_id: u32 = row.try_get("lease_id")?;
        let name: String = row.try_get("name")?;
        let property_id: u32 = row.try_get("property_id")?;
        let street_address: String = row.try_get("address")?;
        let city: String = row.try_get("city")?;
        let state: String = row.try_get("state")?;
        let zip_code: String = row.try_get("zip_code")?;
        let email: String = row.try_get("email")?;
        let phone_number: String = row.try_get("phone_number")?;
        let move_in_date: String = row.try_get("move_in_date")?;

        let naive_date = NaiveDate::parse_from_str(move_in_date.as_str(), "%Y-%m-%d")
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        Ok(Leaseholder {
            id,
            lease: Lease {
                id: lease_id,
                start_date: naive_date,
                end_date: NaiveDate::from_ymd_opt(2024, 3, 3).unwrap(),
                fee_structure: FeeStructure::Gross(Rent { base_rent: 1700.0 }),
                payment_method: "Check".to_string(),
            },
            property_id,
            contact_info: ContactInformation {
                name,
                remittence_address: Address {
                    street_address,
                    city,
                    state,
                    zip_code,
                },
                email,
                phone_number,
            },
            move_in_date: naive_date,
        })
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

pub enum LeaseholderMessage {
    LeaseholderCreated(LeaseholderInput),
    LeaseholderUpdate(LeaseholderInput),
    LeaseholderDelete(LeaseholderInput),
    Quit,
}

pub struct LeaseholderWorker {
    pub channel: UnboundedSender<LeaseholderMessage>,
    pub worker_thread: std::thread::JoinHandle<()>,
}

impl LeaseholderWorker {
    pub fn new(pool: &sqlx::Pool<sqlx::Sqlite>) -> Self {
        println!("Create new Leaseholder Worker");
        let (sender, r) = tokio::sync::mpsc::unbounded_channel();
        let worker_thread = std::thread::spawn({
            let new_pool = pool.clone();
            move || {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(leaseholder_worker_loop(new_pool, r))
            }
        });
        Self {
            channel: sender,
            worker_thread,
        }
    }
    pub fn join(self) -> std::thread::Result<()> {
        let _ = self.channel.send(LeaseholderMessage::Quit);
        self.worker_thread.join()
    }
}

pub async fn leaseholder_worker_loop(
    pool: sqlx::Pool<sqlx::Sqlite>,
    mut r: UnboundedReceiver<LeaseholderMessage>,
) {
    loop {
        let m = r.recv().await;

        match m {
            Some(s) => match s {
                LeaseholderMessage::LeaseholderCreated(create) => {
                    let converted_leaseholder = Leaseholder::convert_from_slint(create);
                    match add_leaseholders(
                        &pool,
                        &converted_leaseholder,
                        converted_leaseholder.property_id,
                    )
                    .await
                    {
                        Ok(_) => println!("Successfully added leaseholder via slint"),
                        Err(e) => println!("Failed to add leaseholder via slint: {e}"),
                    }
                }
                LeaseholderMessage::LeaseholderUpdate(update) => {
                    let converted_leaseholder = Leaseholder::convert_from_slint(update);
                    match update_leaseholder(&pool, &converted_leaseholder).await {
                        Ok(_) => println!("Successfully updated leaseholder via slint"),
                        Err(e) => println!("Failed to update leaseholder via slint: {e}"),
                    }
                }
                LeaseholderMessage::LeaseholderDelete(remove) => {
                    let converted_leaseholder = Leaseholder::convert_from_slint(remove);
                    match remove_leaseholder(&pool, &converted_leaseholder).await {
                        Ok(_) => println!("Successfully removed leaseholder via slint"),
                        Err(e) => println!("Failed to remove leaseholder via slint: {e}"),
                    }
                }
                LeaseholderMessage::Quit => {
                    println!("Quitting");
                    continue;
                }
            },
            None => continue,
        };
    }
}
