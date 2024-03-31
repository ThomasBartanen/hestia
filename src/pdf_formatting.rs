use std::{fs::File, io::BufWriter};

use printpdf::{BuiltinFont, Line, Mm, PdfDocument, Point, TextRenderingMode};

use crate::{companies::Company, properties::Property, statements::Statement};

const LEFT_COLUMN: Mm = Mm(20.0);
const RIGHT_COLUMN: Mm = Mm(115.0);
const TOP_EDGE: Mm = Mm(297.0);
const RIGHT_EDGE: Mm = Mm(210.0);

const HEADER_SIZE: f32 = 16.0;
const BODY_SIZE: f32 = 13.0;
const DETAILS_SIZE: f32 = 12.0;

pub fn write_with_printpdf(s: Statement, p: Property, c: Company) {
    // Max dimension values in mm 215.9 x 279.4
    let (doc, page1, layer1) =
        PdfDocument::new("Monthly Statement", RIGHT_EDGE, TOP_EDGE, "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let tenant = s.tenant;

    let mut y_level = Mm(270.0);
    let mut left_column = LEFT_COLUMN;
    let right_column = RIGHT_COLUMN;
    let center = RIGHT_EDGE / 2.0;

    current_layer.set_text_rendering_mode(TextRenderingMode::Fill);

    current_layer.begin_text_section();
    current_layer.use_text(c.name, HEADER_SIZE, left_column, y_level, &font);
    y_level -= Mm(10.0);
    current_layer.use_text(
        c.contact_info.email,
        HEADER_SIZE,
        left_column,
        y_level,
        &font,
    );
    y_level -= Mm(30.0);
    current_layer.use_text(
        format!(
            "{} {}",
            tenant.contact_info.first_name, tenant.contact_info.last_name
        ),
        HEADER_SIZE,
        left_column,
        y_level,
        &font,
    );
    y_level -= Mm(10.0);
    current_layer.use_text("ADDRESS TODO", HEADER_SIZE, left_column, y_level, &font);
    y_level -= Mm(10.0);
    current_layer.use_text(s.date.to_string(), HEADER_SIZE, left_column, y_level, &font);
    y_level -= Mm(20.0);
    current_layer.end_text_section();

    let line = Line::from_iter(vec![
        (Point::new(Mm(0.0), y_level), false),
        (Point::new(Mm(350.0), y_level), false),
    ]);
    current_layer.add_line(line);

    y_level -= Mm(10.0);
    current_layer.begin_text_section();
    //current_layer.set_line_height(10.0);
    //current_layer.set_word_spacing(10.0);
    //current_layer.set_character_spacing(3.0);
    current_layer.use_text(
        format!("Balance Forward: {:.2}", 0),
        BODY_SIZE,
        left_column,
        y_level,
        &font,
    );
    current_layer.use_text(
        format!("Payment Received {:.2}", 0),
        BODY_SIZE,
        right_column,
        y_level,
        &font,
    );
    y_level -= Mm(10.0);
    current_layer.use_text(
        format!("Outstanding Balance: {:.2}", 0),
        BODY_SIZE,
        right_column,
        y_level,
        &font,
    );
    y_level -= Mm(10.0);
    let table_top_level: Mm = y_level;
    y_level -= Mm(10.0);
    let mut current_iter = 0;
    left_column += Mm(15.0);
    let mut current_x: Mm = left_column;
    let mut total_due: String = String::new();
    for line in s
        .rates
        .display_amounts_due(s.fees, p.property_tax, p.business_insurance)
    {
        if total_due.is_empty() {
            total_due.push_str(&line);
            continue;
        }
        current_layer.use_text(line.to_string(), DETAILS_SIZE, current_x, y_level, &font);
        if current_iter == 1 {
            y_level -= Mm(10.0);
            current_x = left_column;
            current_iter = 0;
        } else {
            current_iter += 1;
            current_x = right_column + Mm(20.0);
        }
    }
    let table_bottom_level: Mm = y_level;
    y_level -= Mm(20.0);
    left_column = LEFT_COLUMN;
    current_layer.use_text(total_due, BODY_SIZE, right_column, y_level, &font);
    current_layer.end_text_section();

    let table_outline = Line::from_iter(vec![
        (Point::new(Mm(25.0), table_top_level), false),
        (Point::new(Mm(25.0), table_bottom_level), false),
        (Point::new(RIGHT_EDGE - Mm(25.0), table_bottom_level), false),
        (Point::new(RIGHT_EDGE - Mm(25.0), table_top_level), false),
        (Point::new(Mm(25.0), table_top_level), false),
    ]);
    current_layer.add_line(table_outline);
    let table_center_line = Line::from_iter(vec![
        (Point::new(center, table_top_level), false),
        (Point::new(center, table_bottom_level), false),
    ]);
    current_layer.add_line(table_center_line);

    y_level = Mm(15.0);
    current_layer.use_text(
        "Thank You".to_owned(),
        BODY_SIZE,
        right_column,
        y_level,
        &font,
    );
    current_layer.use_text(
        format!(
            "{}, {} {}",
            c.remittence_address.city, c.remittence_address.state, c.remittence_address.zip_code
        ),
        BODY_SIZE,
        left_column,
        y_level,
        &font,
    );
    y_level += Mm(10.0);
    current_layer.use_text(
        c.remittence_address.street_address,
        BODY_SIZE,
        left_column,
        y_level,
        &font,
    );
    current_layer.use_text(
        "Payment Due 1st of Coming Month".to_owned(),
        BODY_SIZE,
        right_column,
        y_level,
        &font,
    );
    y_level += Mm(10.0);
    current_layer.use_text("Please Remit To:", BODY_SIZE, left_column, y_level, &font);

    // Save the PDF to a file
    doc.save(&mut BufWriter::new(
        File::create("test_statement.pdf").unwrap(),
    ))
    .unwrap();
}
