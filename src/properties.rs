use crate::PropertyInput;
use sqlx::{sqlite::SqliteRow, Row};

#[derive(Debug, Clone)]
pub struct Property {
    pub id: u32,
    pub name: String,
    pub address: Address,
    pub property_tax: f32,
    pub business_insurance: f32,
    pub num_units: u32,
}

impl Property {
    pub fn new(
        id: u32,
        name: String,
        address: Address,
        property_tax: f32,
        business_insurance: f32,
        num_units: u32,
    ) -> Property {
        Property {
            id,
            name,
            address,
            property_tax,
            business_insurance,
            num_units,
        }
    }

    pub fn convert_from_slint(input: PropertyInput) -> Property {
        Property {
            id: input.id as u32,
            name: input.name.into(),
            address: Address {
                street_address: input.address_number.into(),
                city: input.city.into(),
                state: input.state.into(),
                zip_code: input.zip_code.into(),
            },
            property_tax: input.property_tax,
            business_insurance: input.business_insurance,
            num_units: input.unit_count as u32,
        }
    }

    pub fn convert_to_slint(&self) -> PropertyInput {
        let address = &self.address.clone();
        let address_number = &address.street_address;
        let city = &address.city;
        let state = &address.state;
        let zip_code = &address.zip_code;
        PropertyInput {
            id: self.id as i32,
            name: String::from(&self.name).into(),
            address_number: address_number.into(),
            city: city.into(),
            state: state.into(),
            zip_code: zip_code.into(),
            property_tax: self.property_tax,
            business_insurance: self.business_insurance,
            unit_count: self.num_units as i32,
        }
    }
}

impl<'r> sqlx::FromRow<'r, SqliteRow> for Property {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let id: u32 = row.try_get("property_id")?;
        let name: String = row.try_get("property_name")?;
        let tax_string: String = row.try_get("property_tax")?;
        let insurance_string: String = row.try_get("business_insurance")?;
        let address_string: String = row.try_get("address")?;
        let city_string: String = row.try_get("city")?;
        let state_string: String = row.try_get("state")?;
        let zip_string: String = row.try_get("zip_code")?;
        let num_units: u32 = row.try_get("num_units")?;

        let property_tax: f32 = tax_string.parse::<f32>().unwrap();
        let business_insurance: f32 = insurance_string.parse::<f32>().unwrap();

        let address: Address = Address::new(address_string, city_string, state_string, zip_string);

        Ok(Property {
            id,
            name,
            address,
            property_tax,
            business_insurance,
            num_units,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Address {
    pub street_address: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
}

impl Address {
    pub fn new(address: String, city: String, state: String, zip_code: String) -> Address {
        Address {
            street_address: address,
            city,
            state,
            zip_code,
        }
    }
}
