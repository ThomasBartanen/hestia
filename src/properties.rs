pub struct Property {
    pub id: u16,
    pub name: String,
    pub property_tax: f32,
    pub business_insurance: f32,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub num_units: u16,
}

impl Property {
    pub fn new(
        id: u16,
        name: String,
        property_tax: f32,
        business_insurance: f32,
        address: String,
        city: String,
        state: String,
        zip_code: String,
        num_units: u16,
    ) -> Property {
        Property {
            id,
            name,
            property_tax,
            business_insurance,
            address,
            city,
            state,
            zip_code,
            num_units,
        }
    }
}
