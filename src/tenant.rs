use chrono::NaiveDate;

pub struct Tenant {
    pub lease_id: u16, 
    pub property_id: u16,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: String,
    pub move_in_date: NaiveDate
}

impl Tenant {
    pub fn new(lease_id: u16, property_id: u16, first_name: String, last_name: String, email: String, phone_number: String, move_in_date: NaiveDate
    ) -> Tenant {
        Tenant {
            lease_id,
            property_id,
            first_name,
            last_name,
            email,
            phone_number,
            move_in_date      
        }
    }
}

pub struct Lease {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub monthly_rent: f32,
    pub payment_due_date: u16,
    pub payment_method: String
}

impl Lease {
    pub fn new(start_date: NaiveDate, end_date: NaiveDate, monthly_rent: f32, payment_due_date: u16, payment_method: String
    ) -> Lease {
        Lease {
            start_date,
            end_date,
            monthly_rent,
            payment_due_date,
            payment_method
        }
    }
}