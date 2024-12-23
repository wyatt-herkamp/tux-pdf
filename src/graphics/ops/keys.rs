use lopdf::{
    content::{Content, Operation},
    Object,
};

macro_rules! operation_keys {
    (
        $(
            $(#[$docs:meta])*
            $variant:ident => $value:literal
        ),*
    ) => {
        pub enum OperationKeys {
            $(
                $(#[$docs])*
                $variant
            ),*
        }
        impl AsRef<str> for OperationKeys {
            fn as_ref(&self) -> &str {
                match self {
                    $(
                        OperationKeys::$variant => $value,
                    )*
                }
            }
        }
    };
}
operation_keys!(
    /// Begin Text
    BeginText => "BT",
    /// Text Font
    TextFont => "Tf",
    /// Text Position
    TextPosition => "Td",
    /// Text New Line
    TextNewLine => "T*",
    /// End Text
    EndText => "ET",
    /// Show Text
    ShowText => "Tj",
    /// Save Graphics State
    SaveGraphicsState => "q",
    /// Restore Graphics State
    RestoreGraphicsState => "Q",
    /// Set Line Width
    SetLineWidth => "w",
    /// Path move to
    PathMoveTo => "m",
    /// Stroke Close
    PathPaintStrokeClose => "s",
    /// Path Paint Stroke
    PathPaintStroke => "S",
    /// Path Line To
    PathLineTo => "l",
    /// Cubic Bezier with two points in V1
    ///
    /// Look at Figure 17 in the spec
    ///
    /// But `v` makes it curve closer to the second point
    BezierCurveTwoV1 => "v",
    /// Cubic Bezier with two points in V2
    ///
    /// Look at Figure 17 in the spec
    ///
    /// But `y` makes it curve closer to the first point
    BezierCurveTwoV2 => "y",
    /// Cubic Bezier with 3 points
    ///
    /// Look at Figure 16 in the spec
    ///
    /// But `c` makes it curve in the middle and sets the new current point to the last point
    BezierCurveFour => "c",
    /// Set Fill Color Device RGB
    ColorFillDeviceRgb => "rg",
    /// Set Stroke Color Device RGB
    ColorStrokeDeviceRgb => "RG",
    PathRectangle => "re",
    PathPaintEnd => "n",
    PathPaintFillEvenOdd => "f*",
    PathPaintFillNonZero => "f",
    PathPaintFillStrokeEvenOdd => "B*",
    PathPaintFillStrokeNonZero => "B",
    PathPaintFillStrokeCloseEvenOdd => "b*",
    PathPaintFillStrokeCloseNonZero => "b",
    PathPaintClipNonZero => "W",
    PathPaintClipEvenOdd => "W*"



);

impl OperationKeys {
    pub fn lo_pdf_operation(&self, operands: Vec<Object>) -> Operation {
        Operation::new(self.as_ref(), operands)
    }
    pub fn no_operand(&self) -> Operation {
        Operation::new(self.as_ref(), vec![])
    }
}
