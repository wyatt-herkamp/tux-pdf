/*!
 * # Layouts
 *
 * This module contains the layout system for this PDF library.
 * Layouts will allow you to build pdf files with ease and without as much fear of the layout breaking.
 * or a ton of boilerplate code.
 *
 * ## Available Layouts
 * - [Table Layout](table::Table)
 * - [Taffy Layout](taffy_layout::PdfTaffyLayout) (Requires the `taffy` feature) will allow you to create flex boxes and grid layouts
 *
*/
use thiserror::Error;

mod layout_type;
pub use layout_type::*;

pub mod table;
#[cfg(feature = "taffy")]
pub mod taffy_layout;
/// Re-export the taffy crate
///
/// If you are going to use the taffy_layout module. I highly recommend you import it's prelude
/// using `use tux_pdf::grahpics::layouts::taffy_crate::prelude::*;`
///
/// We rewrite their prelude a bit to prevent name conflicts with our own types
#[cfg(feature = "export-taffy")]
pub mod taffy_crate {
    #[doc(hidden)]
    pub use taffy;

    pub mod prelude {
        pub use taffy::{
            geometry::{Line as TaffyLine, Rect as TaffyRect, Size as TaffySize},
            style::{
                AlignContent, AlignItems, AlignSelf, AvailableSpace, BoxSizing, Dimension, Display,
                JustifyContent, JustifyItems, JustifySelf, LengthPercentage, LengthPercentageAuto,
                Position as TaffyPosition, Style as TaffyStyle,
            },
            style_helpers::{
                FromLength, FromPercent, TaffyAuto, TaffyFitContent, TaffyMaxContent,
                TaffyMinContent, TaffyZero, auto, fit_content, length, max_content, min_content,
                percent, zero,
            },
            tree::{
                Layout as TaffyLayout, LayoutPartialTree, NodeId, PrintTree, RoundTree,
                TraversePartialTree, TraverseTree,
            },
        };

        pub use taffy::style::{FlexDirection, FlexWrap};

        pub use taffy::style::{
            GridAutoFlow, GridPlacement, GridTrackRepetition, MaxTrackSizingFunction,
            MinTrackSizingFunction, NonRepeatedTrackSizingFunction, TrackSizingFunction,
        };
        pub use taffy::style_helpers::{
            TaffyGridLine, TaffyGridSpan, evenly_sized_tracks, flex, fr, line, minmax, repeat, span,
        };

        pub use taffy::TaffyTree;
    }
}
/// Layout related errors
#[derive(Debug, PartialEq, Error)]
pub enum LayoutError {
    /// An item could not be resized
    #[error("Unable to resize the item")]
    UnableToResize,
    /// An error occurred within taffy
    #[cfg(feature = "taffy")]
    #[error(transparent)]
    TaffyError(#[from] taffy::TaffyError),
    /// An error occurred within the table layout
    #[error(transparent)]
    TableError(#[from] table::TableError),
}
