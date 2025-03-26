use crate::models::*;

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
}