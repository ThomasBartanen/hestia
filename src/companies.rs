use crate::{properties::Address, tenant::ContactInformation};

pub struct Company {
    pub name: String,
    pub remittence_address: Address,
    pub contact_info: ContactInformation,
}

impl Company {
    pub fn new(
        name: String,
        remittence_address: Address,
        contact_info: ContactInformation,
    ) -> Company {
        Company {
            name,
            remittence_address,
            contact_info,
        }
    }
}
