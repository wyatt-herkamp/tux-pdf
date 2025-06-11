use super::LayoutError;
use taffy::{Dimension, NodeId, PrintTree, TaffyTree};
mod style_builders;
use crate::{
    TuxPdfError,
    document::PdfDocument,
    graphics::{
        GraphicItems, GraphicStyles, GraphicsGroup, HasPosition, LayerType, PdfPosition,
        color::BLACK_RGB, shapes::OutlineRect, size::Size,
    },
    page::PdfPage,
    units::{Pt, UnitType},
};
pub use style_builders::*;

use tracing::{debug, info};
type TaffyStyle = taffy::Style;
use super::{LayoutItem, LayoutItemType};
#[derive(Debug, Clone, PartialEq)]
pub struct PdfTaffyItem {
    item: LayoutItem,
    node_id: NodeId,
}
#[derive(Debug, Clone)]
pub struct PdfTaffyLayout {
    items: Vec<PdfTaffyItem>,
    styling: TaffyStyle,
    taffy_tree: TaffyTree<()>,
    position: Option<PdfPosition>,
    page_size: Size,
    max_size: Size,
    root_node: Option<NodeId>,
}
impl LayoutItemType for PdfTaffyLayout {
    fn calculate_size(&mut self, document: &PdfDocument) -> Result<Size, TuxPdfError> {
        self.calculate_sizes(document)?;
        let root = self.compute_layout()?;

        Ok(self.taffy_tree.get_final_layout(root).size.into())
    }

    fn render<L: LayerType>(
        mut self,
        document: &PdfDocument,
        page: &mut L,
    ) -> Result<(), TuxPdfError>
    where
        Self: Sized,
    {
        self.calculate_sizes(document)?;
        self.compute_layout()?;
        let Self {
            taffy_tree, items, ..
        } = self;

        for item in items {
            let node = taffy_tree.get_final_layout(item.node_id);

            let content_y: Pt = node.content_box_y().into();
            let content_x: Pt = node.content_box_x().into();
            debug!(?content_x, ?content_y, "Content Box Position");
            let content_y = content_y + node.content_box_height();

            let position: PdfPosition = if let Some(position) = self.position {
                PdfPosition {
                    x: position.x + content_x,
                    y: position.y - content_y,
                }
            } else {
                PdfPosition {
                    x: content_x,
                    y: content_y,
                }
            };
            let mut item: LayoutItem = item.item;
            item.set_position(position);
            debug!(?item, ?position, "Rendering Item");

            item.render(document, page)?;
        }
        Ok(())
    }
}
impl HasPosition for PdfTaffyLayout {
    fn position(&self) -> PdfPosition {
        self.position.unwrap_or_default()
    }

