use crate::{lease::ContactInformation, properties::Address};

pub struct Company {
    pub name: String,
    pub remittence_address: Address,
    pub tax_id_number: u32,
    pub contact_info: ContactInformation,
}

impl Company {
    pub fn new(
        name: String,
        remittence_address: Address,
        tax_id_number: u32,
        contact_info: ContactInformation,
    ) -> Company {
        Company {
            name,
            remittence_address,
            tax_id_number,
            contact_info,
        }
    }
}
