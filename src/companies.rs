use crate::tenant::ContactInformation;

pub struct Company {
    pub name: String,
    pub remittence_address: String,
    pub contact_info: ContactInformation,
}

impl Company {
    pub fn new(
        name: String,
        remittence_address: String,
        contact_info: ContactInformation,
    ) -> Company {
        Company {
            name,
            remittence_address,
            contact_info,
        }
    }
}