    fn set_position(&mut self, position: PdfPosition) {
        self.position = Some(position);
    }
}
impl PartialEq for PdfTaffyLayout {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
            && self.styling == other.styling
            && self.position == other.position
            && self.page_size == other.page_size
            && self.max_size == other.max_size
    }
}
impl PdfTaffyLayout {
    pub fn new(styling: TaffyStyle, max_size: Size, page_size: Size) -> Self {
        Self {
            items: Vec::new(),
            styling,
            taffy_tree: TaffyTree::new(),
            page_size,
            max_size,
            root_node: None,
            position: None,
        }
    }
    pub fn update_styling(&mut self, styling: TaffyStyle) {
        self.styling = styling;
        if let Some(root) = self.root_node {
            self.taffy_tree.remove(root).unwrap();
        }
    }
    pub fn add_item(&mut self, item: impl Into<LayoutItem>, taffy_styles: taffy::Style) {
        let item = item.into();
        let node_id = self.taffy_tree.new_leaf(taffy_styles).unwrap();
        self.items.push(PdfTaffyItem { item, node_id });
    }
    fn calculate_sizes(&mut self, document: &PdfDocument) -> Result<(), TuxPdfError> {
        for item in &mut self.items {
            let size = item.item.calculate_size(document)?;
            let taffy_size: taffy::Size<Dimension> = size.into();
            let node = self.taffy_tree.style(item.node_id).unwrap();
            if node.size == taffy_size {
                continue;
            }
            let mut node_styling = node.clone();
            node_styling.size = size.into();
            self.taffy_tree
                .set_style(item.node_id, node_styling)
                .map_err(LayoutError::from)?;
        }
        Ok(())
    }
    fn compute_layout(&mut self) -> Result<NodeId, LayoutError> {
        if let Some(root) = self.root_node {
            if !self.taffy_tree.dirty(root)? {
                return Ok(root);
            }
            self.taffy_tree.remove(root)?;
        }
        let root = self.taffy_tree.new_with_children(
            self.styling.clone(),
            &self
                .items
                .iter()
                .map(|item| item.node_id)
                .collect::<Vec<_>>(),
        )?;
        self.taffy_tree.compute_layout(root, self.max_size.into())?;
        #[cfg(test)]
        self.taffy_tree.print_tree(root);
        self.root_node = Some(root);

        Ok(root)
    }
    pub fn draw_grid(
        &mut self,
        document: &PdfDocument,
        page: &mut PdfPage,
    ) -> Result<(), TuxPdfError> {
        self.calculate_sizes(document)?;
        let root = self.compute_layout()?;

        let Self {
            taffy_tree, items, ..
        } = self;
        let mut graphic_items: Vec<GraphicItems> = Vec::with_capacity(items.len() + 1);
        for item in items {
            let node = taffy_tree.get_final_layout(item.node_id);
            let content_y: Pt = node.location.y.into();
            let content_x: Pt = node.location.x.into();
            let size: Size<Pt> = Size {
                width: node.size.width.into(),
                height: node.size.height.into(),
            };
            let position: PdfPosition = if let Some(position) = self.position {
                PdfPosition {
                    x: position.x + content_x,
                    y: position.y - content_y,
                }
            } else {
                PdfPosition {
                    x: content_x,
                    y: content_y,
                }
            };
            let outline = OutlineRect { position, size };

            graphic_items.push(outline.into());
        }
        let node = taffy_tree.get_final_layout(root);
        let content_y: Pt = node.content_box_y().into();
        let content_x: Pt = node.content_box_x().into();
        let size: Size<Pt> = Size {
            width: node.size.width.into(),
            height: node.size.height.into(),
        };
        let position: PdfPosition = PdfPosition {
            x: content_x,
            y: (content_y),
        };
        info!(?position, ?size, "Drawing Grid Outline");

        let grid_outline = OutlineRect::new_from_bottom_left(position, size);
        graphic_items.push(grid_outline.into());
        let graphics_items = GraphicsGroup {
            items: graphic_items,
            styles: Some(GraphicStyles {
                fill_color: Some(BLACK_RGB),
                line_width: Some(1f32.pt()),
                ..Default::default()
            }),
            section_name: Some("LayoutOutline".to_string()),
        };

        page.add_to_layer(graphics_items)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use std::fs::File;

    use taffy::prelude::*;

    use crate::{
        document::owned_ttf_parser::OwnedPdfTtfFont,
        graphics::{HasPosition, TextBlock, TextStyle},
        layouts::LayoutItemType,
        page::{PdfPage, page_sizes::A4},
        tests::{create_test_document, fonts_dir, save_pdf_doc},
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
            A4,
            A4,
        )
        .with_position(A4.top_left_point());
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
        grid.draw_grid(&doc, &mut page)?;
        grid.render(&doc, &mut page)?;

        doc.add_page(page);

        save_pdf_doc(doc, "grid_test")?;
        Ok(())
    }

    #[test]
    fn test_footer_flex_box() -> anyhow::Result<()> {
        crate::tests::init_logger();
        let mut doc = create_test_document("test_footer_flex_box");
        let roboto_loaded = File::open(fonts_dir().join("Roboto").join("Roboto-Regular.ttf"))?;
        let roboto = OwnedPdfTtfFont::new_from_reader(roboto_loaded, 0)?;
        let roboto = doc.font_map().register_external_font(roboto)?;
        let mut grid = PdfTaffyLayout::new(
            TaffyStyle {
                display: taffy::Display::Flex,
                justify_content: Some(taffy::AlignContent::SpaceBetween),
                size: A4.into(),

                padding: taffy::Rect {
                    top: taffy::LengthPercentage::length(0f32),
                    bottom: taffy::LengthPercentage::length(10f32),
                    left: taffy::LengthPercentage::length(5f32),
                    right: taffy::LengthPercentage::length(5f32),
                },
                ..Default::default()
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
                align_content: Some(taffy::AlignContent::SpaceBetween),
                padding: taffy::Rect {
                    top: taffy::LengthPercentage::length(0f32),
                    bottom: taffy::LengthPercentage::length(10f32),
                    left: taffy::LengthPercentage::length(0f32),
                    right: taffy::LengthPercentage::length(0f32),
                },
                ..Default::default()
            },
        );

        grid.add_item(
            TextBlock::from("My FlexBox Layout").with_style(TextStyle {
                font_ref: roboto.clone(),
                font_size: 12.0.into(),
                ..Default::default()
            }),
            TaffyStyle {
                align_content: Some(taffy::AlignContent::SpaceBetween),
                padding: taffy::Rect {
                    top: taffy::LengthPercentage::length(0f32),
                    bottom: taffy::LengthPercentage::length(10f32),
                    left: taffy::LengthPercentage::length(0f32),
                    right: taffy::LengthPercentage::length(0f32),
                },
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
                padding: taffy::Rect {
                    top: length(0f32),
                    bottom: taffy::LengthPercentage::length(10f32),
                    left: taffy::LengthPercentage::length(0f32),
                    right: taffy::LengthPercentage::length(0f32),
                },
                ..Default::default()
            },
        );

        let mut page = PdfPage::new_from_page_size(A4);
        grid.draw_grid(&doc, &mut page)?;
        grid.render(&doc, &mut page)?;

        doc.add_page(page);

        save_pdf_doc(doc, "test_footer_flex_box")?;
        Ok(())
    }
}
