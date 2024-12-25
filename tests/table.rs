mod test_utils;
use test_utils::{destination_dir, fonts_dir};
use tux_pdf::{
    document::{owned_ttf_parser::OwnedPdfTtfFont, PdfDocument},
    graphics::{
        color::{BLACK_RGB, GRAY_RGB, WHITE_RGB},
        layouts::grid::{GridColumnMinWidth, GridStyleGroup},
        styles::{Margin, Padding},
        table::{Column, Row, RowStyles, Table, TablePageRules, TableStyles, TableValue},
        TextStyle,
    },
    page::{page_sizes::A4, PdfPage},
    units::UnitType,
};

#[test]
fn table_test() -> anyhow::Result<()> {
    test_utils::init_logger();
    let columns: Vec<_> = vec![
        Column::from("Location"),
        Column::from("Order Number"),
        Column::from("Customer Name"),
        Column::from("Order Type"),
        Column::from("Order Size"),
        Column::from("Work Order"),
        Column::from("Notes").with_min_width(GridColumnMinWidth::AutoFill),
    ];
    let even_row_style = RowStyles {
        background_color: Some(GRAY_RGB),
        ..Default::default()
    };
    let odd_row_style = RowStyles {
        background_color: Some(WHITE_RGB),
        ..Default::default()
    };
    let mut actual_rows = Vec::new();
    for number in 0..50 {
        let location = format!("Location {}", number);
        let order_number = format!("12345{}", number);
        let customer_name = format!("Customer {}", number);
        let order_type = format!("Type {}", number);
        let order_size = format!("Size {}", number);
        let work_order = format!("Work Order {}", number);
        let mut row = Row::from(vec![
            location.into(),
            order_number.into(),
            customer_name.into(),
            order_type.into(),
            order_size.into(),
            work_order.into(),
            TableValue::BlankSpace,
        ]);
        if number % 2 == 0 {
            row = row.with_styles(even_row_style.clone());
        } else {
            row = row.with_styles(odd_row_style.clone());
        }
        actual_rows.push(row);
    }
    let mut doc = PdfDocument::new("Table Test");
    let roboto_font_reader =
        std::fs::File::open(fonts_dir().join("Roboto").join("Roboto-Regular.ttf"))?;
    let roboto_font = OwnedPdfTtfFont::new_from_reader(roboto_font_reader, 0)?;

    let roboto = doc.font_map().register_external_font(roboto_font)?;

    let table = Table {
        columns,
        rows: actual_rows,
        styles: TableStyles {
            text_styles: TextStyle {
                font_ref: roboto,
                font_size: 15.0.pt(),
                ..Default::default()
            },
            cell_content_padding: Padding::all(5f32.pt()),
            outer_styles: Some(GridStyleGroup {
                background_color: None,
                border_color: Some(BLACK_RGB),
                border_width: Some(1f32.pt()),
            }),
            ..Default::default()
        },
        new_page: |_| {
            let page = PdfPage::new_from_page_size(A4.landscape());
            let table_start = A4.landscape().height - 10f32.pt();
            let table_end = 10f32.pt();
            let page_rules = TablePageRules {
                page_size: A4.landscape(),
                table_start_y: Some(table_start),
                table_stop_y: Some(table_end),
                margin: Some(Margin::left_and_right(10f32.pt(), 10f32.pt())),
            };
            Ok((page_rules, page))
        },
    };

    let table_start = A4.landscape().height - 10f32.pt();

    let table_end = 10f32.pt();

    let page = PdfPage::new_from_page_size(A4.landscape());
    let page_rules = TablePageRules {
        page_size: A4.landscape(),
        table_start_y: Some(table_start),
        table_stop_y: Some(table_end),
        margin: Some(Margin::left_and_right(10f32.pt(), 10f32.pt())),
    };
    table.render(&mut doc, (page_rules, page))?;
    let mut pdf = doc.write_to_lopdf_document()?;
    let mut file = std::fs::File::create(destination_dir().join("table.pdf"))?;
    pdf.save_to(&mut file)?;

    Ok(())
}
