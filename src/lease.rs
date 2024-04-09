use chrono::NaiveDate;

use crate::{statements::calculate_share, Expense, ExpenseType, MaintenanceType, UtilitiesType};

#[derive(Debug, Clone)]
pub enum FeeStructure {
    Gross(Rent),
    SingleNet(Rent, PropertyTaxRate),
    DoubleNet(Rent, PropertyTaxRate, InsuranceRate),
    TripleNet(Rent, PropertyTaxRate, InsuranceRate, CAMRates),
}

impl FeeStructure {
    pub fn encode_to_database_string(&self) -> String {
        match self {
            FeeStructure::Gross(rent) => {
                format!("Gross: Base Rent {}", rent.base_rent)
            }
            FeeStructure::SingleNet(rent, tax_rate) => {
                format!(
                    "Single Net: Base Rent {}, Property Tax Rate {}",
                    rent.base_rent, tax_rate.property_tax
                )
            }
            FeeStructure::DoubleNet(rent, tax_rate, insurance_rate) => {
                format!(
                    "Double Net: Base Rent {}, Property Tax Rate {}, Insurance Rate {}",
                    rent.base_rent, tax_rate.property_tax, insurance_rate.building_insurance
                )
            }
            FeeStructure::TripleNet(rent, tax_rate, insurance_rate, cam_rates) => {
                format!(
                    "Triple Net: Base Rent {}, Property Tax Rate {}, Insurance Rate {}, CAM Rates {:?}",
                    rent.base_rent, tax_rate.property_tax, insurance_rate.building_insurance, cam_rates
                )
            }
        }
    }

    pub fn display_amounts_due(
        &self,
        totals: Vec<Expense>,
        prop_tax: f32,
        bus_insurance: f32,
    ) -> Vec<String> {
        let mut lines: Vec<String> = vec![];
        let property_tax_total: f32 = prop_tax;
        let insurance_total: f32 = bus_insurance;
        let mut elect_total: f32 = 0.0;
        let mut garb_recycl_total: f32 = 0.0;
        let mut water_total: f32 = 0.0;
        let mut gas_total: f32 = 0.0;
        let mut landscaping_total: f32 = 0.0;
        let mut misc_total: f32 = 0.0;

        let mut total: f32 = 0.0;

        for expense in totals {
            match expense.expense_type {
                ExpenseType::Maintenance(maintenance_type) => match maintenance_type {
                    MaintenanceType::Repairs => misc_total += expense.amount,
                    MaintenanceType::Cleaning => misc_total += expense.amount,
                    MaintenanceType::Landscaping => landscaping_total += expense.amount,
                    MaintenanceType::Other => misc_total += expense.amount,
                },
                ExpenseType::Utilities(utilities_type) => match utilities_type {
                    UtilitiesType::Water => water_total += expense.amount,
                    UtilitiesType::Electricity => elect_total += expense.amount,
                    UtilitiesType::Garbage => garb_recycl_total += expense.amount,
                    UtilitiesType::Gas => gas_total += expense.amount,
                    UtilitiesType::Other => misc_total += expense.amount,
                },
                ExpenseType::Other => misc_total += expense.amount,
            }
        }

        match *self {
            FeeStructure::Gross(r) => lines.push(format!("Rent: ${:.2}", r.base_rent)),
            FeeStructure::SingleNet(r, t) => {
                let tax_due = calculate_share(t.property_tax, property_tax_total);
                total += r.base_rent + tax_due;
                lines.push(format!("Total Due: ${:.2}", total));
                lines.push("Rent:".to_owned());
                lines.push(format!("${:.2}", r.base_rent));
                lines.push(format!("Property Tax ({:.1}%):", t.property_tax * 100.0));
                lines.push(format!("${:.2}", tax_due));
            }
            FeeStructure::DoubleNet(r, t, i) => {
                let tax_due = calculate_share(t.property_tax, property_tax_total);
                let insurance_due = calculate_share(i.building_insurance, insurance_total);
                total += r.base_rent + tax_due + insurance_due;
                lines.push(format!("Total Due: ${:.2}", total));
                lines.push("Rent:".to_owned());
                lines.push(format!("${:.2}", r.base_rent));
                lines.push(format!("Property Tax ({:.1}%):", t.property_tax * 100.0));
                lines.push(format!("${:.2}", tax_due));
                lines.push(format!("Insurance ({:.1}%):", i.building_insurance * 100.0));
                lines.push(format!("${:.2}", insurance_due));
            }
            FeeStructure::TripleNet(r, t, i, c) => {
                let tax_due = calculate_share(t.property_tax, property_tax_total);
                let insurance_due = calculate_share(i.building_insurance, insurance_total);
                let elec_due = calculate_share(c.electicity, elect_total);
                let garb_recycl_due = calculate_share(c.garbage + c.recycling, garb_recycl_total);
                let water_sewer_due = calculate_share(c.water, water_total);
                let landscaping_due = calculate_share(c.landscaping, landscaping_total);
                let misc_due = calculate_share(c.misc, misc_total);
                total += r.base_rent
                    + tax_due
                    + insurance_due
                    + elec_due
                    + gas_total
                    + garb_recycl_due
                    + water_sewer_due
                    + landscaping_due
                    + misc_due;
                lines.push(format!("Total Due: ${:.2}", total));
                lines.push("Rent:".to_owned());
                lines.push(format!("${:.2}", r.base_rent));
                lines.push(format!("Property Tax ({:.1}%):", t.property_tax * 100.0));
                lines.push(format!("${:.2}", tax_due));
                lines.push(format!("Insurance ({:.1}%):", i.building_insurance * 100.0));
                lines.push(format!("${:.2}", insurance_due));
                lines.push(format!("Electricity ({:.1}%):", c.electicity * 100.0));
                lines.push(format!("${:.2}", elec_due));
                lines.push(format!(
                    "Garbage/Recycling ({:.1}% / {:.1}%):",
                    c.garbage * 100.0,
                    c.recycling * 100.0
                ));
                lines.push(format!("${:.2}", garb_recycl_due));
                lines.push(format!("Water/Sewer ({:.1}%):", c.water * 100.0));
                lines.push(format!("${:.2}", water_sewer_due));
                lines.push(format!("Landscaping ({:.1}%):", c.landscaping * 100.0));
                lines.push(format!("${:.2}", landscaping_due));
                lines.push(format!("Miscellaneous ({:.1}%):", c.misc * 100.0));
                lines.push(format!("${:.2}", misc_due));
            }
        };
        lines
    }
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
            misc: 0.2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lease {
    pub id: u16,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub fee_structure: FeeStructure,
    pub payment_method: String,
}

impl Lease {
    pub fn new(
        start_date: NaiveDate,
        end_date: NaiveDate,
        fee_structure: FeeStructure,
        payment_method: String,
    ) -> Lease {
        Lease {
            id: 0,
            start_date,
            end_date,
            fee_structure,
            payment_method,
        }
    }
}
