use chrono::NaiveDate;
use crate::{tenant::{FeeStructure, Tenant}, pdf_formatting::write_with_printpdf};

#[derive(Debug)]
pub struct Statement {
    pub date: NaiveDate,
    pub tenant: Tenant,
    pub fees: FeeStructure,
    pub total: f32
}

impl Statement {
    pub fn new(date: NaiveDate, tenant: Tenant, fees: FeeStructure) -> Statement {
        Statement {
            date,
            tenant,
            fees: fees.clone(),
            total: calculate_total(fees, 1000.0)
        }
    }
}

pub fn calculate_total(fee_structure: FeeStructure, building_fees: f32) -> f32 {
    match fee_structure {
        FeeStructure::Gross(rent) => return rent.base_rent,
        FeeStructure::SingleNet(rent, tax) => return {
            rent.base_rent + 
            calculate_share(tax.property_tax, building_fees)
        },
        FeeStructure::DoubleNet(rent, tax, insurance) => return{
            rent.base_rent + 
            calculate_share(tax.property_tax, building_fees) + 
            calculate_share(insurance.building_insurance, building_fees)
        },
        FeeStructure::TripleNet(rent, tax, insurance, cam) => return {
            rent.base_rent + 
            calculate_share(tax.property_tax, building_fees) + 
            calculate_share(insurance.building_insurance, building_fees) + 
            calculate_share(cam.amenities, building_fees) + 
            calculate_share(cam.electicity, building_fees) + 
            calculate_share(cam.garbage, building_fees) + 
            calculate_share(cam.landscaping, building_fees) + 
            calculate_share(cam.misc, building_fees) + 
            calculate_share(cam.recycling, building_fees) +
            calculate_share(cam.water, building_fees)
        },
    };
}

pub fn calculate_share(rate: f32, total: f32) -> f32 {
    total * rate
}

pub fn create_statement(statement: Statement) {
    write_with_printpdf(statement);
}