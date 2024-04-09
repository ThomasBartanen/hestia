use crate::{
    app_settings::PathSettings,
    companies::{Company, Leaseholder},
    lease::FeeStructure,
    pdf_formatting::write_with_printpdf,
    properties::Property,
    Expense,
};
use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct Statement {
    pub date: NaiveDate,
    pub leaseholder: Leaseholder,
    pub rates: FeeStructure,
    pub fees: Vec<Expense>,
    pub total: f32,
}

impl Statement {
    pub fn new(date: NaiveDate, tenant: Leaseholder, fees: Vec<Expense>) -> Statement {
        let tenant_clone = tenant.clone();
        Statement {
            date,
            leaseholder: tenant,
            rates: tenant_clone.clone().lease.fee_structure,
            fees,
            total: calculate_total(tenant_clone.lease.fee_structure, 1000.0),
        }
    }
}

pub fn calculate_total(fee_structure: FeeStructure, building_fees: f32) -> f32 {
    match fee_structure {
        FeeStructure::Gross(rent) => rent.base_rent,
        FeeStructure::SingleNet(rent, tax) => {
            rent.base_rent + calculate_share(tax.property_tax, building_fees)
        }
        FeeStructure::DoubleNet(rent, tax, insurance) => {
            rent.base_rent
                + calculate_share(tax.property_tax, building_fees)
                + calculate_share(insurance.building_insurance, building_fees)
        }
        FeeStructure::TripleNet(rent, tax, insurance, cam) => {
            rent.base_rent
                + calculate_share(tax.property_tax, building_fees)
                + calculate_share(insurance.building_insurance, building_fees)
                + calculate_share(cam.amenities, building_fees)
                + calculate_share(cam.electicity, building_fees)
                + calculate_share(cam.garbage, building_fees)
                + calculate_share(cam.landscaping, building_fees)
                + calculate_share(cam.misc, building_fees)
                + calculate_share(cam.recycling, building_fees)
                + calculate_share(cam.water, building_fees)
        }
    }
}

pub fn calculate_share(rate: f32, total: f32) -> f32 {
    total * rate
}

pub fn create_statement(
    statement: Statement,
    property: Property,
    company: Company,
    settings: PathSettings,
) {
    write_with_printpdf(statement, property, company, settings);
}
