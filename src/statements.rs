use chrono::NaiveDate;
use printpdf::*;
use std::{fs::File, io::BufWriter};

use crate::tenant::{FeeStructure, Tenant};

#[derive(Debug)]
pub struct Statement {
    date: NaiveDate,
    tenant: Tenant,
    fees: FeeStructure,
    total: f32
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
    let total = match fee_structure {
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
    // Max dimension values in mm 215.9 x 279.4
    let (doc, page1, layer1) = PdfDocument::new("Monthly Statement", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let tenant = statement.tenant;

    current_layer.set_text_rendering_mode(TextRenderingMode::Fill);

    current_layer.begin_text_section();
        current_layer.set_line_height(2.0);
        current_layer.set_word_spacing(2.0);
        current_layer.set_character_spacing(3.0);
        current_layer.set_font(&font, 20.0);
        current_layer.set_text_cursor(Mm(10.0), Mm(230.0));
        current_layer.write_text(format!("{} {}", tenant.first_name, tenant.last_name), &font);
        current_layer.set_text_cursor(Mm(10.0), Mm(200.0));
        current_layer.write_text(statement.date.to_string(), &font);
        current_layer.add_line_break();
    current_layer.end_text_section();

    current_layer.set_font(&font, 10.0);

    let line = Line::from_iter(vec![
        (Point::new(Mm(0.0), Mm(250.0)), false),
        (Point::new(Mm(350.0), Mm(250.0)), false)]);
    
    current_layer.add_line(line);

    current_layer.begin_text_section();
        current_layer.set_line_height(10.0);
        current_layer.set_word_spacing(10.0);
        current_layer.set_character_spacing(3.0);

        current_layer.write_text(format!("Balance Forward: {:.2}, Payment Received {:.2}", 0, 0), &font);
        current_layer.write_text(format!("Outstanding Balance: {:.2}", 0), &font);

    current_layer.end_text_section();
    // Save the PDF to a file
    doc.save(&mut BufWriter::new(
        File::create("test_statement.pdf").unwrap(),
    )).unwrap();
}