//! Build a monthly calandar within a PDF page

use chrono::{Datelike, Month, NaiveDate, Weekday};
mod month;
mod utils;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeekStartDay {
    Monday,
    Sunday,
}
impl WeekStartDay {
    pub fn to_chrono_weekday(&self) -> Weekday {
        match self {
            WeekStartDay::Monday => Weekday::Mon,
            WeekStartDay::Sunday => Weekday::Sun,
        }
    }

    pub fn days_from_start(&self, weekday: Weekday) -> i32 {
        match self {
            WeekStartDay::Monday => weekday.num_days_from_monday() as i32,
            WeekStartDay::Sunday => weekday.num_days_from_sunday() as i32,
        }
    }
}
pub(crate) trait ChronoSpecialUtils {
    fn year_internal(&self) -> i32;
    fn year_last_digit(&self) -> i32 {
        let year = self.year_internal();
        year % 100
    }
    fn month_internal(&self) -> Month;

    fn year_code(&self) -> i32 {
        let last_two_digits = self.year_last_digit();

        (last_two_digits + (last_two_digits / 4)) % 7
    }
    fn month_code(&self) -> i32 {
        let month = self.month_internal();
        match month {
            Month::January => 0,
            Month::February => 3,
            Month::March => 3,
            Month::April => 6,
            Month::May => 1,
            Month::June => 4,
            Month::July => 6,
            Month::August => 2,
            Month::September => 5,
            Month::October => 0,
            Month::November => 3,
            Month::December => 5,
        }
    }

    fn century_code(&self) -> i32 {
        let year = self.year_internal();
        let century = year / 100;
        match century {
            17 => 4,
            18 => 2,
            19 => 0,
            20 => 6,
            21 => 4,
            22 => 2,
            _ => panic!("Invalid century"),
        }
    }
    fn days_in_month(&self) -> i32 {
        match self.month_internal() {
            Month::January => 31,
            Month::February => {
                if self.is_leap_year() {
                    29
                } else {
                    28
                }
            }
            Month::March => 31,
            Month::April => 30,
            Month::May => 31,
            Month::June => 30,
            Month::July => 31,
            Month::August => 31,
            Month::September => 30,
            Month::October => 31,
            Month::November => 30,
            Month::December => 31,
        }
    }

    fn days_in_year(&self) -> i32 {
        if self.is_leap_year() { 366 } else { 365 }
    }

    fn is_leap_year(&self) -> bool {
        let year = self.year_internal() as u32;
        return year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    }
    fn leap_year_code(&self) -> i32 {
        let month = self.month_internal();
        if self.is_leap_year() && (month == Month::January || month == Month::February) {
            1
        } else {
            0
        }
    }
    fn first_day_of_month(&self) -> chrono::Weekday {
        let raw_week_day = (self.year_code() + self.month_code() + self.century_code() + 1
            - self.leap_year_code())
            % 7;
        weekday_from_index(raw_week_day)
    }

    fn last_day_of_month(&self) -> chrono::Weekday {
        let raw_week_day =
            (self.year_code() + self.month_code() + self.century_code() + self.days_in_month()
                - self.leap_year_code())
                % 7;
        weekday_from_index(raw_week_day)
    }
    /// If you were to lay the month out on the calendar. How many rows would it take?

    fn number_of_week_rows(&self, starting_column: WeekStartDay) -> u32 {
        let first_day_of_month = self.first_day_of_month();
        let last_day_of_month = self.last_day_of_month();
        let days_in_month = self.days_in_month();
        let days_in_week = 7;

        let first_day_index = starting_column.days_from_start(first_day_of_month) as i32;
        let last_day_index = starting_column.days_from_start(last_day_of_month) as i32;

        let mut number_of_rows = 0;
        let mut days_left = days_in_month - (days_in_week - first_day_index) - last_day_index;
        if days_left > 0 {
            number_of_rows += 1;
            days_left -= days_in_week;
        }
        number_of_rows += (days_left / days_in_week) as u32;
        if days_left % days_in_week > 0 {
            number_of_rows += 1;
        }
        number_of_rows += 1; // Add one for the first week
        number_of_rows
    }
}

impl ChronoSpecialUtils for chrono::NaiveDate {
    fn year_internal(&self) -> i32 {
        <chrono::NaiveDate as chrono::Datelike>::year(self)
    }
    fn month_internal(&self) -> Month {
        Month::try_from(self.month() as u8).unwrap()
    }
}

impl ChronoSpecialUtils for (i32, Month) {
    fn year_internal(&self) -> i32 {
        self.0
    }
    fn month_internal(&self) -> Month {
        self.1
    }
}

mod tests {

    use chrono::Weekday;

    use super::*;
    #[test]
    fn test_first_day_of_month() {
        let test_values = vec![
            ((2025, Month::January), chrono::Weekday::Wed),
            ((2025, Month::February), chrono::Weekday::Sat),
            ((2025, Month::March), chrono::Weekday::Sat),
            ((2025, Month::April), chrono::Weekday::Tue),
            ((2025, Month::May), chrono::Weekday::Thu),
            ((2025, Month::June), chrono::Weekday::Sun),
            ((2025, Month::July), chrono::Weekday::Tue),
        ];
        for (input, expected) in test_values {
            let date = input.first_day_of_month();
            assert_eq!(date, expected, "Failed for input: {:?}", input);
        }
    }

    #[test]
    fn test_number_of_weeks_in_month() {
        let test_values = vec![
            ((2025, Month::January), 5, 5),
            ((2025, Month::February), 5, 5),
            ((2025, Month::March), 6, 6),
        ];
        for (input, expected_sun, expected_mon) in test_values {
            let date = input.number_of_week_rows(WeekStartDay::Sunday);
            assert_eq!(date, expected_sun, "Failed for input (SUNDAY): {:?}", input);

            let date = input.number_of_week_rows(WeekStartDay::Monday);
            assert_eq!(date, expected_mon, "Failed for input (MONDAY): {:?}", input);
        }
    }
}
fn weekday_from_index(index: i32) -> Weekday {
    match index {
        0 => chrono::Weekday::Sun,
        1 => chrono::Weekday::Mon,
        2 => chrono::Weekday::Tue,
        3 => chrono::Weekday::Wed,
        4 => chrono::Weekday::Thu,
        5 => chrono::Weekday::Fri,
        6 => chrono::Weekday::Sat,
        _ => panic!("Invalid week day"),
    }
}
