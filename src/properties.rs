#[derive(Debug, Clone)]
pub struct Property {
    pub id: u16,
    pub name: String,
    pub address: Address,
    pub property_tax: f32,
    pub business_insurance: f32,
    pub num_units: u16,
}

impl Property {
    pub fn new(
        id: u16,
        name: String,
        address: Address,
        property_tax: f32,
        business_insurance: f32,
        num_units: u16,
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
