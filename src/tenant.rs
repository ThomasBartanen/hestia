use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub enum FeeStructure {
    Gross(Rent),
    SingleNet(Rent, PropertyTaxRate),
    DoubleNet(Rent, PropertyTaxRate, InsuranceRate),
    TripleNet(
        Rent,
        PropertyTaxRate,
        InsuranceRate,
        CAMRates,
    ),
}

#[derive(Debug, Clone, Copy)]
pub struct Rent {
    pub base_rent: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct PropertyTaxRate {
    pub property_tax: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct InsuranceRate {
    pub building_insurance: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct CAMRates {
    pub electicity: f32,
    pub recycling: f32,
    pub garbage: f32,
    pub water: f32,
    pub landscaping: f32,
    pub amenities: f32,
    pub misc: f32,
}

impl Default for CAMRates {
    fn default() -> CAMRates {
        CAMRates {
            electicity: 0.4,
            recycling: 0.3,
            garbage: 0.3,
            water: 0.3,
            landscaping: 0.3,
            amenities: 0.2,
            misc: 0.0,
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Lease {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub fee_structure: FeeStructure,
    pub payment_method: String
}

impl Lease {
    pub fn new(start_date: NaiveDate, end_date: NaiveDate, fee_structure: FeeStructure, payment_method: String
    ) -> Lease {
        Lease {
            start_date,
            end_date,
            fee_structure,
            payment_method
        }
    }
}