use super::LayoutError;
pub use taffy;
use taffy::{NodeId, PrintTree, TaffyTree};

use crate::{
    document::PdfDocument,
    graphics::{
        color::BLACK_RGB, shapes::OutlineRect, size::Size, GraphicStyles, GraphicsGroup,
        HasPosition as _, Point,
    },
    page::PdfPage,
    units::{Pt, UnitType},
    TuxPdfError,
};

use tracing::{debug, info};
pub type TaffyStyle = taffy::Style;
use super::{LayoutItem, LayoutItemType};

pub struct PdfTaffyItem {
    item: LayoutItem,
    node_id: NodeId,
}
pub struct PdfTaffyLayout {
    items: Vec<PdfTaffyItem>,
    styling: TaffyStyle,
    taffy_tree: TaffyTree<()>,
    position: Point,
    page_size: Size,
    max_size: Size,
}
impl PdfTaffyLayout {
    pub fn new(styling: TaffyStyle, position: Point, max_size: Size, page_size: Size) -> Self {
        Self {
            items: Vec::new(),
            styling,
            taffy_tree: TaffyTree::new(),
            position,
            page_size,
            max_size,
        }
    }
    pub fn add_item(&mut self, item: impl Into<LayoutItem>, taffy_styles: taffy::Style) {
        let item = item.into();
        let node_id = self.taffy_tree.new_leaf(taffy_styles).unwrap();
        self.items.push(PdfTaffyItem { item, node_id });
    }
    fn calculate_sizes(&mut self, document: &mut PdfDocument) -> Result<(), TuxPdfError> {
        for item in &mut self.items {
            let size = item.item.calculate_size(document)?;
            let mut node = self.taffy_tree.style(item.node_id).unwrap().clone();
            node.size = size.into();
            self.taffy_tree
                .set_style(item.node_id, node)
                .map_err(LayoutError::from)?;
        }
        Ok(())
    }
    fn compute_layout(&mut self) -> Result<NodeId, LayoutError> {
        let root = self.taffy_tree.new_with_children(
            self.styling.clone(),
            &self
                .items
                .iter()
                .map(|item| item.node_id)
                .collect::<Vec<_>>(),
        )?;
        println!("{:?}", self.max_size);
        self.taffy_tree.compute_layout(root, self.max_size.into())?;
        self.taffy_tree.print_tree(root);
        Ok(root)
    }
    pub fn draw_grid(
        &mut self,
        document: &mut PdfDocument,
        page: &mut PdfPage,
    ) -> Result<(), TuxPdfError> {
        self.calculate_sizes(document)?;
        let root = self.compute_layout()?;

        let Self {
            taffy_tree, items, ..
        } = self;

        for item in items {
            let node = taffy_tree.get_final_layout(item.node_id);
            let content_y: Pt = node.location.y.into();
            let content_x: Pt = node.location.x.into();
            let size: Size<Pt> = Size {
                width: node.size.width.into(),
                height: node.size.height.into(),
            };
            let position: Point = Point {
                x: self.position.x + content_x,
                y: self.position.y - (content_y),
            };

            let outline = OutlineRect { position, size };

            let graphics_items = GraphicsGroup {
                items: vec![outline.into()],
                styles: Some(GraphicStyles {
                    fill_color: Some(BLACK_RGB),
                    line_width: Some(1f32.pt()),
                    ..Default::default()
                }),
            };
            page.add_operation(graphics_items.into());
        }
        let node = taffy_tree.get_final_layout(root);
        let content_y: Pt = node.content_box_y().into();
        let content_x: Pt = node.content_box_x().into();
        let size: Size<Pt> = Size {
            width: node.size.width.into(),
            height: node.size.height.into(),
        };
        let position: Point = Point {
            x: content_x + self.position.x,
            y: self.position.y - (content_y),
        };
        info!(?position, ?size, "Drawing Grid Outline");

        let grid_outline = OutlineRect::new_from_bottom_left(position, size);

        let graphics_items = GraphicsGroup {
            items: vec![grid_outline.into()],
            styles: Some(GraphicStyles {
                fill_color: Some(BLACK_RGB),
                line_width: Some(1f32.pt()),
                ..Default::default()
            }),
        };

        page.add_operation(graphics_items.into());
        Ok(())
    }
    pub fn render(
        mut self,
        document: &mut PdfDocument,
        page: &mut PdfPage,
    ) -> Result<(), TuxPdfError> {
        self.calculate_sizes(document)?;
        self.compute_layout()?;
        let Self {
            taffy_tree, items, ..
        } = self;

        for item in items {
            let node = taffy_tree.get_final_layout(item.node_id);
            let content_y: Pt = node.content_box_y().into();
            let content_y = content_y + node.content_box_height();

            let content_x: Pt = node.content_box_x().into();
            let position: Point = Point {
                x: self.position.x + content_x,
                y: self.position.y - (content_y),
            };
            let mut item: LayoutItem = item.item;
            item.set_position(position);
            debug!(?item, ?position, "Rendering Item");

            item.render(document, page)?;
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use std::fs::File;

    use taffy::prelude::{fr, line, span};

    use crate::{
        document::owned_ttf_parser::OwnedPdfTtfFont,
        graphics::{Point, TextBlock, TextStyle},
        page::{page_sizes::A4, PdfPage},
        tests::{create_test_document, fonts_dir, save_pdf_doc},
        units::UnitType,
    };

    use super::{PdfTaffyLayout, TaffyStyle};

    #[test]
    fn test_grid() -> anyhow::Result<()> {
        crate::tests::init_logger();
        let mut doc = create_test_document("grid_test");
        let roboto_loaded = File::open(fonts_dir().join("Roboto").join("Roboto-Regular.ttf"))?;
        let roboto = OwnedPdfTtfFont::new_from_reader(roboto_loaded, 0)?;
        let roboto = doc.font_map().register_external_font(roboto)?;
        let mut grid = PdfTaffyLayout::new(
            TaffyStyle {
                display: taffy::Display::Grid,
                grid_template_columns: vec![fr(50.0), fr(50.0)],
                grid_template_rows: vec![fr(50.0), fr(50.0)],
                align_content: Some(taffy::AlignContent::Center),
                justify_content: Some(taffy::JustifyContent::Center),
                ..Default::default()
            },
            A4.top_left_point()
                + Point {
                    x: 10f32.pt(),
                    y: -10f32.pt(),
                },
            A4,
            A4,
        );
        grid.add_item(
            TextBlock::from("Hello World").with_style(TextStyle {
                font_ref: roboto.clone(),
                font_size: 48.0.into(),
                ..Default::default()
            }),
            TaffyStyle {
                grid_column: span(2),
                grid_row: line(1),
                ..Default::default()
            },
        );

        grid.add_item(
            TextBlock::from("My Grid Layout").with_style(TextStyle {
                font_ref: roboto.clone(),
                font_size: 12.0.into(),
                ..Default::default()
            }),
            TaffyStyle {
                grid_column: span(1),
                grid_row: line(2),
                align_self: Some(taffy::AlignSelf::Center),
                ..Default::default()
            },
        );

        grid.add_item(
            TextBlock::from("This is a test").with_style(TextStyle {
                font_ref: roboto.clone(),
                font_size: 12.0.into(),
                ..Default::default()
            }),
            TaffyStyle {
                grid_column: span(1),
                grid_row: line(2),
                align_self: Some(taffy::AlignSelf::End),
                ..Default::default()
            },
        );

        let mut page = PdfPage::new_from_page_size(A4);
        grid.draw_grid(&mut doc, &mut page)?;
        grid.render(&mut doc, &mut page)?;

        doc.add_page(page);

        save_pdf_doc(doc, "grid_test")?;
        Ok(())
    }
}
