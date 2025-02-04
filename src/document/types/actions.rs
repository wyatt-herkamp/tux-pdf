use derive_more::derive::From;
use either::Either;
use tux_pdf_low::types::{Dictionary, Object, PdfString, Stream};
/// Section 12.6
#[derive(Debug, Clone, PartialEq, From)]
pub struct PdfAction {
    pub action: PdfActionType,
    pub next: Vec<PdfAction>,
}
impl<T> From<T> for PdfAction
where
    PdfActionType: From<T>,
{
    fn from(value: T) -> Self {
        Self {
            action: PdfActionType::from(value),
            next: Vec::new(),
        }
    }
}
impl From<PdfAction> for Dictionary {
    fn from(value: PdfAction) -> Self {
        let mut dict: Dictionary = value.action.into();
        dict.set("Type", Object::name("Action"));
        if !value.next.is_empty() {
            let mut next = Vec::new();
            for action in value.next {
                next.push(action.into());
            }
            dict.set("Next", Object::Array(next));
        }
        dict
    }
}
impl From<PdfAction> for Object {
    fn from(value: PdfAction) -> Self {
        Dictionary::from(value).into()
    }
}
#[derive(Debug, Clone, PartialEq, From)]
pub enum PdfActionType {
    JavaScript(JavascriptAction),
}
impl From<PdfActionType> for Dictionary {
    fn from(value: PdfActionType) -> Self {
        match value {
            PdfActionType::JavaScript(js) => js.into(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct JavascriptAction {
    pub action: Either<PdfString, Stream>,
}
impl From<&str> for JavascriptAction {
    fn from(value: &str) -> Self {
        Self {
            action: Either::Left(PdfString::from(value)),
        }
    }
}
impl From<String> for JavascriptAction {
    fn from(value: String) -> Self {
        Self {
            action: Either::Left(PdfString::from(value)),
        }
    }
}

impl From<JavascriptAction> for Dictionary {
    fn from(value: JavascriptAction) -> Self {
        let mut dict = Dictionary::new();
        match value.action {
            Either::Left(s) => {
                dict.set("S", Object::name("JavaScript"));
                dict.set("JS", s);
            }
            Either::Right(s) => {
                dict.set("S", Object::name("JavaScript"));
                dict.set("JS", Object::Stream(s));
            }
        }
        dict
    }
}
