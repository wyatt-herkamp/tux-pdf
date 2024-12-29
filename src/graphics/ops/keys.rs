use lopdf::{content::Operation, Object};

macro_rules! operation_keys {
    (
        $(
            $(#[$docs:meta])*
            $variant:ident => $value:literal
        ),*
    ) => {
        operation_keys! {
            OperationKeys => { $(
                $(#[$docs])*
                $variant => $value
            ),*
            }
        }
    };

    (
       $key_group_name:ident => {
        $(
            $(#[$docs:meta])*
            $variant:ident => $value:literal
        ),*
    }
    ) => {
        pub enum $key_group_name {
            $(
                $(#[$docs])*
                $variant
            ),*
        }
        impl crate::graphics::ops::OperationKeyType for $key_group_name {
            fn key(&self) -> &str {
                match self {
                    $(
                        $key_group_name::$variant => $value,
                    )*
                }
            }
        }
        impl AsRef<str> for $key_group_name {
            fn as_ref(&self) -> &str {
                match self {
                    $(
                        $key_group_name::$variant => $value,
                    )*
                }
            }
        }
    };
}
pub(crate) use operation_keys;
pub trait OperationKeyType {
    fn key(&self) -> &str;
    #[inline(always)]
    fn to_operation(&self, operands: Vec<Object>) -> Operation {
        Operation::new(self.key(), operands)
    }
    #[inline(always)]
    fn no_operand(&self) -> Operation {
        Operation::new(self.key(), vec![])
    }
}
operation_keys!(
    /// Save Graphics State
    SaveGraphicsState => "q",
    /// Restore Graphics State
    RestoreGraphicsState => "Q",
    /// Set Line Width
    SetLineWidth => "w",
    /// Current Transformation Matrix
    CurrentTransformationMatrix => "cm",
    /// Render a xobject
    PaintXObject => "Do",
    BeginLayer => "BDC",
    BeginMarkedContent => "BMC",
    EndSection => "EMC"
);
