use std::borrow::Cow;

use chrono::{Datelike, Month, NaiveDate, Weekday};
use styles::MonthCalendarStyle;

use crate::{
    TuxPdfError,
    document::PdfDocument,
    graphics::{
        TextBlockContent, TextStyle,
        size::{RenderSize, Size},
    },
    layouts::table::{RowStyles, TablePageRules},
};
pub mod styles;
use super::{ChronoSpecialUtils, WeekStartDay};
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MonthCalendarLayoutType {
    MondayThroughFriday,
    MondayThroughSaturday,
    SevenDay,
}
impl MonthCalendarLayoutType {
    pub fn number_of_days(&self) -> u32 {
        match self {
            MonthCalendarLayoutType::MondayThroughFriday => 5,
            MonthCalendarLayoutType::MondayThroughSaturday => 6,
            MonthCalendarLayoutType::SevenDay => 7,
        }
    }
    pub fn weekdays(&self) -> Vec<chrono::Weekday> {
        match self {
            MonthCalendarLayoutType::MondayThroughFriday => vec![
                chrono::Weekday::Mon,
                chrono::Weekday::Tue,
                chrono::Weekday::Wed,
                chrono::Weekday::Thu,
                chrono::Weekday::Fri,
            ],
            MonthCalendarLayoutType::MondayThroughSaturday => vec![
                chrono::Weekday::Mon,
                chrono::Weekday::Tue,
                chrono::Weekday::Wed,
                chrono::Weekday::Thu,
                chrono::Weekday::Fri,
                chrono::Weekday::Sat,
            ],
            MonthCalendarLayoutType::SevenDay => vec![
                chrono::Weekday::Mon,
                chrono::Weekday::Tue,
                chrono::Weekday::Wed,
                chrono::Weekday::Thu,
                chrono::Weekday::Fri,
                chrono::Weekday::Sat,
                chrono::Weekday::Sun,
            ],
        }
    }
}

pub type WeekdayNameValue = fn(Weekday) -> TextBlockContent;

fn default_weekday_name(weekday: Weekday) -> TextBlockContent {
    match weekday {
        Weekday::Mon => TextBlockContent::from("Monday"),
        Weekday::Tue => TextBlockContent::from("Tuesday"),
        Weekday::Wed => TextBlockContent::from("Wednesday"),
        Weekday::Thu => TextBlockContent::from("Thursday"),
        Weekday::Fri => TextBlockContent::from("Friday"),
        Weekday::Sat => TextBlockContent::from("Saturday"),
        Weekday::Sun => TextBlockContent::from("Sunday"),
    }
}
pub trait StyleContent {}
pub struct MonthCalendarLayout {
    pub year: i32,
    pub month: chrono::Month,
    pub start_day: WeekStartDay,
    pub layout_type: MonthCalendarLayoutType,
    pub styles: MonthCalendarStyle,
    pub weekday_name: WeekdayNameValue,
    pub table_page_rules: TablePageRules,
}

impl Default for MonthCalendarLayout {
    fn default() -> Self {
        let local_date = chrono::Local::now();
        MonthCalendarLayout {
            year: local_date.year(),
            month: Month::try_from(local_date.month() as u8).unwrap(),
            start_day: WeekStartDay::Monday,
            layout_type: MonthCalendarLayoutType::SevenDay,
            styles: MonthCalendarStyle::default(),
            weekday_name: default_weekday_name,
            table_page_rules: TablePageRules::default(),
        }
    }
}
impl ChronoSpecialUtils for MonthCalendarLayout {
    fn year_internal(&self) -> i32 {
        self.year
    }
    fn month_internal(&self) -> chrono::Month {
        self.month
    }
}

impl MonthCalendarLayout {
    pub fn new(year: i32, month: chrono::Month) -> Self {
        MonthCalendarLayout {
            year,
            month,
            ..Default::default()
        }
    }
    pub fn with_start_day_and_layout(
        mut self,
        start_day: WeekStartDay,
        layout_type: MonthCalendarLayoutType,
    ) -> Self {
        if self.start_day == WeekStartDay::Sunday {
            assert!(
                layout_type == MonthCalendarLayoutType::SevenDay,
                "Sunday start day is only supported for 7 day layouts"
            );
        }
        self.layout_type = layout_type;
        self.start_day = start_day;
        self
    }

    fn number_of_weeks_row_inner(&self) -> u32 {
        self.number_of_week_rows(self.start_day)
    }
}
