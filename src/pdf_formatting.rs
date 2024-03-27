use std::{fmt::{Display, Formatter}, fs::File, io::BufWriter};

use printpdf::{BuiltinFont, Line, Mm, PdfDocument, Point, TextRenderingMode};

use crate::{statements::Statement, tenant::FeeStructure};

pub fn write_with_printpdf(s: Statement) {
    // Max dimension values in mm 215.9 x 279.4
    let (doc, page1, layer1) = PdfDocument::new("Monthly Statement", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let tenant = s.tenant;

    let mut y_level = Mm(270.0);
    let left_column = Mm(10.0);
    let right_column = Mm(115.0);

    let header_size = 18.0;
    let text_size = 14.0;

    current_layer.set_text_rendering_mode(TextRenderingMode::Fill);

    current_layer.begin_text_section();
        current_layer.use_text(format!("CW Holdings LLC"), header_size, left_column, y_level, &font);
        y_level -= Mm(30.0);
        current_layer.use_text(format!("{} {}", tenant.first_name, tenant.last_name), header_size, left_column, y_level, &font);
        y_level -= Mm(10.0);
        current_layer.use_text(s.date.to_string(), header_size, left_column, y_level, &font);
        y_level -= Mm(20.0);
    current_layer.end_text_section();

    let line = Line::from_iter(vec![
        (Point::new(Mm(0.0), y_level), false),
        (Point::new(Mm(350.0), y_level), false)]);
    current_layer.add_line(line);

    y_level -= Mm(10.0);
    current_layer.begin_text_section();
        //current_layer.set_line_height(10.0);
        //current_layer.set_word_spacing(10.0);
        //current_layer.set_character_spacing(3.0);

        current_layer.use_text(format!("Balance Forward: {:.2}", 0), text_size, left_column, y_level, &font);
        current_layer.use_text(format!("Payment Received {:.2}", 0), text_size, right_column, y_level, &font);
        y_level -= Mm(10.0);
        current_layer.use_text(format!("Outstanding Balance: {:.2}", 0), text_size, right_column, y_level, &font);
        y_level -= Mm(20.0);
        current_layer.use_text(format!("Rent Due : {:.2}", s.total), text_size, left_column, y_level, &font);
        for line in s.fees.display_amounts_due(1000.0) {
            current_layer.use_text(line.to_string(), text_size, right_column, y_level, &font);
            y_level -= Mm(10.0);
        }
    current_layer.end_text_section();

    y_level = Mm(15.0);
    current_layer.use_text(format!("Thank You"), text_size, right_column, y_level, &font);
    y_level += Mm(10.0);
    current_layer.use_text(format!("Please Remit To: MAILING ADDRESS"), text_size, right_column, y_level, &font);    
    y_level += Mm(10.0);
    current_layer.use_text(format!("Payment Due 1st of Coming Month"), text_size, right_column, y_level, &font);

    // Save the PDF to a file
    doc.save(&mut BufWriter::new(
        File::create("test_statement.pdf").unwrap(),
    )).unwrap();
}

fn write_with_pdfgen(s: Statement) {

}