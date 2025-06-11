use derive_more::From;

use crate::units::Pt;

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub enum ColumnMinWidth {
    /// The column width is fixed and will not be recalculated
    Fixed(Pt),
    /// The column width is a percentage of the total width
    Percentage(f32),
    /// The column width set to the whatever is left after the other columns have been calculated
    /// Good for a notes column
    ///
    /// You are limited to one column with this setting
    AutoFill,
}

#[derive(Debug, Clone, Copy, PartialEq, From)]
pub enum ColumnMaxWidth {
    /// The column width is fixed and will not be recalculated
    Fixed(Pt),
    /// The column width is a percentage of the available width
    Percentage(f32),
}
