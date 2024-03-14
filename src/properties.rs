use std::result::Result;

pub struct Property {
    pub id: u16,
    pub name: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub num_units: u16
}

impl Property {
    pub fn new(id: u16, name: String, address: String, city: String, state: String, zip_code: String, num_units: u16) -> Property {
        Property {
            id,
            name,
            address,
            city,
            state,
            zip_code,
            num_units
        }
    }
}