use std::{fs::File, path::PathBuf};

use clap::Parser;
use tux_pdf::{
    document::{static_ttf_parser::StaticTtfFace, PdfDocument},
    graphics::{
        color::BLACK_RGB,
        layouts::grid::GridStyleGroup,
        styles::Margin,
        table::{Column, Row, Table, TablePageRules, TableStyles, TableValueWithStyle},
        TextStyle,
    },
    page::{page_sizes::A4, PdfPage},
    units::UnitType,
    TuxPdfError,
};
static ROBOTO_FONT: &[u8] = include_bytes!("../../tests/fonts/Roboto/Roboto-Regular.ttf");
#[derive(Debug, Clone, Parser)]
struct CsvToPdf {
    cvs_file: PathBuf,
    #[clap(short, long)]
    output_file: Option<PathBuf>,
}
fn table_page(_: &mut PdfDocument) -> Result<(TablePageRules, PdfPage), TuxPdfError> {
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
}
fn main() -> anyhow::Result<()> {
    let args = CsvToPdf::parse();
    if !args.cvs_file.exists() {
        eprintln!("The file {:?} does not exist", args.cvs_file);
        std::process::exit(1);
    }
    let output_file = if let Some(output_file) = args.output_file {
        output_file
    } else {
        PathBuf::from("table.pdf")
    };
    let (columns, rows) = read_csv(&args.cvs_file)?;

    let mut doc = PdfDocument::new(format!(
        "Table from {}",
        args.cvs_file.file_name().unwrap().to_string_lossy()
    ));
    let roboto_font = StaticTtfFace::from_slice(ROBOTO_FONT, 0)?;
    let roboto = doc.font_map().register_external_font(roboto_font)?;

    let table = Table {
        columns,
        rows,
        styles: TableStyles {
            text_styles: TextStyle {
                font_ref: roboto,
                font_size: 15f32.pt(),
                ..Default::default()
            },
            outer_styles: Some(GridStyleGroup {
                background_color: None,
                border_color: Some(BLACK_RGB),
                border_width: Some(1f32.pt()),
            }),
            ..Default::default()
        },
        new_page: table_page,
    };

    let first_page = table_page(&mut doc)?;
    table.render(&mut doc, first_page)?;
    let mut pdf = doc.write_to_lopdf_document()?;
    let mut file = std::fs::File::create(output_file)?;
    pdf.save_to(&mut file)?;
    Ok(())
}

fn read_csv(file: &PathBuf) -> anyhow::Result<(Vec<Column>, Vec<Row>)> {
    let cvs_file = File::open(file)?;

    let mut cvs_reader = csv::Reader::from_reader(cvs_file);
    if !cvs_reader.has_headers() {
        eprintln!("The CSV file must have headers");
        std::process::exit(1);
    }

    let headers = cvs_reader.headers()?.clone();
    let header_columns = headers
        .into_iter()
        .map(|header| Column {
            header: header.to_owned(),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    let mut rows = Vec::new();
    for (index, row) in cvs_reader.records().enumerate() {
        let row = row?;
        let values: Vec<_> = row
            .into_iter()
            .map(|value| TableValueWithStyle {
                value: value.into(),
                style: None,
            })
            .collect();
        if values.len() != header_columns.len() {
            eprintln!(
                "Row {} has {} columns but the header has {} columns",
                index,
                values.len(),
                header_columns.len()
            );
            std::process::exit(1);
        }
        let row = Row {
            values,
            ..Default::default()
        };
        rows.push(row);
    }

    Ok((header_columns, rows))
}
