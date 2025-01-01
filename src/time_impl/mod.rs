use time::format_description;
use time::OffsetDateTime;
use tux_pdf_low::types::Object;
pub fn format_time_offset_date_time(offset_date_time: OffsetDateTime) -> String {
    let format = "D:[year][month padding:zero][day padding:zero][hour padding:zero][minute padding:zero][second padding:zero][offset_hour sign:mandatory]'[offset_minute]";
    let format = format_description::parse(format).unwrap();
    offset_date_time.format(&format).unwrap()
}
pub trait PdfDateTimeType {
    fn format_pdf_date_time(&self) -> String;

    fn format_into_object(&self) -> Object {
        Object::string_literal_owned(self.format_pdf_date_time())
    }
}
impl PdfDateTimeType for OffsetDateTime {
    fn format_pdf_date_time(&self) -> String {
        format_time_offset_date_time(*self)
    }
}
#[cfg(test)]
mod tests {

    use time::OffsetDateTime;
    #[test]
    fn print_tests() {
        let now = OffsetDateTime::now_utc();
        let result = super::format_time_offset_date_time(now);
        println!("Result: {:?}", result);

        let now = OffsetDateTime::now_local().unwrap();
        let result = super::format_time_offset_date_time(now);
        println!("Result: {:?}", result);
    }
    #[test]
    fn test() {
        let now = OffsetDateTime::from_unix_timestamp(1734362989).unwrap();
        let result = super::format_time_offset_date_time(now);
        assert_eq!(result, "D:20241216152949+00'00");
    }
}
